use anyhow::anyhow;
use axum::{
    extract::{Query, State},
    Json,
};
use http::StatusCode;
use serde::Deserialize;
use sqlx::MySqlPool;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{
    db::{self, DbError},
    models::{Budget, BudgetTarget, CreateBudgetRequest, CreateBudgetTargetRequest, RepeatingTargetType, Schedule, SchedulePeriod, SchedulePeriodType},
    AppError,
};

#[derive(OpenApi)]
#[openapi(
    paths(get_budgets, create_budget),
    components(schemas(Budget, CreateBudgetRequest, BudgetTarget, CreateBudgetRequest, Schedule, SchedulePeriod, RepeatingTargetType, SchedulePeriodType))
)]
pub struct BudgetsApi;

const API_TAG: &str = "Budgets";

#[derive(Deserialize)]
pub struct GetBudgetsQuery {
    user_id: Uuid,
}

#[utoipa::path(
    get,
    path = "/api/budgets",
    responses(
        (status = OK, description = "Success", body = Box<[Budget]>, content_type = "application/json")
    ),
    params(
        ("user_id" = Uuid, Query,)
    ),
    tag = API_TAG
)]
pub async fn get_budgets(
    State(db_pool): State<MySqlPool>,
    Query(query): Query<GetBudgetsQuery>,
) -> Result<Json<Box<[Budget]>>, AppError> {
    if query.user_id.is_nil() {
        return Err(AppError::BadRequest(anyhow!("user_id must be set")));
    }

    db::budgets::get_budgets(&db_pool, query.user_id)
        .await
        .map(Json)
        .map_err(|e| e.to_app_error(anyhow!("Failed to get budgets")))
}

#[utoipa::path(
    post,
    path = "/api/budgets",
    responses(
        (status = CREATED, description = "Success", body = Uuid, content_type = "application/json")
    ),
    request_body = CreateBudgetRequest,
    tag = API_TAG
)]
pub async fn create_budget(
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

    let user_result = db::users::get_user(&db_pool, request.user_id).await;
    match user_result {
        Ok(_) => (),
        Err(DbError::NotFound) => {
            return Err(AppError::NotFound(anyhow!(
                "user with id {} was not found",
                request.user_id
            )))
        }
        Err(e) => return Err(e.to_app_error(anyhow!("Failed to create budget"))),
    };

    let budget_id = Uuid::new_v4();
    let schedule = if let Some(CreateBudgetTargetRequest::Repeating { schedule, .. }) = &request.target {
        let schedule_id = Uuid::new_v4();
        let schedule = Schedule {
            id: schedule_id,
            period: schedule.period.clone(),
        };

        db::schedule::create_schedule(&db_pool, schedule.clone())
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
            _ => unreachable!("We create schedule above if target is repeating"),
        }),
        user_id: request.user_id,
    };

    db::budgets::create_budget(&db_pool, budget)
        .await
        .map_err(|e| e.to_app_error(anyhow!("Failed to create budget")))?;

    Ok((StatusCode::CREATED, Json(budget_id)))
}
