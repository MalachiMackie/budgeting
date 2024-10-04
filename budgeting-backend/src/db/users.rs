use sqlx::MySqlPool;
use uuid::Uuid;

use crate::models::{CreateUserRequest, User};

use super::DbError;

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

pub async fn get_users(db_pool: &MySqlPool) -> Result<Box<[User]>, DbError> {
    let users = sqlx::query_as!(UserDbModel, "SELECT id, email, name FROM Users")
        .fetch_all(db_pool)
        .await?
        .into_iter()
        .map(|user| user.try_into().unwrap())
        .collect();

    Ok(users)
}

pub async fn get_user(db_pool: &MySqlPool, user_id: Uuid) -> Result<User, DbError> {
    sqlx::query_as!(
        UserDbModel,
        "SELECT id, name, email FROM Users WHERE id = ?",
        user_id.as_simple()
    )
    .fetch_optional(db_pool)
    .await?
    .map(|u| u.try_into().unwrap())
    .ok_or(DbError::NotFound)
}

pub async fn create_user(db_pool: &MySqlPool, id: Uuid, request: CreateUserRequest) -> Result<(), DbError> {
    sqlx::query!(
        "INSERT INTO Users(id, name, email) VALUE (?, ?, ?)",
        id.as_simple(),
        request.name,
        request.email
    )
    .execute(db_pool)
    .await?;

    Ok(())
}