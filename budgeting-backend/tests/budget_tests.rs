mod common;

use chrono::NaiveDate;
use common::*;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use rust_decimal_macros::dec;
use std::sync::LazyLock;

use budgeting_backend::{
    db::{self, Error},
    models::{
        Budget, BudgetAssignment, BudgetTarget, CreateBudgetRequest, CreateBudgetTargetRequest,
        CreateScheduleRequest, RepeatingTargetType, Schedule, SchedulePeriod, SchedulePeriodType,
        UpdateBudgetRequest, UpdateBudgetTargetRequest, UpdateScheduleRequest, User,
        BudgetAssignmentSource, TransferBudgetRequest
    },
};
use sqlx::MySqlPool;
use uuid::Uuid;

static USER_ID: LazyLock<Uuid> = LazyLock::new(Uuid::new_v4);
static OTHER_BUDGET_ID: LazyLock<Uuid> = LazyLock::new(Uuid::new_v4);

async fn test_init(db_pool: &MySqlPool) {
    let user_id = *USER_ID;

    db::users::create(
        db_pool,
        User::new(user_id, "name".into(), "email@email.com".into(), None),
    )
    .await
    .unwrap();

    db::budgets::create(&db_pool, Budget {
        id: *OTHER_BUDGET_ID,
        assignments: vec![],
        name: "name".into(),
        target: None,
        user_id
    }).await.unwrap();
}

#[sqlx::test]
pub async fn test_create_budget(db_pool: MySqlPool) {
    let test_server = integration_test_init(db_pool.clone());
    test_init(&db_pool).await;

    let user_id = *USER_ID;

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
    assert_eq!(budget.len(), 1);

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

    let user_id = *USER_ID;

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
            source: BudgetAssignmentSource::OtherBudget {
                from_budget_id: *OTHER_BUDGET_ID,
                link_id: Uuid::new_v4()
            }
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

    let user_id = *USER_ID;
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
                source: BudgetAssignmentSource::OtherBudget {
                    from_budget_id: *OTHER_BUDGET_ID,
                    link_id: Uuid::new_v4()
                }
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

    let user_id = *USER_ID;
    let id = Uuid::new_v4();

    let assignments = vec![BudgetAssignment {
        id: Uuid::new_v4(),
        amount: dec!(10),
        date: NaiveDate::from_ymd_opt(2024, 11, 28).unwrap(),
        source: BudgetAssignmentSource::OtherBudget {
            from_budget_id: *OTHER_BUDGET_ID,
            link_id: Uuid::new_v4()
        }
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

#[sqlx::test]
pub async fn transfer_between_budgets(db_pool: MySqlPool) {
    let test_server = integration_test_init(db_pool.clone());
    test_init(&db_pool).await;

    let budget = Budget {
        id: Uuid::new_v4(),
        name: "name".into(),
        user_id: *USER_ID,
        target: None,
        assignments: vec![]
    };

    db::budgets::create(&db_pool, budget.clone()).await.unwrap();

    let response = test_server.put(&format!("api/budgets/{}/transfer-to/{}", budget.id, *OTHER_BUDGET_ID))
        .json(&TransferBudgetRequest {
            amount: dec!(1),
            date: NaiveDate::from_ymd_opt(2024, 11, 30).unwrap(),
        })
        .await;

    response.assert_ok();

    let mut fetched_1 = db::budgets::get_single(&db_pool, budget.id).await.unwrap();
    let mut fetched_2 = db::budgets::get_single(&db_pool, *OTHER_BUDGET_ID).await.unwrap();

    assert_eq!(fetched_1.assignments.len(), 1);
    assert_eq!(fetched_2.assignments.len(), 1);

    let BudgetAssignmentSource::OtherBudget {from_budget_id: from_budget_id1, link_id: link_id1} = &fetched_1.assignments[0].source else {
        panic!("assignment source must be OtherBudget")
    };
    let BudgetAssignmentSource::OtherBudget {from_budget_id: from_budget_id2, link_id: link_id2} = &fetched_2.assignments[0].source else {
        panic!("assignment source must be OtherBudget")
    };

    assert_eq!(link_id1, link_id2);
    assert_eq!(from_budget_id1, &fetched_2.id);
    assert_eq!(from_budget_id2, &fetched_1.id);

    let link_id = *link_id1;

    for assignment in &mut fetched_1.assignments {
        assignment.id = Uuid::nil();
    }
    for assignment in &mut fetched_2.assignments {
        assignment.id = Uuid::nil();
    }

    assert_eq!(fetched_1, Budget {
        assignments: vec![BudgetAssignment {
            id: Uuid::nil(),
            date: NaiveDate::from_ymd_opt(2024, 11, 30).unwrap(),
            amount: dec!(-1),
            source: BudgetAssignmentSource::OtherBudget {
                from_budget_id: fetched_2.id,
                link_id
            }
        }],
        ..budget.clone()
    });
    assert_eq!(fetched_1, Budget {
        assignments: vec![BudgetAssignment {
            id: Uuid::nil(),
            date: NaiveDate::from_ymd_opt(2024, 11, 30).unwrap(),
            amount: dec!(1),
            source: BudgetAssignmentSource::OtherBudget {
                from_budget_id: fetched_1.id,
                link_id
            }
        }],
        ..budget.clone()
    })
}
