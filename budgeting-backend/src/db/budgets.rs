use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use anyhow::anyhow;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::{prelude::Type, FromRow, MySql, MySqlPool, QueryBuilder};
use uuid::Uuid;

use crate::models::{Budget, BudgetAssignment, BudgetAssignmentSource, BudgetTarget, Schedule};

use super::{schedule, Error};

#[derive(Clone, Debug, PartialEq, FromRow)]
struct BudgetDbModel {
    id: uuid::fmt::Simple,
    name: String,
    target_type: Option<String>,
    repeating_target_type: Option<String>,
    target_amount: Option<Decimal>,
    target_schedule_id: Option<uuid::fmt::Simple>,
    user_id: uuid::fmt::Simple,
    #[sqlx(skip)]
    assignments: Vec<BudgetAssignmentDbModel>,
}

#[derive(Clone, Debug, PartialEq, FromRow)]
struct BudgetAssignmentDbModel {
    budget_id: uuid::fmt::Simple,
    id: uuid::fmt::Simple,
    amount: Decimal,
    date: NaiveDate,
    assignment_type: String,
    from_budget_id: Option<uuid::fmt::Simple>,
    link_id: Option<uuid::fmt::Simple>,
    from_transaction_id: Option<uuid::fmt::Simple>,
}

#[derive(Clone, Debug, PartialEq, Type)]
enum BudgetAssignmentType {
    MoveBetweenBudgets,
    FromTransaction,
}

impl FromStr for BudgetAssignmentType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "MoveBetweenBudgets" => Ok(BudgetAssignmentType::MoveBetweenBudgets),
            "FromTransaction" => Ok(BudgetAssignmentType::FromTransaction),
            _ => Err(anyhow!("{s} is not a valid BudgetAssignmentType"))
        }
    }
}

impl Display for BudgetAssignmentType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BudgetAssignmentType::MoveBetweenBudgets => f.write_str("MoveBetweenBudgets"),
            BudgetAssignmentType::FromTransaction => f.write_str("FromTransaction"),
        }
    }
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
                    Some(schedule.id.simple()),
                ),
            };
        Self {
            id: value.id.simple(),
            name: value.name,
            target_type,
            repeating_target_type,
            target_amount,
            target_schedule_id,
            user_id: value.user_id.simple(),
            assignments: value
                .assignments
                .into_iter()
                .map(|assignment| BudgetAssignmentDbModel {
                    budget_id: value.id.simple(),
                    id: assignment.id.simple(),
                    amount: assignment.amount,
                    date: assignment.date,
                    assignment_type: match &assignment.source {
                        BudgetAssignmentSource::OtherBudget { .. } => {
                            "MoveBetweenBudgets".into()
                        }
                        BudgetAssignmentSource::Transaction { .. } => {
                            "FromTransaction".into()
                        }
                    },
                    link_id: if let BudgetAssignmentSource::OtherBudget { link_id, .. } = &assignment.source {
                        Some(link_id.simple())
                    } else {
                        None
                    },
                    from_budget_id: if let BudgetAssignmentSource::OtherBudget { from_budget_id, .. } =
                        &assignment.source
                    {
                        Some(from_budget_id.simple())
                    } else {
                        None
                    },
                    from_transaction_id: if let BudgetAssignmentSource::Transaction {
                        from_transaction_id,
                    } = &assignment.source
                    {
                        Some(from_transaction_id.simple())
                    } else {
                        None
                    },
                })
                .collect(),
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
            id: self.id.into_uuid(),
            name: self.name,
            user_id: self.user_id.into_uuid(),
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
            assignments: self
                .assignments
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl TryInto<BudgetAssignment> for BudgetAssignmentDbModel {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<BudgetAssignment, Self::Error> {
        let assignment_type: BudgetAssignmentType = self.assignment_type.parse()?;
        match (assignment_type, self.from_budget_id, self.from_transaction_id, self.link_id) {
            (BudgetAssignmentType::FromTransaction, _, Some(from_transaction_id), _) =>
                Ok(BudgetAssignment {
                    id: self.id.into(),
                    date: self.date,
                    amount: self.amount,
                    source: BudgetAssignmentSource::Transaction {
                        from_transaction_id: from_transaction_id.into_uuid(),
                    }
                }),
            (BudgetAssignmentType::FromTransaction, _, _, _) => Err(anyhow!("from_transaction_id must be populated for FromTransaction assignment type")),
            (BudgetAssignmentType::MoveBetweenBudgets, Some(from_budget_id), _, Some(link_id)) => {
                Ok(BudgetAssignment {
                    id: self.id.into(),
                    date: self.date,
                    amount: self.amount,
                    source: BudgetAssignmentSource::OtherBudget {
                        from_budget_id: from_budget_id.into(),
                        link_id: link_id.into(),
                    }
                })
            },
            (BudgetAssignmentType::MoveBetweenBudgets, _, _, _) => Err(anyhow!("from_budget_id and link_id must be populated for MoveBetweenBudgets assignment type")),
        }
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

    if !db_model.assignments.is_empty() {
        let mut query_builder =
            QueryBuilder::new("INSERT INTO BudgetAssignments (id, amount, date, budget_id, assignment_type, from_budget_id, from_transaction_id, link_id)");
        query_builder.push_values(db_model.assignments, |mut b, assignment| {
            b.push_bind(assignment.id)
                .push_bind(assignment.amount)
                .push_bind(assignment.date)
                .push_bind(assignment.budget_id)
                .push_bind(assignment.assignment_type.to_string())
                .push_bind(assignment.from_budget_id)
                .push_bind(assignment.from_transaction_id)
                .push_bind(assignment.link_id);
        });

        query_builder.build().execute(db_pool).await?;
    }

    Ok(())
}

pub async fn get_by_ids(db_pool: &MySqlPool, ids: &[Uuid]) -> Result<Box<[Budget]>, Error> {
    let mut query_builder = QueryBuilder::new(
        "SELECT id, name, target_type, repeating_target_type, target_amount, target_schedule_id, user_id
        FROM Budgets
        WHERE id IN (");

    let mut separated = query_builder.separated(',');
    for id in ids {
        separated.push_bind(id.as_simple());
    }

    separated.push_unseparated(')');

    let budget_db_models = query_builder
        .build_query_as::<BudgetDbModel>()
        .fetch_all(db_pool)
        .await?;

    get_budgets_from_db_models(db_pool, budget_db_models).await
}

pub async fn get_single(db_pool: &MySqlPool, id: Uuid) -> Result<Budget, Error> {
    let mut budget = sqlx::query_as::<MySql, BudgetDbModel>(
        "SELECT id, name, target_type, repeating_target_type, target_amount, target_schedule_id, user_id
        FROM Budgets
        WHERE id = ?").bind(id.simple()).fetch_one(db_pool)
        .await?;

    let assignments = sqlx::query_as::<MySql, BudgetAssignmentDbModel>(
        "SELECT id, amount, date, budget_id, assignment_type, from_budget_id, from_transaction_id, link_id
        FROM BudgetAssignments
        WHERE budget_id = ?",
    )
    .bind(id.simple())
    .fetch_all(db_pool)
    .await?;

    budget.assignments = assignments;

    let schedule = if let Some(schedule_id) = &budget.target_schedule_id {
        Some(schedule::get_single(db_pool, schedule_id.into_uuid()).await?)
    } else {
        None
    };

    budget
        .try_into_budget(schedule)
        .map_err(|e| Error::MappingError { error: e })
}

async fn get_budgets_from_db_models(
    db_pool: &MySqlPool,
    budget_db_models: Vec<BudgetDbModel>,
) -> Result<Box<[Budget]>, Error> {
    let mut schedule_ids = Vec::new();
    let mut budget_ids = Vec::new();
    for budget in &budget_db_models {
        if let Some(target_schedule_id) = &budget.target_schedule_id {
            schedule_ids.push(target_schedule_id.into_uuid());
        }

        budget_ids.push(budget.id);
    }

    if budget_db_models.is_empty() {
        return Ok(Box::new([]));
    }

    let mut query_builder = QueryBuilder::new(
        "SELECT id, amount, date, budget_id, assignment_type, from_budget_id, from_transaction_id, link_id FROM BudgetAssignments WHERE budget_id IN (",
    );

    let mut separated = query_builder.separated(",");
    for budget_id in budget_ids {
        separated.push_bind(budget_id);
    }
    separated.push_unseparated(")");

    let assignments_vec = query_builder
        .build_query_as::<BudgetAssignmentDbModel>()
        .fetch_all(db_pool)
        .await?;

    let mut assignments_by_budget_id: HashMap<uuid::fmt::Simple, Vec<_>> = HashMap::new();

    for assignment in assignments_vec {
        if let Some(assignments) = assignments_by_budget_id.get_mut(&assignment.budget_id) {
            assignments.push(assignment);
        } else {
            assignments_by_budget_id.insert(assignment.budget_id, vec![assignment]);
        }
    }

    let schedules = if schedule_ids.is_empty() {
        Box::new([])
    } else {
        schedule::get_by_ids(db_pool, &schedule_ids).await?
    };

    let mut schedules: HashMap<_, _> = schedules
        .into_vec()
        .into_iter()
        .map(|s| (s.id.simple(), s))
        .collect();

    let mut budgets = Vec::new();

    // I'm not clever enough to do this with just iterators
    for mut db_model in budget_db_models {
        let schedule_id = db_model.target_schedule_id;

        // a schedule is owned by a single budget, so removing from schedules should be ok
        let schedule = schedule_id.and_then(|s| schedules.remove(&s));

        let assignments = assignments_by_budget_id
            .remove(&db_model.id)
            .unwrap_or_default();

        db_model.assignments = assignments;

        let budget: Budget = db_model
            .try_into_budget(schedule)
            .map_err(|e| Error::MappingError { error: e })?;

        budgets.push(budget);
    }

    Ok(budgets.into_boxed_slice())
}

pub async fn get(db_pool: &MySqlPool, user_id: Uuid) -> Result<Box<[Budget]>, Error> {
    let budget_db_models = sqlx::query_as::<MySql, BudgetDbModel>(
        r"SELECT id, name, target_type, repeating_target_type, target_amount, target_schedule_id, user_id
        FROM Budgets
        WHERE user_id = ?").bind(user_id.simple())
        .fetch_all(db_pool)
        .await?;

    get_budgets_from_db_models(db_pool, budget_db_models).await
}

pub async fn delete(db_pool: &MySqlPool, id: Uuid) -> Result<(), Error> {
    sqlx::query!(
        "DELETE FROM BudgetAssignments WHERE budget_id = ?",
        id.as_simple()
    )
    .execute(db_pool)
    .await?;

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

    if !db_model.assignments.is_empty() {
        let mut query_builder =
            QueryBuilder::new("INSERT IGNORE INTO BudgetAssignments (id, amount, date, budget_id, assignment_type, from_budget_id, from_transaction_id, link_id)");
        query_builder.push_values(db_model.assignments, |mut b, assignment| {
            b.push_bind(assignment.id)
                .push_bind(assignment.amount)
                .push_bind(assignment.date)
                .push_bind(assignment.budget_id)
                .push_bind(assignment.assignment_type.to_string())
                .push_bind(assignment.from_budget_id)
                .push_bind(assignment.from_transaction_id)
                .push_bind(assignment.link_id);
        });

        query_builder.build().execute(db_pool).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod mapping_tests {
        use chrono::NaiveDate;
        use rust_decimal::prelude::FromPrimitive;
        use rust_decimal_macros::dec;

        use crate::models::{RepeatingTargetType, SchedulePeriod};

        use super::*;

        #[test]
        pub fn db_model_to_domain_budget() {
            let id = Uuid::new_v4();
            let user_id = Uuid::new_v4();
            let amount = Decimal::from_f32(10.1).unwrap();
            let schedule_id = Uuid::new_v4();
            let assignment_id1 = Uuid::new_v4();
            let assignment_id2 = Uuid::new_v4();
            let assignment_amount = dec!(15.2);
            let from_budget_id = Uuid::new_v4();
            let from_transaction_id = Uuid::new_v4();

            let default_assignment = BudgetAssignment {
                id: assignment_id1,
                amount: assignment_amount,
                date: NaiveDate::from_ymd_opt(2024, 11, 28).unwrap(),
                source: BudgetAssignmentSource::Transaction {
                    from_transaction_id,
                },
            };

            let link_id = Uuid::new_v4();

            let no_target = BudgetDbModel {
                id: id.simple(),
                user_id: user_id.simple(),
                name: "hi".into(),
                target_type: None,
                repeating_target_type: None,
                target_amount: None,
                target_schedule_id: None,
                assignments: vec![
                    BudgetAssignmentDbModel {
                        id: assignment_id1.simple(),
                        amount: assignment_amount,
                        budget_id: id.simple(),
                        date: NaiveDate::from_ymd_opt(2024, 11, 28).unwrap(),
                        assignment_type: BudgetAssignmentType::FromTransaction.to_string(),
                        from_budget_id: None,
                        from_transaction_id: Some(from_transaction_id.simple()),
                        link_id: None
                    },
                    {
                        BudgetAssignmentDbModel {
                            id: assignment_id2.simple(),
                            amount: assignment_amount,
                            budget_id: id.simple(),
                            date: NaiveDate::from_ymd_opt(2024, 11, 28).unwrap(),
                            assignment_type: BudgetAssignmentType::MoveBetweenBudgets.to_string(),
                            from_budget_id: Some(from_budget_id.simple()),
                            from_transaction_id: None,
                            link_id: Some(link_id.simple())
                        }
                    },
                ],
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
                target_schedule_id: Some(schedule_id.simple()),
                ..no_target.clone()
            };

            let default_budget = Budget {
                id,
                name: "hi".into(),
                target: None,
                user_id,
                assignments: vec![
                    default_assignment.clone(),
                    BudgetAssignment {
                        source: BudgetAssignmentSource::OtherBudget { from_budget_id, link_id },
                        ..default_assignment.clone()
                    },
                ],
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
        use std::sync::LazyLock;

        use chrono::NaiveDate;
        use rust_decimal::prelude::FromPrimitive;
        use rust_decimal_macros::dec;

        use crate::{
            db,
            models::{
                CreateBankAccountRequest, CreatePayeeRequest, CreateTransactionRequest,
                RepeatingTargetType, SchedulePeriod, User,
            },
        };

        use super::*;

        static USER_ID: LazyLock<Uuid> = LazyLock::new(Uuid::new_v4);

        async fn test_init(db_pool: &MySqlPool) {
            let user_id = *USER_ID;

            db::users::create(
                db_pool,
                User::new(user_id, "name".into(), "email@email.com".into(), None),
            )
            .await
            .unwrap();
        }

        #[sqlx::test]
        pub async fn create_and_get_budget_test(db_pool: MySqlPool) {
            test_init(&db_pool).await;

            let without_assignments_id = Uuid::new_v4();
            let with_assignments_id = Uuid::new_v4();
            let from_transaction_id = Uuid::new_v4();
            let user_id = *USER_ID;
            let schedule_id = Uuid::new_v4();
            let schedule = Schedule {
                id: schedule_id,
                period: SchedulePeriod::Weekly {
                    starting_on: NaiveDate::from_ymd_opt(2024, 9, 28).unwrap(),
                },
            };

            let without_assignments = Budget {
                id: without_assignments_id,
                name: "name".into(),
                target: Some(BudgetTarget::Repeating {
                    target_amount: Decimal::ZERO,
                    repeating_type: RepeatingTargetType::RequireRepeating,
                    schedule: schedule.clone(),
                }),
                user_id,
                assignments: vec![],
            };
            db::schedule::create(&db_pool, schedule.clone())
                .await
                .unwrap();
            create(&db_pool, without_assignments.clone()).await.unwrap();

            let bank_account_id = Uuid::new_v4();
            db::bank_accounts::create(
                &db_pool,
                bank_account_id,
                CreateBankAccountRequest::new("name".into(), Decimal::ZERO, user_id),
            )
            .await
            .unwrap();

            let payee_id = Uuid::new_v4();
            db::payees::create(
                &db_pool,
                payee_id,
                CreatePayeeRequest::new("name".into(), user_id),
            )
            .await
            .unwrap();

            db::transactions::create(
                &db_pool,
                from_transaction_id,
                bank_account_id,
                CreateTransactionRequest::new(
                    payee_id,
                    Decimal::ZERO,
                    NaiveDate::from_ymd_opt(2024, 11, 19).unwrap(),
                    without_assignments_id,
                ),
            )
            .await
            .unwrap();

            let link_id = Uuid::new_v4();

            let assignments = vec![
                BudgetAssignment {
                    id: Uuid::new_v4(),
                    amount: Decimal::ZERO,
                    date: NaiveDate::from_ymd_opt(2024, 11, 28).unwrap(),
                    source: BudgetAssignmentSource::Transaction {
                        from_transaction_id,
                    },
                },
                BudgetAssignment {
                    id: Uuid::new_v4(),
                    amount: Decimal::ZERO,
                    date: NaiveDate::from_ymd_opt(2024, 11, 28).unwrap(),
                    source: BudgetAssignmentSource::OtherBudget {
                        from_budget_id: without_assignments_id,
                        link_id: link_id.into()
                    },
                },
            ];

            let with_assignments = Budget {
                id: with_assignments_id,
                name: "name".into(),
                target: Some(BudgetTarget::Repeating {
                    target_amount: Decimal::ZERO,
                    repeating_type: RepeatingTargetType::RequireRepeating,
                    schedule,
                }),
                user_id,
                assignments,
            };

            create(&db_pool, with_assignments.clone()).await.unwrap();

            let fetched = get(&db_pool, user_id).await.unwrap();
            assert_eq!(
                fetched,
                vec![without_assignments.clone(), with_assignments.clone()].into_boxed_slice()
            );

            let fetched_without_assignments =
                get_single(&db_pool, without_assignments_id).await.unwrap();
            let fetched_with_assignments = get_single(&db_pool, with_assignments_id).await.unwrap();
            assert_eq!(fetched_without_assignments, without_assignments);
            assert_eq!(fetched_with_assignments, with_assignments);
        }

        #[sqlx::test]
        pub async fn get_by_ids_test(db_pool: MySqlPool) {
            test_init(&db_pool).await;

            let budget1 = Budget {
                assignments: vec![],
                id: Uuid::new_v4(),
                user_id: *USER_ID,
                name: "name1".into(),
                target: None,
            };
            let budget2 = Budget {
                id: Uuid::new_v4(),
                name: "name2".into(),
                ..budget1.clone()
            };

            create(&db_pool, budget1.clone()).await.unwrap();
            create(&db_pool, budget2.clone()).await.unwrap();

            let mut result = Vec::from(
                get_by_ids(&db_pool, &[budget1.id, budget2.id])
                    .await
                    .unwrap(),
            );

            result.sort_by(|a, b| a.id.cmp(&b.id));

            let mut expected = vec![budget1, budget2];
            expected.sort_by(|a, b| a.id.cmp(&b.id));

            assert_eq!(result, expected);
        }

        #[sqlx::test]
        pub async fn get_by_ids_when_empty_ids(db_pool: MySqlPool) {
            test_init(&db_pool).await;

            let result = get_by_ids(&db_pool, &[]).await.unwrap();

            assert!(result.is_empty());
        }

        #[sqlx::test]
        pub async fn get_budgets_without_schedule(db_pool: MySqlPool) {
            test_init(&db_pool).await;
            let id = Uuid::new_v4();
            let user_id = *USER_ID;
            let other_budget_id = Uuid::new_v4();

            create(&db_pool, Budget {
                id: other_budget_id,
                user_id,
                assignments: vec![],
                name: "name".into(),
                target: None
            }).await.unwrap();

            let budget = Budget {
                id,
                name: "name".into(),
                target: Some(BudgetTarget::OneTime {
                    target_amount: Decimal::from_f32(1.1).unwrap(),
                }),
                user_id,
                assignments: vec![BudgetAssignment {
                    id: Uuid::new_v4(),
                    amount: Decimal::ZERO,
                    date: NaiveDate::from_ymd_opt(2024, 11, 30).unwrap(),
                    source: BudgetAssignmentSource::OtherBudget { from_budget_id: other_budget_id, link_id: Uuid::new_v4() }
                }],
            };

            create(&db_pool, budget.clone()).await.unwrap();

            let fetched = get(&db_pool, user_id).await.unwrap();
            assert_eq!(fetched, vec![budget].into_boxed_slice());
        }

        #[sqlx::test]
        pub async fn update_budget_add_schedule(db_pool: MySqlPool) {
            test_init(&db_pool).await;
            let id1 = Uuid::new_v4();
            let id2 = Uuid::new_v4();
            let from_transaction_id = Uuid::new_v4();
            let user_id = *USER_ID;
            let new_schedule_id = Uuid::new_v4();

            create(
                &db_pool,
                Budget {
                    id: id2,
                    name: "name2".into(),
                    target: None,
                    user_id,
                    assignments: vec![],
                },
            )
            .await
            .unwrap();

            let bank_account_id = Uuid::new_v4();
            db::bank_accounts::create(
                &db_pool,
                bank_account_id,
                CreateBankAccountRequest::new("name".into(), Decimal::ZERO, user_id),
            )
            .await
            .unwrap();

            let payee_id = Uuid::new_v4();
            db::payees::create(
                &db_pool,
                payee_id,
                CreatePayeeRequest::new("name".into(), user_id),
            )
            .await
            .unwrap();

            db::transactions::create(
                &db_pool,
                from_transaction_id,
                bank_account_id,
                CreateTransactionRequest::new(
                    payee_id,
                    Decimal::ZERO,
                    NaiveDate::from_ymd_opt(2024, 11, 19).unwrap(),
                    id2,
                ),
            )
            .await
            .unwrap();

            let budget = Budget {
                id: id1,
                name: "name".into(),
                target: None,
                user_id,
                assignments: vec![
                    BudgetAssignment {
                        id: Uuid::new_v4(),
                        amount: Decimal::ZERO,
                        date: NaiveDate::from_ymd_opt(2024, 11, 28).unwrap(),
                        source: BudgetAssignmentSource::Transaction {
                            from_transaction_id,
                        },
                    },
                    BudgetAssignment {
                        id: Uuid::new_v4(),
                        amount: Decimal::ZERO,
                        date: NaiveDate::from_ymd_opt(2024, 11, 28).unwrap(),
                        source: BudgetAssignmentSource::OtherBudget {
                            from_budget_id: id2,
                            link_id: Uuid::new_v4()
                        },
                    },
                ],
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

            let mut updated = Budget::new(
                id1,
                "newName".into(),
                Some(target),
                user_id,
                budget
                    .assignments
                    .into_iter()
                    .chain([
                        BudgetAssignment {
                            id: Uuid::new_v4(),
                            amount: Decimal::ZERO,
                            date: NaiveDate::from_ymd_opt(2024, 11, 28).unwrap(),
                            source: BudgetAssignmentSource::Transaction {
                                from_transaction_id,
                            },
                        },
                        BudgetAssignment {
                            id: Uuid::new_v4(),
                            amount: Decimal::ZERO,
                            date: NaiveDate::from_ymd_opt(2024, 11, 28).unwrap(),
                            source: BudgetAssignmentSource::OtherBudget {
                                from_budget_id: id2,
                                link_id: Uuid::new_v4()
                            },
                        },
                    ])
                    .collect(),
            );

            update(&db_pool, updated.clone()).await.unwrap();

            let mut fetched = get_single(&db_pool, id1).await.unwrap();

            // sort assignments because we don't actually care about the order
            updated.assignments.sort_by(|a, b| a.id.cmp(&b.id));
            fetched.assignments.sort_by(|a, b| a.id.cmp(&b.id));

            assert_eq!(fetched, updated);
        }

        #[sqlx::test]
        pub async fn update_budget_remove_schedule_onetime_target(db_pool: MySqlPool) {
            test_init(&db_pool).await;
            let id = Uuid::new_v4();
            let user_id = *USER_ID;
            let schedule_id = Uuid::new_v4();

            let schedule = Schedule {
                id: schedule_id,
                period: SchedulePeriod::Weekly {
                    starting_on: NaiveDate::from_ymd_opt(2024, 10, 7).unwrap(),
                },
            };

            schedule::create(&db_pool, schedule.clone()).await.unwrap();

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
                assignments: vec![],
            };

            create(&db_pool, budget.clone()).await.unwrap();

            let updated_target = BudgetTarget::OneTime {
                target_amount: dec!(1.2),
            };

            let updated = Budget::new(id, "newName".into(), Some(updated_target), user_id, vec![]);

            update(&db_pool, updated.clone()).await.unwrap();

            let fetched = get_single(&db_pool, id).await.unwrap();

            assert_eq!(fetched, updated);
        }

        #[sqlx::test]
        pub async fn update_budget_remove_schedule_no_target(db_pool: MySqlPool) {
            test_init(&db_pool).await;
            let id = Uuid::new_v4();
            let user_id = *USER_ID;
            let schedule_id = Uuid::new_v4();

            let schedule = Schedule {
                id: schedule_id,
                period: SchedulePeriod::Weekly {
                    starting_on: NaiveDate::from_ymd_opt(2024, 10, 7).unwrap(),
                },
            };

            schedule::create(&db_pool, schedule.clone()).await.unwrap();

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
                assignments: vec![],
            };

            create(&db_pool, budget.clone()).await.unwrap();

            let updated = Budget::new(id, "newName".into(), None, user_id, vec![]);

            update(&db_pool, updated.clone()).await.unwrap();

            let fetched = get_single(&db_pool, id).await.unwrap();

            assert_eq!(fetched, updated);
        }

        #[sqlx::test]
        pub async fn update_budget_no_schedule(db_pool: MySqlPool) {
            test_init(&db_pool).await;
            let id = Uuid::new_v4();
            let user_id = *USER_ID;

            let budget = Budget {
                id,
                name: "name".into(),
                target: None,
                user_id,
                assignments: vec![],
            };

            create(&db_pool, budget.clone()).await.unwrap();

            let updated = Budget::new(id, "newName".into(), None, user_id, vec![]);

            update(&db_pool, updated.clone()).await.unwrap();

            let fetched = get_single(&db_pool, id).await.unwrap();

            assert_eq!(fetched, updated);
        }

        #[sqlx::test]
        pub async fn update_budget_schedule(db_pool: MySqlPool) {
            test_init(&db_pool).await;
            let id = Uuid::new_v4();
            let user_id = *USER_ID;
            let schedule_id = Uuid::new_v4();

            let schedule = Schedule {
                id: schedule_id,
                period: SchedulePeriod::Weekly {
                    starting_on: NaiveDate::from_ymd_opt(2024, 10, 7).unwrap(),
                },
            };

            schedule::create(&db_pool, schedule.clone()).await.unwrap();

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
                assignments: vec![],
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

            let updated = Budget::new(id, "newName".into(), Some(updated_target), user_id, vec![]);

            update(&db_pool, updated.clone()).await.unwrap();

            let fetched = get_single(&db_pool, id).await.unwrap();

            assert_eq!(fetched, updated);
        }

        #[sqlx::test]
        pub async fn delete_budget_test(db_pool: MySqlPool) {
            test_init(&db_pool).await;
            let id = Uuid::new_v4();
            let id2 = Uuid::new_v4();
            let user_id = *USER_ID;
            let schedule_id = Uuid::new_v4();
            
            create(&db_pool, Budget {
                id: id2,
                user_id,
                assignments: vec![],
                name: "name2".into(),
                target: None
            }).await.unwrap();

            let schedule = Schedule {
                id: schedule_id,
                period: SchedulePeriod::Weekly {
                    starting_on: NaiveDate::from_ymd_opt(2024, 11, 28).unwrap(),
                },
            };

            schedule::create(&db_pool, schedule.clone()).await.unwrap();

            let budget = Budget {
                id,
                name: "name".into(),
                target: Some(BudgetTarget::Repeating {
                    target_amount: Decimal::ZERO,
                    repeating_type: RepeatingTargetType::BuildUpTo,
                    schedule,
                }),
                user_id,
                assignments: vec![BudgetAssignment {
                    id: Uuid::new_v4(),
                    amount: Decimal::ZERO,
                    date: NaiveDate::from_ymd_opt(2024, 11, 28).unwrap(),
                    source: BudgetAssignmentSource::OtherBudget { from_budget_id: id2, link_id: Uuid::new_v4() }
                }],
            };

            create(&db_pool, budget).await.unwrap();

            delete(&db_pool, id).await.unwrap();

            let find_result = get(&db_pool, user_id).await.unwrap();

            assert_eq!(find_result.len(), 0);

            let assignments_count = sqlx::query_scalar!("SELECT COUNT(*) FROM BudgetAssignments")
                .fetch_one(&db_pool)
                .await
                .unwrap();

            assert_eq!(assignments_count, 0);
        }
    }
}
