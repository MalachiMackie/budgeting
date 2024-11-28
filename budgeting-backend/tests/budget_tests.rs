mod common;

use chrono::NaiveDate;
use common::*;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use rust_decimal_macros::dec;
use std::sync::OnceLock;

use budgeting_backend::{
    db::{self, Error},
    models::{
        Budget, BudgetAssignment, BudgetTarget, CreateBudgetRequest, CreateBudgetTargetRequest,
        CreateScheduleRequest, RepeatingTargetType, Schedule, SchedulePeriod, SchedulePeriodType,
        UpdateBudgetRequest, UpdateBudgetTargetRequest, UpdateScheduleRequest, User,
    },
};
use sqlx::MySqlPool;
use uuid::Uuid;

static USER_ID: OnceLock<Uuid> = OnceLock::new();

async fn test_init(db_pool: &MySqlPool) {
    let user_id = *USER_ID.get_or_init(Uuid::new_v4);

    db::users::create(
        db_pool,
        User::new(user_id, "name".into(), "email@email.com".into(), None),
    )
    .await
    .unwrap();
}

#[sqlx::test]
pub async fn test_create_budget(db_pool: MySqlPool) {
    let test_server = integration_test_init(db_pool.clone());
    test_init(&db_pool).await;

    let user_id = *USER_ID.unwrap();

    let response = test_server
        .post("/api/budgets")
        .json(&CreateBudgetRequest::new(
            "name".into(),
            Some(CreateBudgetTargetRequest::Repeating {
                target_amount: Decimal::from_f32(1.1).unwrap(),
                repeating_type: RepeatingTargetType::RequireRepeating,
                schedule: CreateScheduleRequest {
                    period: SchedulePeriod::Custom {
                        period: SchedulePeriodType::Monthly,
                        every_x_periods: 2,
                    },
                },
            }),
            user_id,
        ))
        .await;

    response.assert_created();

    let budget_id: Uuid = response.json();

    let budget = db::budgets::get(&db_pool, user_id).await;
    assert!(budget.is_ok());

    let budget = budget.unwrap();
    assert!(budget.len() == 1);

    let Some(BudgetTarget::Repeating { schedule, .. }) = budget[0].clone().target else {
        panic!("expected budget target to be repeating");
    };

    let expected_budget = Budget {
        id: budget_id,
        name: "name".into(),
        user_id,
        target: Some(BudgetTarget::Repeating {
            target_amount: Decimal::from_f32(1.1).unwrap(),
            repeating_type: RepeatingTargetType::RequireRepeating,
            schedule,
        }),
        assignments: vec![],
    };

    assert_eq!(budget[0], expected_budget);
}

#[sqlx::test]
pub async fn test_get_budgets(db_pool: MySqlPool) {
    let test_server = integration_test_init(db_pool.clone());
    test_init(&db_pool).await;

    let schedule = Schedule {
        id: Uuid::new_v4(),
        period: SchedulePeriod::Custom {
            period: SchedulePeriodType::Fortnightly,
            every_x_periods: 1,
        },
    };
    db::schedule::create(&db_pool, schedule.clone())
        .await
        .unwrap();

    let user_id = *USER_ID.unwrap();

    let budget = Budget {
        id: Uuid::new_v4(),
        user_id,
        name: "name".into(),
        target: Some(BudgetTarget::Repeating {
            target_amount: Decimal::from_f32(1.1).unwrap(),
            repeating_type: RepeatingTargetType::BuildUpTo,
            schedule,
        }),
        assignments: vec![BudgetAssignment {
            id: Uuid::new_v4(),
            amount: dec!(10),
            date: NaiveDate::from_ymd_opt(2024, 11, 28).unwrap(),
        }],
    };

    db::budgets::create(&db_pool, budget.clone()).await.unwrap();

    let response = test_server
        .get(&format!("/api/budgets?user_id={user_id}"))
        .await;

    response.assert_ok();
    response.assert_json(&vec![budget]);
}

#[sqlx::test]
pub async fn delete_budget(db_pool: MySqlPool) {
    let test_server = integration_test_init(db_pool.clone());
    test_init(&db_pool).await;

    let user_id = *USER_ID.unwrap();
    let id = Uuid::new_v4();

    db::budgets::create(
        &db_pool,
        Budget::new(
            id,
            "name".into(),
            None,
            user_id,
            vec![BudgetAssignment {
                amount: dec!(10),
                id: Uuid::new_v4(),
                date: NaiveDate::from_ymd_opt(2024, 11, 28).unwrap(),
            }],
        ),
    )
    .await
    .unwrap();

    let response = test_server
        .delete(&format!("/api/budgets/{id}?user_id={user_id}"))
        .await;

    response.assert_ok();

    let find_response = db::budgets::get_single(&db_pool, id).await;

    assert!(matches!(dbg!(find_response), Err(Error::NotFound)));
}

#[sqlx::test]
pub async fn update_budget(db_pool: MySqlPool) {
    let test_server = integration_test_init(db_pool.clone());
    test_init(&db_pool).await;

    let user_id = *USER_ID.unwrap();
    let id = Uuid::new_v4();

    let assignments = vec![BudgetAssignment {
        id: Uuid::new_v4(),
        amount: dec!(10),
        date: NaiveDate::from_ymd_opt(2024, 11, 28).unwrap(),
    }];

    db::budgets::create(
        &db_pool,
        Budget::new(id, "name".into(), None, user_id, assignments.clone()),
    )
    .await
    .unwrap();

    let response = test_server
        .put(&format!("/api/budgets/{id}?user_id={user_id}"))
        .json(&UpdateBudgetRequest::new(
            "newName".into(),
            Some(UpdateBudgetTargetRequest::Repeating {
                target_amount: dec!(0),
                repeating_type: RepeatingTargetType::BuildUpTo,
                schedule: UpdateScheduleRequest {
                    period: SchedulePeriod::Weekly {
                        starting_on: NaiveDate::from_ymd_opt(2024, 10, 6).unwrap(),
                    },
                },
            }),
        ))
        .await;

    response.assert_ok();

    let mut find_response = db::budgets::get(&db_pool, user_id).await.unwrap()[0].clone();

    if let Some(BudgetTarget::Repeating { schedule, .. }) = &mut find_response.target {
        schedule.id = Uuid::nil();
    }

    let expected = Budget::new(
        id,
        "newName".into(),
        Some(BudgetTarget::Repeating {
            target_amount: dec!(0),
            repeating_type: RepeatingTargetType::BuildUpTo,
            schedule: Schedule {
                id: Uuid::nil(),
                period: SchedulePeriod::Weekly {
                    starting_on: NaiveDate::from_ymd_opt(2024, 10, 6).unwrap(),
                },
            },
        }),
        user_id,
        assignments,
    );

    assert_eq!(find_response, expected);
}
