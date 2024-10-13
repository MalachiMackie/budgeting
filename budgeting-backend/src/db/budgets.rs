use std::collections::HashMap;

use anyhow::anyhow;
use rust_decimal::Decimal;
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::models::{Budget, BudgetTarget, Schedule};

use super::{schedule, Error};

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

impl BudgetTarget {
    fn try_new(
        target_type: &str,
        target_amount: Option<Decimal>,
        repeating_target_type: Option<&str>,
        schedule: Option<Schedule>,
    ) -> Result<Self, anyhow::Error> {
        match target_type {
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
                schedule: schedule
                    .ok_or(anyhow!("Missing schedule for Repeating budget target"))?,
            }),
            other => Err(anyhow!("Unexpected target_type {other}")),
        }
    }
}

impl BudgetDbModel {
    fn try_into_budget(self, schedule: Option<Schedule>) -> Result<Budget, anyhow::Error> {
        Ok(Budget {
            id: self.id.parse()?,
            name: self.name,
            user_id: self.user_id.parse()?,
            target: self
                .target_type
                .map(|target_type| {
                    BudgetTarget::try_new(
                        target_type.as_str(),
                        self.target_amount,
                        self.repeating_target_type.as_deref(),
                        schedule,
                    )
                })
                .transpose()?,
        })
    }
}

pub async fn create(db_pool: &MySqlPool, budget: Budget) -> Result<(), Error> {
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

pub async fn get_single(db_pool: &MySqlPool, id: Uuid) -> Result<Budget, Error> {
    let budget = sqlx::query_as!(
        BudgetDbModel,
        "SELECT id, name, target_type, repeating_target_type, target_amount, target_schedule_id, user_id
        FROM Budgets
        WHERE id = ?",
        id.as_simple()
    ).fetch_one(db_pool)
        .await?;

    let schedule = if let Some(schedule_id) = &budget.target_schedule_id {
        let schedule_id = schedule_id
            .parse::<Uuid>()
            .map_err(|e| Error::MappingError { error: e.into() })?;

        Some(schedule::get_single(db_pool, schedule_id).await?)
    } else {
        None
    };

    budget
        .try_into_budget(schedule)
        .map_err(|e| Error::MappingError { error: e })
}

pub async fn get(db_pool: &MySqlPool, user_id: Uuid) -> Result<Box<[Budget]>, Error> {
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
        .map_err(|e| Error::MappingError { error: e.into() })?;

    let schedules = if schedule_ids.is_empty() {
        Box::new([])
    } else {
        schedule::get_by_ids(db_pool, &schedule_ids).await?
    };

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
            .map_err(|e| Error::MappingError { error: e.into() })?;

        // a schedule is owned by a single budget, so removing from schedules should be ok
        let schedule = schedule_id.and_then(|s| schedules.remove(&s));

        let budget: Budget = db_model
            .try_into_budget(schedule)
            .map_err(|e| Error::MappingError { error: e })?;

        budgets.push(budget);
    }

    Ok(budgets.into_boxed_slice())
}

pub async fn delete(db_pool: &MySqlPool, id: Uuid) -> Result<(), Error> {

    sqlx::query!("DELETE FROM Budgets WHERE id = ?", id.as_simple())
        .execute(db_pool)
        .await?;

    Ok(())
}

pub async fn update(db_pool: &MySqlPool, budget: Budget) -> Result<(), Error> {
    let db_model: BudgetDbModel = budget.into();
    sqlx::query!(
        "UPDATE Budgets
    SET name = ?,
    target_type = ?,
    repeating_target_type = ?,
    target_amount = ?,
    target_schedule_id = ?",
        db_model.name,
        db_model.target_type,
        db_model.repeating_target_type,
        db_model.target_amount,
        db_model.target_schedule_id,
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod mapping_tests {
        use chrono::NaiveDate;
        use rust_decimal::prelude::FromPrimitive;

        use crate::models::{RepeatingTargetType, SchedulePeriod};

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
                let result = db_model.clone().try_into_budget(maybe_schedule).unwrap();

                assert_eq!(result, expected_budget);

                let mapped_db_model: BudgetDbModel = expected_budget.into();
                assert_eq!(mapped_db_model, db_model);
            }
        }
    }

    mod db_tests {
        use std::sync::OnceLock;

        use chrono::NaiveDate;
        use rust_decimal::prelude::FromPrimitive;
        use rust_decimal_macros::dec;

        use crate::{
            db,
            extensions::once_lock::OnceLockExt,
            models::{CreateUserRequest, RepeatingTargetType, SchedulePeriod},
        };

        use super::*;

        static USER_ID: OnceLock<Uuid> = OnceLock::new();

        async fn test_init(db_pool: &MySqlPool) {
            let user_id = *USER_ID.init_uuid();

            db::users::create(
                db_pool,
                user_id,
                CreateUserRequest::new("name".into(), "email@email.com".into()),
            )
            .await
            .unwrap();
        }

        #[sqlx::test]
        pub async fn create_and_get_budget_test(db_pool: MySqlPool) {
            test_init(&db_pool).await;

            let id = Uuid::new_v4();
            let user_id = *USER_ID.get().unwrap();
            let schedule_id = Uuid::new_v4();
            let schedule = Schedule {
                id: schedule_id,
                period: SchedulePeriod::Weekly {
                    starting_on: NaiveDate::from_ymd_opt(2024, 9, 28).unwrap(),
                },
            };

            db::schedule::create(&db_pool, schedule.clone())
                .await
                .unwrap();

            let budget = Budget {
                id,
                name: "name".into(),
                target: Some(BudgetTarget::Repeating {
                    target_amount: Decimal::from_f32(1.1).unwrap(),
                    repeating_type: RepeatingTargetType::RequireRepeating,
                    schedule,
                }),
                user_id,
            };

            create(&db_pool, budget.clone()).await.unwrap();

            let fetched = get(&db_pool, user_id).await.unwrap();
            assert_eq!(fetched, vec![budget.clone()].into_boxed_slice());

            let fetched_single = get_single(&db_pool, id).await.unwrap();
            assert_eq!(fetched_single, budget);
        }

        #[sqlx::test]
        pub async fn get_budgets_without_schedule(db_pool: MySqlPool) {
            test_init(&db_pool).await;
            let id = Uuid::new_v4();
            let user_id = *USER_ID.get().unwrap();

            let budget = Budget {
                id,
                name: "name".into(),
                target: Some(BudgetTarget::OneTime {
                    target_amount: Decimal::from_f32(1.1).unwrap(),
                }),
                user_id,
            };

            create(&db_pool, budget.clone()).await.unwrap();

            let fetched = get(&db_pool, user_id).await.unwrap();
            assert_eq!(fetched, vec![budget].into_boxed_slice());
        }

        #[sqlx::test]
        pub async fn update_budget_add_schedule(db_pool: MySqlPool) {
            test_init(&db_pool).await;
            let id = Uuid::new_v4();
            let user_id = *USER_ID.get().unwrap();
            let new_schedule_id = Uuid::new_v4();

            let budget = Budget {
                id,
                name: "name".into(),
                target: None,
                user_id,
            };

            create(&db_pool, budget.clone()).await.unwrap();

            let new_schedule = Schedule {
                id: new_schedule_id,
                period: SchedulePeriod::Weekly {
                    starting_on: NaiveDate::from_ymd_opt(2024, 10, 7).unwrap(),
                },
            };

            schedule::create(&db_pool, new_schedule.clone())
                .await
                .unwrap();

            let target = BudgetTarget::Repeating {
                target_amount: dec!(1.2),
                repeating_type: RepeatingTargetType::BuildUpTo,
                schedule: new_schedule,
            };

            let updated = Budget::new(id, "newName".into(), Some(target), user_id);

            update(&db_pool, updated.clone()).await.unwrap();

            let fetched = get_single(&db_pool, id).await.unwrap();

            assert_eq!(fetched, updated);
        }

        #[sqlx::test]
        pub async fn update_budget_remove_schedule_onetime_target(db_pool: MySqlPool) {
            test_init(&db_pool).await;
            let id = Uuid::new_v4();
            let user_id = *USER_ID.get().unwrap();
            let schedule_id = Uuid::new_v4();

            let schedule = Schedule {
                id: schedule_id,
                period: SchedulePeriod::Weekly {
                    starting_on: NaiveDate::from_ymd_opt(2024, 10, 7).unwrap(),
                },
            };

            schedule::create(&db_pool, schedule.clone())
                .await
                .unwrap();

            let target = BudgetTarget::Repeating {
                target_amount: dec!(1.2),
                repeating_type: RepeatingTargetType::BuildUpTo,
                schedule,
            };

            let budget = Budget {
                id,
                name: "name".into(),
                target: Some(target.clone()),
                user_id,
            };

            create(&db_pool, budget.clone()).await.unwrap();

            let updated_target = BudgetTarget::OneTime {
                target_amount: dec!(1.2),
            };

            let updated = Budget::new(id, "newName".into(), Some(updated_target), user_id);

            update(&db_pool, updated.clone()).await.unwrap();

            let fetched = get_single(&db_pool, id).await.unwrap();

            assert_eq!(fetched, updated);
        }

        #[sqlx::test]
        pub async fn update_budget_remove_schedule_no_target(db_pool: MySqlPool) {
            test_init(&db_pool).await;
            let id = Uuid::new_v4();
            let user_id = *USER_ID.get().unwrap();
            let schedule_id = Uuid::new_v4();

            let schedule = Schedule {
                id: schedule_id,
                period: SchedulePeriod::Weekly {
                    starting_on: NaiveDate::from_ymd_opt(2024, 10, 7).unwrap(),
                },
            };

            schedule::create(&db_pool, schedule.clone())
                .await
                .unwrap();

            let target = BudgetTarget::Repeating {
                target_amount: dec!(1.2),
                repeating_type: RepeatingTargetType::BuildUpTo,
                schedule,
            };

            let budget = Budget {
                id,
                name: "name".into(),
                target: Some(target.clone()),
                user_id,
            };

            create(&db_pool, budget.clone()).await.unwrap();

            let updated = Budget::new(id, "newName".into(), None, user_id);

            update(&db_pool, updated.clone()).await.unwrap();

            let fetched = get_single(&db_pool, id).await.unwrap();

            assert_eq!(fetched, updated);
        }

        #[sqlx::test]
        pub async fn update_budget_no_schedule(db_pool: MySqlPool) {
            test_init(&db_pool).await;
            let id = Uuid::new_v4();
            let user_id = *USER_ID.get().unwrap();

            let budget = Budget {
                id,
                name: "name".into(),
                target: None,
                user_id,
            };

            create(&db_pool, budget.clone()).await.unwrap();

            let updated = Budget::new(id, "newName".into(), None, user_id);

            update(&db_pool, updated.clone()).await.unwrap();

            let fetched = get_single(&db_pool, id).await.unwrap();

            assert_eq!(fetched, updated);
        }

        #[sqlx::test]
        pub async fn update_budget_schedule(db_pool: MySqlPool) {
            test_init(&db_pool).await;
            let id = Uuid::new_v4();
            let user_id = *USER_ID.get().unwrap();
            let schedule_id = Uuid::new_v4();

            let schedule = Schedule {
                id: schedule_id,
                period: SchedulePeriod::Weekly {
                    starting_on: NaiveDate::from_ymd_opt(2024, 10, 7).unwrap(),
                },
            };

            schedule::create(&db_pool, schedule.clone())
                .await
                .unwrap();

            let target = BudgetTarget::Repeating {
                target_amount: dec!(1.2),
                repeating_type: RepeatingTargetType::BuildUpTo,
                schedule,
            };

            let budget = Budget {
                id,
                name: "name".into(),
                target: Some(target.clone()),
                user_id,
            };

            create(&db_pool, budget.clone()).await.unwrap();

            let updated_schedule = Schedule {
                id: schedule_id,
                period: SchedulePeriod::Monthly {
                    starting_on: NaiveDate::from_ymd_opt(2024, 10, 13).unwrap(),
                },
            };

            schedule::update(&db_pool, updated_schedule.clone())
                .await
                .unwrap();

            let updated_target = BudgetTarget::Repeating {
                target_amount: dec!(1.2),
                repeating_type: RepeatingTargetType::BuildUpTo,
                schedule: updated_schedule,
            };

            let updated = Budget::new(id, "newName".into(), Some(updated_target), user_id);

            update(&db_pool, updated.clone()).await.unwrap();

            let fetched = get_single(&db_pool, id).await.unwrap();

            assert_eq!(fetched, updated);
        }

        #[sqlx::test]
        pub async fn delete_budget_test(db_pool: MySqlPool) {
            test_init(&db_pool).await;
            let id = Uuid::new_v4();
            let user_id = *USER_ID.get().unwrap();

            let budget = Budget {
                id,
                name: "name".into(),
                target: Some(BudgetTarget::OneTime {
                    target_amount: Decimal::from_f32(1.1).unwrap(),
                }),
                user_id,
            };

            create(&db_pool, budget).await.unwrap();

            delete(&db_pool, id).await.unwrap();

            let find_result = get(&db_pool, user_id).await.unwrap();

            assert_eq!(find_result.len(), 0);
        }
    }
}
