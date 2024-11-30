use anyhow::anyhow;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use http::StatusCode;
use serde::Deserialize;
use sqlx::MySqlPool;
use utoipa::{IntoParams, OpenApi};
use uuid::Uuid;

use crate::{
    db::{self},
    models::{
        Budget, BudgetAssignment, BudgetTarget, CreateBudgetRequest, CreateBudgetTargetRequest,
        CreateScheduleRequest, RepeatingTargetType, Schedule, SchedulePeriod, SchedulePeriodType,
        UpdateBudgetRequest, UpdateBudgetTargetRequest, UpdateScheduleRequest, BudgetAssignmentSource
    },
    AppError,
};
use crate::models::{GetBudgetResponse, TransferBudgetRequest};

#[derive(OpenApi)]
#[openapi(
    paths(get, create, update, delete, transfer_between),
    components(schemas(
        Budget,
        CreateBudgetRequest,
        BudgetTarget,
        UpdateBudgetRequest,
        Schedule,
        SchedulePeriod,
        RepeatingTargetType,
        SchedulePeriodType,
        CreateBudgetTargetRequest,
        UpdateBudgetTargetRequest,
        CreateScheduleRequest,
        UpdateScheduleRequest,
        TransferBudgetRequest,
        BudgetAssignment,
        BudgetAssignmentSource,
        GetBudgetResponse
    ))
)]
pub struct Api;

const API_TAG: &str = "Budgets";

#[derive(Deserialize, IntoParams)]
pub struct GetBudgetsQuery {
    user_id: Uuid,
}

#[utoipa::path(
    get,
    path = "/api/budgets",
    responses(
        (status = OK, description = "Success", body = Box<[GetBudgetResponse]>, content_type = "application/json")
    ),
    params(
        GetBudgetsQuery,
    ),
    tag = API_TAG,
    operation_id = "getBudgets"
)]
pub async fn get(
    State(db_pool): State<MySqlPool>,
    Query(query): Query<GetBudgetsQuery>,
) -> Result<Json<Box<[GetBudgetResponse]>>, AppError> {
    if query.user_id.is_nil() {
        return Err(AppError::BadRequest(anyhow!("user_id must be set")));
    }

    let budgets: Box<[GetBudgetResponse]> = Vec::from(db::budgets::get(&db_pool, query.user_id)
        .await
        .map_err(|e| e.to_app_error(anyhow!("Failed to get budgets")))?)
        .into_iter()
        .map(Into::into)
        .collect();

    Ok(Json(budgets))
}

#[utoipa::path(
    post,
    path = "/api/budgets",
    responses(
        (status = CREATED, description = "Success", body = Uuid, content_type = "application/json")
    ),
    request_body = CreateBudgetRequest,
    tag = API_TAG,
    operation_id = "createBudget"
)]
pub async fn create(
    State(db_pool): State<MySqlPool>,
    Json(request): Json<CreateBudgetRequest>,
) -> Result<(StatusCode, Json<Uuid>), AppError> {
    if request.user_id.is_nil() {
        return Err(AppError::BadRequest(anyhow!("user_id must be set")));
    }

    let name = request.name.trim();
    if name.is_empty() {
        return Err(AppError::BadRequest(anyhow!("Budget name cannot be empty")));
    }

    let user_result = db::users::get_single(&db_pool, request.user_id).await;
    match user_result {
        Ok(_) => (),
        Err(db::Error::NotFound) => {
            return Err(AppError::NotFound(anyhow!(
                "user with id {} was not found",
                request.user_id
            )))
        }
        Err(e) => return Err(e.to_app_error(anyhow!("Failed to create budget"))),
    };

    let budget_id = Uuid::new_v4();
    let schedule =
        if let Some(CreateBudgetTargetRequest::Repeating { schedule, .. }) = &request.target {
            let schedule_id = Uuid::new_v4();
            let schedule = Schedule {
                id: schedule_id,
                period: schedule.period.clone(),
            };

            db::schedule::create(&db_pool, schedule.clone())
                .await
                .map_err(|e| e.to_app_error(anyhow!("Failed to create budget")))?;

            Some(schedule)
        } else {
            None
        };

    let budget = Budget {
        id: budget_id,
        name: name.into(),
        target: request.target.map(|t| match t {
            CreateBudgetTargetRequest::OneTime { target_amount } => {
                BudgetTarget::OneTime { target_amount }
            }
            CreateBudgetTargetRequest::Repeating {
                target_amount,
                repeating_type,
                ..
            } if schedule.is_some() => BudgetTarget::Repeating {
                target_amount,
                repeating_type,
                schedule: schedule.expect("checked by arm guard"),
            },
            CreateBudgetTargetRequest::Repeating { .. } => {
                unreachable!("We create schedule above if target is repeating")
            }
        }),
        user_id: request.user_id,
        assignments: vec![],
    };

    db::budgets::create(&db_pool, budget)
        .await
        .map_err(|e| e.to_app_error(anyhow!("Failed to create budget")))?;

    Ok((StatusCode::CREATED, Json(budget_id)))
}

#[utoipa::path(
    put,
    path = "/api/budgets/{budget_id}",
    responses(
        (status = OK, description = "Success")
    ),
    request_body = UpdateBudgetRequest,
    tag = API_TAG,
    operation_id = "updateBudget"
)]
pub async fn update(
    State(db_pool): State<MySqlPool>,
    Path(budget_id): Path<Uuid>,
    Json(request): Json<UpdateBudgetRequest>,
) -> Result<(), AppError> {
    let mut existing_budget = db::budgets::get_single(&db_pool, budget_id)
        .await
        .map_err(|e| e.to_app_error(anyhow!("Failed to get budget")))?;

    let (schedule, schedule_id_to_delete) = match (&existing_budget.target, request.target.clone())
    {
        // update schedule
        (
            Some(BudgetTarget::Repeating {
                schedule: existing_schedule,
                ..
            }),
            Some(UpdateBudgetTargetRequest::Repeating {
                schedule: update_schedule_request,
                ..
            }),
        ) => {
            let updated_schedule = Schedule {
                id: existing_schedule.id,
                period: update_schedule_request.period,
            };
            db::schedule::update(&db_pool, updated_schedule.clone())
                .await
                .map_err(|e| e.to_app_error(anyhow!("Failed to update schedule")))?;

            (Some(updated_schedule), None)
        }
        // delete schedule
        (
            Some(BudgetTarget::Repeating { schedule, .. }),
            None | Some(UpdateBudgetTargetRequest::OneTime { .. }),
        ) => (None, Some(schedule.id)),
        // create schedule
        (
            None | Some(BudgetTarget::OneTime { .. }),
            Some(UpdateBudgetTargetRequest::Repeating { schedule, .. }),
        ) => {
            let new_schedule_id = Uuid::new_v4();

            let new_schedule = Schedule {
                id: new_schedule_id,
                period: schedule.period,
            };

            db::schedule::create(&db_pool, new_schedule.clone())
                .await
                .map_err(|e| e.to_app_error(anyhow!("Failed to create new schedule")))?;

            (Some(new_schedule), None)
        }
        _ => (None, None),
    };

    let target = request.target.map(|t| match t {
        UpdateBudgetTargetRequest::OneTime { target_amount } => {
            BudgetTarget::OneTime { target_amount }
        }
        UpdateBudgetTargetRequest::Repeating {
            target_amount,
            repeating_type,
            ..
        } => BudgetTarget::Repeating {
            target_amount,
            repeating_type,
            schedule: schedule.expect("Matched the same arm above"),
        },
    });

    existing_budget.name = request.name;
    existing_budget.target = target;

    db::budgets::update(&db_pool, existing_budget)
        .await
        .map_err(|e| e.to_app_error(anyhow!("Failed to update budget")))?;

    if let Some(schedule_id_to_delete) = schedule_id_to_delete {
        db::schedule::delete(&db_pool, schedule_id_to_delete)
            .await
            .map_err(|e| e.to_app_error(anyhow!("Failed to delete schedule")))?;
    }

    Ok(())
}

#[utoipa::path(
    delete,
    path = "/api/budgets/{budget_id}",
    responses(
        (status = OK, description = "Success")
    ),
    params(
        ("budget_id" = Uuid, Path,)
    ),
    tag = API_TAG,
    operation_id = "deleteBudget"
)]
pub async fn delete(
    State(db_pool): State<MySqlPool>,
    Path(budget_id): Path<Uuid>,
) -> Result<(), AppError> {
    let budget = db::budgets::get_single(&db_pool, budget_id)
        .await
        .map_err(|e| e.to_app_error(anyhow!("Failed to get budget to delete")))?;

    db::budgets::delete(&db_pool, budget_id)
        .await
        .map_err(|e| e.to_app_error(anyhow!("Failed to delete budget")))?;

    if let Some(BudgetTarget::Repeating { schedule, .. }) = budget.target {
        db::schedule::delete(&db_pool, schedule.id)
            .await
            .map_err(|e| e.to_app_error(anyhow!("Failed to delete schedule")))?;
    }

    Ok(())
}

#[utoipa::path(
    put,
    path = "/api/budgets/{budgetId}/transfer-to/{otherBudgetId}",
    responses(
        (status = OK, description = "Success")
    ),
    request_body = TransferBudgetRequest,
    params(
        ("budgetId" = Uuid, Path,),
        ("otherBudgetId" = Uuid, Path,),
    ),
    tag = API_TAG,
    operation_id = "transferBetweenBudgets"
)]
pub async fn transfer_between(
    State(db_pool): State<MySqlPool>,
    Path((from_budget_id, to_budget_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<TransferBudgetRequest>,
) -> Result<(), AppError> {
    let mut budgets = Vec::from(
        db::budgets::get_by_ids(&db_pool, &[from_budget_id, to_budget_id])
            .await
            .map_err(|e| e.to_app_error(anyhow!("Failed to get budget to assign to")))?,
    );

    if budgets.len() != 2 {
        return Err(AppError::NotFound(anyhow!(
            "Could not find either one or both budgets with ids {} and {}",
            from_budget_id,
            to_budget_id
        )));
    }

    let (mut from, mut to) = if budgets[0].id == from_budget_id {
        (budgets.remove(0), budgets.remove(0))
    } else {
        (budgets.remove(1), budgets.remove(0))
    };

    Budget::move_between_budgets(&mut from, &mut to, request.amount, request.date);

    db::budgets::update(&db_pool, from)
        .await
        .map_err(|e| e.to_app_error(anyhow!("Failed to update budget")))?;
    db::budgets::update(&db_pool, to)
        .await
        .map_err(|e| e.to_app_error(anyhow!("Failed to update budget")))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod update_budget_tests {
        use std::sync::{LazyLock, OnceLock};

        use chrono::NaiveDate;
        use rust_decimal_macros::dec;

        use crate::models::{BudgetAssignment, BudgetAssignmentSource, UpdateScheduleRequest, User};

        use super::*;

        static BUDGET_NO_TARGET: OnceLock<Budget> = OnceLock::new();
        static BUDGET_NO_TARGET_ID: LazyLock<Uuid> = LazyLock::new(Uuid::new_v4);
        static BUDGET_ONETIME_TARGET: OnceLock<Budget> = OnceLock::new();
        static BUDGET_ONETIME_TARGET_ID: LazyLock<Uuid> = LazyLock::new(Uuid::new_v4);
        static BUDGET_REPEATING_TARGET: OnceLock<Budget> = OnceLock::new();
        static BUDGET_REPEATING_TARGET_ID: LazyLock<Uuid> = LazyLock::new(Uuid::new_v4);
        static SCHEDULE_ID: LazyLock<Uuid> = LazyLock::new(Uuid::new_v4);
        static USER_ID: LazyLock<Uuid> = LazyLock::new(Uuid::new_v4);
        static ASSIGNMENT_ID1: LazyLock<Uuid> = LazyLock::new(Uuid::new_v4);
        static ASSIGNMENT_ID2: LazyLock<Uuid> = LazyLock::new(Uuid::new_v4);
        static ASSIGNMENT_ID3: LazyLock<Uuid> = LazyLock::new(Uuid::new_v4);
        static OTHER_BUDGET_ID: LazyLock<Uuid> = LazyLock::new(Uuid::new_v4);
        static ASSIGNMENT_LINK_ID: LazyLock<Uuid> = LazyLock::new(Uuid::new_v4);

        async fn test_init(db_pool: &MySqlPool) {
            let budget_no_target_id = *BUDGET_NO_TARGET_ID;
            let budget_onetime_target_id = *BUDGET_ONETIME_TARGET_ID;
            let budget_repeating_target_id = *BUDGET_REPEATING_TARGET_ID;
            let schedule_id = *SCHEDULE_ID;
            let assignment_id1 = *ASSIGNMENT_ID1;
            let assignment_id2 = *ASSIGNMENT_ID2;
            let assignment_id3 = *ASSIGNMENT_ID3;
            let other_budget_id = *OTHER_BUDGET_ID;
            let user_id = *USER_ID;
            let assignment_link_id = *ASSIGNMENT_LINK_ID;

            db::users::create(
                db_pool,
                User::new(user_id, "name".into(), "email@email.com".into(), None),
            ).await.unwrap();

            db::budgets::create(
                db_pool,
                Budget {
                    id: other_budget_id,
                    assignments: vec![],
                    name: "name".into(),
                    target: None,
                    user_id,
                },
            ).await.unwrap();

            let no_target = BUDGET_NO_TARGET
                .get_or_init(|| Budget {
                    id: budget_no_target_id,
                    name: "name".into(),
                    target: None,
                    user_id,
                    assignments: vec![BudgetAssignment {
                        id: assignment_id1,
                        amount: dec!(10),
                        date: NaiveDate::from_ymd_opt(2024, 11, 28).unwrap(),
                        source: BudgetAssignmentSource::OtherBudget
                        {
                            from_budget_id: other_budget_id,
                            link_id: assignment_link_id
                        }
                    }],
                })
                .clone();

            let onetime_target = BUDGET_ONETIME_TARGET
                .get_or_init(|| Budget {
                    id: budget_onetime_target_id,
                    name: "name".into(),
                    target: Some(BudgetTarget::OneTime {
                        target_amount: dec!(1.2),
                    }),
                    user_id,
                    assignments: vec![BudgetAssignment {
                        id: assignment_id2,
                        amount: dec!(10),
                        date: NaiveDate::from_ymd_opt(2024, 11, 28).unwrap(),
                        source: BudgetAssignmentSource::OtherBudget
                        {
                            from_budget_id: other_budget_id,
                            link_id: assignment_link_id
                        }
                    }],
                })
                .clone();

            let schedule = Schedule {
                id: schedule_id,
                period: SchedulePeriod::Weekly {
                    starting_on: NaiveDate::from_ymd_opt(2024, 10, 13).unwrap(),
                },
            };

            let repeating_target = BUDGET_REPEATING_TARGET
                .get_or_init(|| Budget {
                    id: budget_repeating_target_id,
                    name: "name".into(),
                    target: Some(BudgetTarget::Repeating {
                        target_amount: dec!(1.2),
                        repeating_type: RepeatingTargetType::BuildUpTo,
                        schedule: schedule.clone(),
                    }),
                    user_id,
                    assignments: vec![BudgetAssignment {
                        id: assignment_id3,
                        amount: dec!(10),
                        date: NaiveDate::from_ymd_opt(2024, 11, 28).unwrap(),
                        source: BudgetAssignmentSource::OtherBudget
                        {
                            from_budget_id: other_budget_id,
                            link_id: assignment_link_id
                        }
                    }],
                })
                .clone();

            db::budgets::create(db_pool, no_target).await.unwrap();
            db::budgets::create(db_pool, onetime_target).await.unwrap();

            db::schedule::create(db_pool, schedule).await.unwrap();

            db::budgets::create(db_pool, repeating_target)
                .await
                .unwrap();
        }

        #[sqlx::test]
        pub async fn no_schedule_to_no_schedule(db_pool: MySqlPool) {
            test_init(&db_pool).await;

            let budget = BUDGET_NO_TARGET.get().unwrap().clone();
            let user_id = *USER_ID;

            update(
                State(db_pool.clone()),
                Path(budget.id),
                Json(UpdateBudgetRequest {
                    name: "newName".into(),
                    target: None,
                }),
            )
            .await
            .unwrap();

            let expected = Budget {
                id: budget.id,
                name: "newName".into(),
                target: None,
                user_id,
                assignments: vec![BudgetAssignment {
                    id: *ASSIGNMENT_ID1,
                    amount: dec!(10),
                    date: NaiveDate::from_ymd_opt(2024, 11, 28).unwrap(),
                    source: BudgetAssignmentSource::OtherBudget {
                        from_budget_id: *OTHER_BUDGET_ID,
                        link_id: *ASSIGNMENT_LINK_ID
                    }
                }],
            };

            let fetched = db::budgets::get_single(&db_pool, budget.id).await.unwrap();

            assert_eq!(fetched, expected);
        }

        #[sqlx::test]
        pub async fn onetime_to_repeating(db_pool: MySqlPool) {
            test_init(&db_pool).await;

            let budget = BUDGET_ONETIME_TARGET.get().unwrap().clone();
            let user_id = *USER_ID;

            update(
                State(db_pool.clone()),
                Path(budget.id),
                Json(UpdateBudgetRequest {
                    name: "newName".into(),
                    target: Some(UpdateBudgetTargetRequest::Repeating {
                        target_amount: dec!(1.2),
                        repeating_type: RepeatingTargetType::BuildUpTo,
                        schedule: UpdateScheduleRequest {
                            period: SchedulePeriod::Weekly {
                                starting_on: NaiveDate::from_ymd_opt(2024, 10, 13).unwrap(),
                            },
                        },
                    }),
                }),
            )
            .await
            .unwrap();

            let fetched = db::budgets::get_single(&db_pool, budget.id).await.unwrap();

            let Some(BudgetTarget::Repeating { schedule, .. }) = &fetched.target else {
                panic!("Expected budget to be repeating");
            };

            let expected = Budget {
                id: budget.id,
                name: "newName".into(),
                target: Some(BudgetTarget::Repeating {
                    target_amount: dec!(1.2),
                    repeating_type: RepeatingTargetType::BuildUpTo,
                    schedule: schedule.clone(),
                }),
                user_id,
                assignments: vec![BudgetAssignment {
                    id: *ASSIGNMENT_ID2,
                    amount: dec!(10),
                    date: NaiveDate::from_ymd_opt(2024, 11, 28).unwrap(),
                    source: BudgetAssignmentSource::OtherBudget {
                        from_budget_id: *OTHER_BUDGET_ID,
                        link_id: *ASSIGNMENT_LINK_ID
                    }
                }],
            };

            assert_eq!(fetched, expected);
        }

        #[sqlx::test]
        pub async fn no_target_to_repeating(db_pool: MySqlPool) {
            test_init(&db_pool).await;

            let budget = BUDGET_NO_TARGET.get().unwrap().clone();
            let user_id = *USER_ID;

            update(
                State(db_pool.clone()),
                Path(budget.id),
                Json(UpdateBudgetRequest {
                    name: "newName".into(),
                    target: Some(UpdateBudgetTargetRequest::Repeating {
                        target_amount: dec!(1.2),
                        repeating_type: RepeatingTargetType::BuildUpTo,
                        schedule: UpdateScheduleRequest {
                            period: SchedulePeriod::Weekly {
                                starting_on: NaiveDate::from_ymd_opt(2024, 10, 13).unwrap(),
                            },
                        },
                    }),
                }),
            )
            .await
            .unwrap();

            let fetched = db::budgets::get_single(&db_pool, budget.id).await.unwrap();

            let Some(BudgetTarget::Repeating { schedule, .. }) = &fetched.target else {
                panic!("Expected budget to be repeating");
            };

            let expected = Budget {
                id: budget.id,
                name: "newName".into(),
                target: Some(BudgetTarget::Repeating {
                    target_amount: dec!(1.2),
                    repeating_type: RepeatingTargetType::BuildUpTo,
                    schedule: schedule.clone(),
                }),
                user_id,
                assignments: vec![BudgetAssignment {
                    id: *ASSIGNMENT_ID1,
                    amount: dec!(10),
                    date: NaiveDate::from_ymd_opt(2024, 11, 28).unwrap(),
                    source: BudgetAssignmentSource::OtherBudget {
                        from_budget_id: *OTHER_BUDGET_ID,
                        link_id: *ASSIGNMENT_LINK_ID
                    }
                }],
            };

            assert_eq!(fetched, expected);
        }

        #[sqlx::test]
        pub async fn repeating_to_no_target(db_pool: MySqlPool) {
            test_init(&db_pool).await;

            let budget = BUDGET_REPEATING_TARGET.get().unwrap().clone();
            let user_id = *USER_ID;

            update(
                State(db_pool.clone()),
                Path(budget.id),
                Json(UpdateBudgetRequest {
                    name: "newName".into(),
                    target: None,
                }),
            )
            .await
            .unwrap();

            let fetched = db::budgets::get_single(&db_pool, budget.id).await.unwrap();

            let expected = Budget {
                id: budget.id,
                name: "newName".into(),
                target: None,
                user_id,
                assignments: vec![BudgetAssignment {
                    id: *ASSIGNMENT_ID3,
                    amount: dec!(10),
                    date: NaiveDate::from_ymd_opt(2024, 11, 28).unwrap(),
                    source: BudgetAssignmentSource::OtherBudget {
                        from_budget_id: *OTHER_BUDGET_ID,
                        link_id: *ASSIGNMENT_LINK_ID
                    }
                }],
            };

            assert_eq!(fetched, expected);
        }

        #[sqlx::test]
        pub async fn repeating_to_one_time(db_pool: MySqlPool) {
            test_init(&db_pool).await;

            let budget = BUDGET_REPEATING_TARGET.get().unwrap().clone();
            let user_id = *USER_ID;

            update(
                State(db_pool.clone()),
                Path(budget.id),
                Json(UpdateBudgetRequest {
                    name: "newName".into(),
                    target: Some(UpdateBudgetTargetRequest::OneTime {
                        target_amount: dec!(1.2),
                    }),
                }),
            )
            .await
            .unwrap();

            let fetched = db::budgets::get_single(&db_pool, budget.id).await.unwrap();

            let expected = Budget {
                id: budget.id,
                name: "newName".into(),
                target: Some(BudgetTarget::OneTime {
                    target_amount: dec!(1.2),
                }),
                user_id,
                assignments: vec![BudgetAssignment {
                    id: *ASSIGNMENT_ID3,
                    amount: dec!(10),
                    date: NaiveDate::from_ymd_opt(2024, 11, 28).unwrap(),
                    source: BudgetAssignmentSource::OtherBudget {
                        from_budget_id: *OTHER_BUDGET_ID,
                        link_id: *ASSIGNMENT_LINK_ID
                    }
                }],
            };

            assert_eq!(fetched, expected);
        }
    }

    mod transfer_between_budget_tests {
        use std::sync::LazyLock;
        use chrono::NaiveDate;
        use rust_decimal::Decimal;
        use crate::models::{BudgetAssignment, BudgetAssignmentSource, User};

        use super::*;

        static USER_ID: LazyLock<Uuid> = LazyLock::new(Uuid::new_v4);
        static BUDGET1_ID: LazyLock<Uuid> = LazyLock::new(Uuid::new_v4);
        static BUDGET2_ID: LazyLock<Uuid> = LazyLock::new(Uuid::new_v4);

        async fn test_init(db_pool: &MySqlPool) {
            let user_id = *USER_ID;

            db::users::create(
                db_pool,
                User::new(user_id, "name".into(), "email@email.com".into(), None),
            )
            .await
            .unwrap();
            
            db::budgets::create(db_pool, Budget {
                id: *BUDGET1_ID,
                assignments: vec![],
                name: "name".into(),
                target: None,
                user_id
            }).await.unwrap();

            db::budgets::create(db_pool, Budget {
                id: *BUDGET2_ID,
                assignments: vec![],
                name: "name".into(),
                target: None,
                user_id
            }).await.unwrap();
        }

        #[sqlx::test]
        pub async fn existing_assignment_test(db_pool: MySqlPool) {
            test_init(&db_pool).await;

            let link_id = Uuid::new_v4();
            let mut budget1 = db::budgets::get_single(&db_pool, *BUDGET1_ID).await.unwrap();
            let mut budget2 = db::budgets::get_single(&db_pool, *BUDGET2_ID).await.unwrap();
            budget1.assignments.push(BudgetAssignment {
                id: Uuid::new_v4(),
                amount: Decimal::ZERO,
                date: NaiveDate::from_ymd_opt(2024, 11, 28).unwrap(),
                source: BudgetAssignmentSource::OtherBudget {
                    from_budget_id: *BUDGET2_ID,
                    link_id
                }
            });

            budget2.assignments.push(BudgetAssignment {
                id: Uuid::new_v4(),
                amount: Decimal::ZERO,
                date: NaiveDate::from_ymd_opt(2024, 11, 28).unwrap(),
                source: BudgetAssignmentSource::OtherBudget {
                    from_budget_id: *BUDGET1_ID,
                    link_id
                }
            });

            db::budgets::update(&db_pool, budget1).await.unwrap();
            db::budgets::update(&db_pool, budget2).await.unwrap();

            transfer_between(
                State(db_pool.clone()),
                Path((*BUDGET1_ID, *BUDGET2_ID)),
                Json(TransferBudgetRequest {
                    amount: Decimal::ZERO,
                    date: NaiveDate::from_ymd_opt(2024, 11, 30).unwrap(),
                }),
            )
            .await
            .unwrap();

            let mut fetched1 = db::budgets::get_single(&db_pool, *BUDGET1_ID).await.unwrap();
            let mut fetched2 = db::budgets::get_single(&db_pool, *BUDGET2_ID).await.unwrap();

            fetched1.assignments.sort_by_key(|a| a.date);
            fetched2.assignments.sort_by_key(|a| a.date);

            assert_eq!(fetched1.assignments.len(), 2);
            assert_eq!(fetched2.assignments.len(), 2);

            let BudgetAssignmentSource::OtherBudget {from_budget_id: from_budget_id1, link_id: link_id1} = fetched1.assignments[1].source else {
                panic!("expected assignment source to be OtherBudget");
            };
            let BudgetAssignmentSource::OtherBudget {from_budget_id: from_budget_id2, link_id: link_id2} = fetched2.assignments[1].source else {
                panic!("expected assignment source to be OtherBudget");
            };
            assert_eq!(link_id1, link_id2);
            assert_eq!(from_budget_id1, *BUDGET2_ID);
            assert_eq!(from_budget_id2, *BUDGET1_ID);
            assert_eq!(fetched1.assignments[1].date, NaiveDate::from_ymd_opt(2024, 11, 30).unwrap());
            assert_eq!(fetched1.assignments[1].amount, Decimal::ZERO);
            assert_eq!(fetched2.assignments[1].date, NaiveDate::from_ymd_opt(2024, 11, 30).unwrap());
            assert_eq!(fetched2.assignments[1].amount, Decimal::ZERO);
        }

        #[sqlx::test]
        pub async fn empty_assignment_test(db_pool: MySqlPool) {
            test_init(&db_pool).await;

            transfer_between(
                State(db_pool.clone()),
                Path((*BUDGET1_ID, *BUDGET2_ID)),
                Json(TransferBudgetRequest {
                    amount: Decimal::ZERO,
                    date: NaiveDate::from_ymd_opt(2024, 11, 30).unwrap(),
                }),
            )
            .await
            .unwrap();

            let mut fetched1 = db::budgets::get_single(&db_pool, *BUDGET1_ID).await.unwrap();
            let mut fetched2 = db::budgets::get_single(&db_pool, *BUDGET2_ID).await.unwrap();

            fetched1.assignments.sort_by_key(|a| a.date);
            fetched2.assignments.sort_by_key(|a| a.date);

            assert_eq!(fetched1.assignments.len(), 1);
            assert_eq!(fetched2.assignments.len(), 1);

            let BudgetAssignmentSource::OtherBudget {from_budget_id: from_budget_id1, link_id: link_id1} = fetched1.assignments[0].source else {
                panic!("expected assignment source to be OtherBudget");
            };
            let BudgetAssignmentSource::OtherBudget {from_budget_id: from_budget_id2, link_id: link_id2} = fetched2.assignments[0].source else {
                panic!("expected assignment source to be OtherBudget");
            };
            assert_eq!(link_id1, link_id2);
            assert_eq!(from_budget_id1, *BUDGET2_ID);
            assert_eq!(from_budget_id2, *BUDGET1_ID);
            assert_eq!(fetched1.assignments[0].date, NaiveDate::from_ymd_opt(2024, 11, 30).unwrap());
            assert_eq!(fetched1.assignments[0].amount, Decimal::ZERO);
            assert_eq!(fetched2.assignments[0].date, NaiveDate::from_ymd_opt(2024, 11, 30).unwrap());
            assert_eq!(fetched2.assignments[0].amount, Decimal::ZERO);
        }
    }

    mod delete_budget_tests {
        use chrono::NaiveDate;
        use db::Error;
        use rust_decimal_macros::dec;

        use crate::models::User;

        use super::*;

        #[sqlx::test]
        pub async fn delete_budget_should_delete_schedule(db_pool: MySqlPool) {
            let user_id = Uuid::new_v4();

            db::users::create(
                &db_pool,
                User::new(user_id, "name".into(), "email@email.com".into(), None),
            )
            .await
            .unwrap();

            let schedule_id = Uuid::new_v4();

            let schedule = Schedule {
                id: schedule_id,
                period: SchedulePeriod::Weekly {
                    starting_on: NaiveDate::from_ymd_opt(2024, 10, 13).unwrap(),
                },
            };

            db::schedule::create(&db_pool, schedule.clone())
                .await
                .unwrap();

            let budget_id = Uuid::new_v4();

            let budget = Budget {
                id: budget_id,
                name: "name".into(),
                target: Some(BudgetTarget::Repeating {
                    target_amount: dec!(1),
                    repeating_type: RepeatingTargetType::BuildUpTo,
                    schedule: schedule.clone(),
                }),
                user_id,
                assignments: vec![],
            };

            db::budgets::create(&db_pool, budget).await.unwrap();

            delete(State(db_pool.clone()), Path(budget_id))
                .await
                .unwrap();

            let fetch_result = db::budgets::get_single(&db_pool, budget_id).await;

            assert!(matches!(fetch_result, Err(Error::NotFound)));

            let fetch_schedule_result = db::schedule::get_single(&db_pool, schedule_id).await;
            assert!(matches!(fetch_schedule_result, Err(Error::NotFound)));
        }

        #[sqlx::test]
        pub async fn delete_budget_should_succeed_when_no_schedule(db_pool: MySqlPool) {
            let user_id = Uuid::new_v4();

            db::users::create(
                &db_pool,
                User::new(user_id, "name".into(), "email@email.com".into(), None),
            )
            .await
            .unwrap();

            let budget_id = Uuid::new_v4();

            let budget = Budget {
                id: budget_id,
                name: "name".into(),
                target: None,
                user_id,
                assignments: vec![],
            };

            db::budgets::create(&db_pool, budget).await.unwrap();

            delete(State(db_pool.clone()), Path(budget_id))
                .await
                .unwrap();

            let fetch_result = db::budgets::get_single(&db_pool, budget_id).await;

            assert!(matches!(fetch_result, Err(Error::NotFound)));
        }
    }
}
