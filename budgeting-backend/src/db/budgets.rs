use std::collections::HashMap;

use anyhow::anyhow;
use rust_decimal::Decimal;
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::routes::{
    budgets::{Budget, BudgetTarget},
    schedule::Schedule,
};

use super::{schedule, DbError};

#[derive(Clone, Debug, PartialEq)]
struct BudgetDbModel {
    id: String,
    name: String,
    target_type: Option<String>,
    repeating_target_type: Option<String>,
    target_amount: Option<Decimal>,
    target_schedule_id: Option<String>,
    user_id: String,
}

impl From<Budget> for BudgetDbModel {
    fn from(value: Budget) -> Self {
        let (target_type, repeating_target_type, target_amount, target_schedule_id) =
            match &value.target {
                None => (None, None, None, None),
                Some(target @ BudgetTarget::OneTime { target_amount }) => {
                    (Some(target.to_string()), None, Some(*target_amount), None)
                }
                Some(
                    target @ BudgetTarget::Repeating {
                        target_amount,
                        repeating_type,
                        schedule,
                    },
                ) => (
                    Some(target.to_string()),
                    Some(repeating_type.to_string()),
                    Some(*target_amount),
                    Some(schedule.id.as_simple().to_string()),
                ),
            };
        Self {
            id: value.id.as_simple().to_string(),
            name: value.name,
            target_type,
            repeating_target_type,
            target_amount,
            target_schedule_id,
            user_id: value.user_id.as_simple().to_string(),
        }
    }
}

impl<'a> TryFrom<(String, Option<Decimal>, Option<String>, Option<Schedule>)> for BudgetTarget {
    type Error = anyhow::Error;

    fn try_from(
        (target_type, target_amount, repeating_target_type, maybe_schedule): (
            String,
            Option<Decimal>,
            Option<String>,
            Option<Schedule>,
        ),
    ) -> Result<Self, Self::Error> {
        match target_type.as_str() {
            "OneTime" => Ok(BudgetTarget::OneTime {
                target_amount: target_amount
                    .ok_or(anyhow!("Missing target_amount for OneTime target"))?,
            }),
            "Repeating" => Ok(BudgetTarget::Repeating {
                target_amount: target_amount
                    .ok_or(anyhow!("Missing target_amount for Repeating target"))?,
                repeating_type: repeating_target_type
                    .ok_or(anyhow!(
                        "Missing repeating_target_type for repeating target"
                    ))?
                    .parse()?,
                schedule: maybe_schedule
                    .ok_or(anyhow!("Missing schedule for Repeating budget target"))?,
            }),
            other => Err(anyhow!("Unexpected target_type {other}")),
        }
    }
}

impl TryFrom<(BudgetDbModel, Option<Schedule>)> for Budget {
    type Error = anyhow::Error;

    fn try_from(
        (value, maybe_schedule): (BudgetDbModel, Option<Schedule>),
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.parse()?,
            name: value.name,
            user_id: value.user_id.parse()?,
            target: value
                .target_type
                .map(|target_type| {
                    (
                        target_type,
                        value.target_amount,
                        value.repeating_target_type,
                        maybe_schedule,
                    )
                        .try_into()
                })
                .transpose()?,
        })
    }
}

pub async fn create_budget(db_pool: &MySqlPool, budget: Budget) -> Result<(), DbError> {
    if let Some(BudgetTarget::Repeating { schedule, .. }) = &budget.target {
        super::schedule::create_schedule(db_pool, schedule.clone()).await?;
    }

    let db_model: BudgetDbModel = budget.into();

    sqlx::query!(
r"INSERT INTO Budgets(id, name, target_type, repeating_target_type, target_amount, target_schedule_id, user_id)
VALUE(?, ?, ?, ?, ?, ?, ?)",
        db_model.id,
        db_model.name,
        db_model.target_type,
        db_model.repeating_target_type,
        db_model.target_amount,
        db_model.target_schedule_id,
        db_model.user_id)
        .execute(db_pool)
        .await?;

    Ok(())
}

pub async fn get_budgets(db_pool: &MySqlPool, user_id: Uuid) -> Result<Box<[Budget]>, DbError> {
    let budget_db_models = sqlx::query_as!(
        BudgetDbModel,
        r"SELECT id, name, target_type, repeating_target_type, target_amount, target_schedule_id, user_id
        FROM Budgets
        WHERE user_id = ?", user_id.as_simple())
        .fetch_all(db_pool)
        .await?;

    let schedule_ids = budget_db_models
        .iter()
        .filter_map(|b| b.target_schedule_id.as_ref())
        .map(|schedule_id| schedule_id.parse())
        .collect::<Result<Box<[Uuid]>, _>>()
        .map_err(|e| DbError::MappingError { error: e.into() })?;

    let schedules = schedule::get_schedules_by_ids(db_pool, &*schedule_ids).await?;

    let mut schedules: HashMap<_, _> = schedules
        .into_vec()
        .into_iter()
        .map(|s| (s.id, s))
        .collect();

    let mut budgets = Vec::new();

    // I'm not clever enough to do this with just iterators
    for db_model in budget_db_models {
        let schedule_id = db_model
            .target_schedule_id
            .as_ref()
            .map(|s| s.parse::<Uuid>())
            .transpose()
            .map_err(|e| DbError::MappingError { error: e.into() })?;

        // a schedule is owned by a single budget, so removing from schedules should be ok
        let schedule = schedule_id.map(|s| schedules.remove(&s)).flatten();

        let budget: Budget = (db_model, schedule)
            .try_into()
            .map_err(|e| DbError::MappingError { error: e })?;

        budgets.push(budget);
    }

    Ok(budgets.into_boxed_slice())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod mapping_tests {
        use chrono::NaiveDate;
        use rust_decimal::prelude::FromPrimitive;

        use crate::routes::{budgets::RepeatingTargetType, schedule::SchedulePeriod};

        use super::*;

        #[test]
        pub fn db_model_to_domain_budget() {
            let id = Uuid::new_v4();
            let user_id = Uuid::new_v4();
            let amount = Decimal::from_f32(10.1).unwrap();
            let schedule_id = Uuid::new_v4();

            let no_target = BudgetDbModel {
                id: id.as_simple().to_string(),
                user_id: user_id.as_simple().to_string(),
                name: "hi".into(),
                target_type: None,
                repeating_target_type: None,
                target_amount: None,
                target_schedule_id: None,
            };

            let one_time_target = BudgetDbModel {
                target_type: Some("OneTime".into()),
                target_amount: Some(amount),
                ..no_target.clone()
            };

            let repeating_target = BudgetDbModel {
                target_type: Some("Repeating".into()),
                target_amount: Some(amount),
                repeating_target_type: Some(RepeatingTargetType::BuildUpTo.to_string()),
                target_schedule_id: Some(schedule_id.as_simple().to_string()),
                ..no_target.clone()
            };

            let default_budget = Budget {
                id,
                name: "hi".into(),
                target: None,
                user_id,
            };

            let schedule = Schedule {
                id: schedule_id,
                period: SchedulePeriod::Weekly {
                    starting_on: NaiveDate::from_ymd_opt(2024, 9, 28).unwrap(),
                },
            };

            let pairs = [
                (
                    // no target
                    (no_target.clone(), None),
                    Budget {
                        target: None,
                        ..default_budget.clone()
                    },
                ),
                (
                    // one time target
                    (one_time_target.clone(), None),
                    Budget {
                        target: Some(BudgetTarget::OneTime {
                            target_amount: amount,
                        }),
                        ..default_budget.clone()
                    },
                ),
                (
                    (
                        BudgetDbModel {
                            repeating_target_type: Some("BuildUpTo".into()),
                            ..repeating_target.clone()
                        },
                        Some(schedule.clone()),
                    ),
                    Budget {
                        target: Some(BudgetTarget::Repeating {
                            target_amount: amount,
                            repeating_type: RepeatingTargetType::BuildUpTo,
                            schedule: schedule.clone(),
                        }),
                        ..default_budget.clone()
                    },
                ),
                (
                    (
                        BudgetDbModel {
                            repeating_target_type: Some("RequireRepeating".into()),
                            ..repeating_target.clone()
                        },
                        Some(schedule.clone()),
                    ),
                    Budget {
                        target: Some(BudgetTarget::Repeating {
                            target_amount: amount,
                            repeating_type: RepeatingTargetType::RequireRepeating,
                            schedule: schedule.clone(),
                        }),
                        ..default_budget.clone()
                    },
                ),
            ];

            for ((db_model, maybe_schedule), expected_budget) in pairs {
                let result: Result<Budget, _> = (db_model.clone(), maybe_schedule).try_into();

                assert!(result.is_ok());
                assert_eq!(result.unwrap(), expected_budget);

                let mapped_db_model: BudgetDbModel = expected_budget.into();
                assert_eq!(mapped_db_model, db_model);
            }
        }
    }

    mod db_tests {
        use chrono::NaiveDate;
        use rust_decimal::prelude::FromPrimitive;

        use crate::{
            db,
            models::CreateUserRequest,
            routes::{budgets::RepeatingTargetType, schedule::SchedulePeriod},
        };

        use super::*;

        #[sqlx::test]
        pub async fn create_and_get_budget_test(db_pool: MySqlPool) {
            let id = Uuid::new_v4();
            let user_id = Uuid::new_v4();
            let schedule_id = Uuid::new_v4();
            let budget = Budget {
                id,
                name: "name".into(),
                target: Some(BudgetTarget::Repeating {
                    target_amount: Decimal::from_f32(1.1).unwrap(),
                    repeating_type: RepeatingTargetType::RequireRepeating,
                    schedule: Schedule {
                        id: schedule_id,
                        period: SchedulePeriod::Weekly {
                            starting_on: NaiveDate::from_ymd_opt(2024, 9, 28).unwrap(),
                        },
                    },
                }),
                user_id,
            };

            db::users::create_user(
                &db_pool,
                user_id,
                CreateUserRequest::new("name".into(), "email@email.com".into()),
            )
            .await
            .unwrap();

            let result = create_budget(&db_pool, budget.clone()).await;
            assert!(result.is_ok());

            let fetched = get_budgets(&db_pool, user_id).await;
            assert!(fetched.is_ok());
            assert_eq!(fetched.unwrap(), vec![budget].into_boxed_slice())
        }
    }
}
