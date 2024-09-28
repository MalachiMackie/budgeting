mod common;

use common::*;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use std::sync::OnceLock;

use budgeting_backend::{
    db,
    models::{
        Budget, BudgetTarget, CreateBudgetRequest, CreateBudgetTargetRequest, CreateScheduleRequest, CreateUserRequest, RepeatingTargetType, Schedule, SchedulePeriod, SchedulePeriodType
    },
};
use sqlx::MySqlPool;
use uuid::Uuid;

static USER_ID: OnceLock<Uuid> = OnceLock::new();

async fn test_init(db_pool: &MySqlPool) {
    let user_id = *USER_ID.get_or_init(|| Uuid::new_v4());

    db::users::create_user(
        db_pool,
        user_id,
        CreateUserRequest::new("name".into(), "email@email.com".into()),
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

    let budget = db::budgets::get_budgets(&db_pool, user_id).await;
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
            schedule: schedule,
        }),
    };

    assert_eq!(budget[0], expected_budget);
}

#[sqlx::test]
pub async fn test_get_budgets(db_pool: MySqlPool) {
    let test_server = integration_test_init(db_pool.clone());
    test_init(&db_pool).await;

    let schedule = Schedule {
        id: Uuid::new_v4(),
        period: SchedulePeriod::Custom { period: SchedulePeriodType::Fortnightly, every_x_periods: 1 }
    };
    db::schedule::create_schedule(&db_pool, schedule.clone()).await.unwrap();
    
    let user_id = *USER_ID.unwrap();
    
    let budget = Budget {
        id: Uuid::new_v4(),
        user_id,
        name: "name".into(),
        target: Some(BudgetTarget::Repeating { target_amount: Decimal::from_f32(1.1).unwrap(), repeating_type: RepeatingTargetType::BuildUpTo, schedule: schedule })
    };
    
    db::budgets::create_budget(&db_pool, budget.clone()).await.unwrap();
    
    let response = test_server.get(&format!("/api/budgets?user_id={user_id}")).await;
    
    response.assert_ok();
    response.assert_json(&vec![budget]);
}

