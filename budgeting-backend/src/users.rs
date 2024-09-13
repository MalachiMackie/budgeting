use anyhow::anyhow;
use axum::{
    extract::{Path, State},
    Json,
};
use email_address::EmailAddress;
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

use crate::AppError;

#[derive(OpenApi)]
#[openapi(paths(get_users, get_user, create_user), components(schemas(User, CreateUserRequest)))]
pub struct UserApi;

const API_TAG: &str = "Users";

#[derive(Deserialize, Serialize, ToSchema)]
pub struct User {
    id: Uuid,
    name: String,
    email: String,
}

struct UserDbModel {
    id: String,
    name: String,
    email: String,
}

impl TryInto<User> for UserDbModel {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<User, Self::Error> {
        let id = Uuid::parse_str(&self.id)?;

        Ok(User {
            email: self.email,
            id: id,
            name: self.name,
        })
    }
}

#[derive(Deserialize, ToSchema)]
pub struct CreateUserRequest {
    name: String,
    email: String,
}

#[utoipa::path(
    post,
    path = "/api/users",
    responses(
        (status = OK, description = "Success", body = Uuid, content_type = "application/json")
    ),
    request_body = CreateUserRequest,
    tag = API_TAG
)]
pub async fn create_user(
    State(db_pool): State<MySqlPool>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<Uuid>, AppError> {
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

    sqlx::query!(
        "INSERT INTO Users(id, name, email) VALUE (?, ?, ?)",
        id.as_simple(),
        request.name,
        request.email
    )
    .execute(&db_pool)
    .await?;

    Ok(Json(id))
}

#[utoipa::path(
    get,
    path = "/api/users",
    responses(
        (status = OK, description = "Success", body = Box<[User]>, content_type = "application/json")
    ),
    tag = API_TAG
)]
pub async fn get_users(State(db_pool): State<MySqlPool>) -> Result<Json<Box<[User]>>, AppError> {
    let users: Vec<User> = sqlx::query_as!(UserDbModel, "SELECT id, email, name FROM Users")
        .fetch_all(&db_pool)
        .await?
        .into_iter()
        .map(|user| user.try_into().unwrap())
        .collect();

    Ok(Json(users.into_boxed_slice()))
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
    tag = API_TAG
)]
pub async fn get_user(
    State(db_pool): State<MySqlPool>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<User>, AppError> {
    let user = sqlx::query_as!(
        UserDbModel,
        "SELECT id, name, email FROM Users WHERE id = ?",
        user_id.as_simple()
    )
    .fetch_optional(&db_pool)
    .await?;

    let Some(user) = user else {
        return Err(AppError::NotFound(anyhow!(
            "User with id {user_id} was not found"
        )));
    };

    Ok(Json(user.try_into().unwrap()))
}
