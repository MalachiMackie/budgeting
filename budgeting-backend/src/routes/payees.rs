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
    db,
    models::{CreatePayeeRequest, Payee},
    AppError,
};

#[derive(OpenApi)]
#[openapi(paths(get, create), components(schemas(Payee, CreatePayeeRequest)))]
pub struct Api;

const API_TAG: &str = "Payees";

#[derive(Deserialize)]
pub struct GetPayeesQuery {
    user_id: Uuid,
}

#[utoipa::path(
    get,
    path = "/api/payees",
    responses(
        (status = OK, description = "Success", body = Box<[Payee]>, content_type = "application/json")
    ),
    params(
        ("user_id" = Uuid, Query,),
    ),
    tag = API_TAG
)]
pub async fn get(
    State(db_pool): State<MySqlPool>,
    Query(query): Query<GetPayeesQuery>,
) -> Result<Json<Box<[Payee]>>, AppError> {
    db::payees::get(&db_pool, query.user_id)
        .await
        .map(Json)
        .map_err(|e| e.to_app_error(anyhow!("Could not get payees")))
}

#[utoipa::path(
    post,
    path = "/api/payees",
    responses(
        (status = CREATED, description = "Success", body = Uuid, content_type = "application/json")
    ),
    request_body = CreatePayeeRequest,
    tag = API_TAG
)]
pub async fn create(
    State(db_pool): State<MySqlPool>,
    Json(request): Json<CreatePayeeRequest>,
) -> Result<(StatusCode, Json<Uuid>), AppError> {
    if request.user_id.is_nil() {
        return Err(AppError::BadRequest(anyhow!("User Id must be set")));
    }

    let id = Uuid::new_v4();

    db::payees::create(&db_pool, id, request)
        .await
        .map_err(|e| e.to_app_error(anyhow!("Could not create payee")))?;

    Ok((StatusCode::CREATED, Json(id)))
}
