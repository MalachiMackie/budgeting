use sqlx::MySqlPool;
use uuid::Uuid;

use crate::models::{CreatePayeeRequest, Payee};

use super::DbError;

struct PayeeModel {
    id: String,
    name: String,
    user_id: String,
}

impl TryFrom<PayeeModel> for Payee {
    type Error = anyhow::Error;

    fn try_from(value: PayeeModel) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.name,
            id: value.id.parse()?,
            user_id: value.user_id.parse()?,
        })
    }
}

pub async fn get_payees(db_pool: &MySqlPool, user_id: Uuid) -> Result<Box<[Payee]>, DbError> {
    let payees: Box<[Payee]> = sqlx::query_as!(
        PayeeModel,
        "SELECT id, name, user_id FROM Payees WHERE user_id = ?",
        user_id.as_simple()
    )
    .fetch_all(db_pool)
    .await?
    .into_iter()
    .map(|payee| payee.try_into().unwrap())
    .collect();

    Ok(payees)
}

pub async fn create_payee(
    db_pool: &MySqlPool,
    id: Uuid,
    request: CreatePayeeRequest,
) -> Result<(), DbError> {
    sqlx::query!(
        "INSERT INTO Payees(id, name, user_id) VALUE (?, ?, ?)",
        id.as_simple(),
        request.name,
        request.user_id.as_simple()
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn get_payee(db_pool: &MySqlPool, id: Uuid) -> Result<Payee, DbError> {
    sqlx::query_as!(
        PayeeModel,
        "SELECT id, name, user_id FROM Payees WHERE id = ?",
        id.as_simple()
    )
    .fetch_optional(db_pool)
    .await?
    .ok_or(DbError::NotFound)
    .map(|p| p.try_into().unwrap())
}
