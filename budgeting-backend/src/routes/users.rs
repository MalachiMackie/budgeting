use anyhow::anyhow;
use axum::{
    extract::{Path, State},
    Json,
};
use email_address::EmailAddress;
use http::StatusCode;
use sqlx::MySqlPool;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{
    db,
    models::{CreateUserRequest, User},
    AppError,
};

#[derive(OpenApi)]
#[openapi(
    paths(get, get_single, create),
    components(schemas(User, CreateUserRequest))
)]
pub struct Api;

const API_TAG: &str = "Users";

#[utoipa::path(
    post,
    path = "/api/users",
    responses(
        (status = CREATED, description = "Success", body = Uuid, content_type = "application/json")
    ),
    request_body = CreateUserRequest,
    tag = API_TAG,
    operation_id = "createUser"
)]
pub async fn create(
    State(db_pool): State<MySqlPool>,
    Json(request): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<Uuid>), AppError> {
    if request.name.trim().is_empty() {
        return Err(AppError::BadRequest(anyhow!("User name must not be empty")));
    }

    if !EmailAddress::is_valid(&request.email) {
        return Err(AppError::BadRequest(anyhow!(
            "\"{}\" is not a valid email address",
            request.email
        )));
    }

    let id = Uuid::new_v4();

    db::users::create(&db_pool, id, request)
        .await
        .map_err(|e| e.to_app_error(anyhow!("Could not create user")))?;

    Ok((StatusCode::CREATED, Json(id)))
}

#[utoipa::path(
    get,
    path = "/api/users",
    responses(
        (status = OK, description = "Success", body = Box<[User]>, content_type = "application/json")
    ),
    tag = API_TAG,
    operation_id = "getUsers"
)]
pub async fn get(State(db_pool): State<MySqlPool>) -> Result<Json<Box<[User]>>, AppError> {
    db::users::get(&db_pool)
        .await
        .map(Json)
        .map_err(|e| e.to_app_error(anyhow!("Could not get users")))
}

#[utoipa::path(
    get,
    path = "/api/users/{userId}",
    responses(
        (status = OK, description = "Success", body = User, content_type = "application/json")
    ),
    params(
        ("userId" = Uuid, Path,)
    ),
    tag = API_TAG,
    operation_id = "getUser"
)]
pub async fn get_single(
    State(db_pool): State<MySqlPool>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<User>, AppError> {
    db::users::get_single(&db_pool, user_id)
        .await
        .map_err(|e| e.to_app_error(anyhow!("Could not get user")))
        .map(Json)
}
