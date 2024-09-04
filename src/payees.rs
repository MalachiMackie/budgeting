use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use sqlx::{query_as, MySqlPool};
use uuid::Uuid;

use crate::AppError;

#[derive(Serialize, Deserialize)]
pub struct Payee
{
    id: PayeeId,
    name: String
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct PayeeId(pub Uuid);

impl PayeeId {
    pub fn new() -> Self {
        PayeeId(Uuid::new_v4())
    }
}

struct PayeeModel {
    id: String,
    name: String
}

impl TryFrom<PayeeModel> for Payee {
    type Error = anyhow::Error;

    fn try_from(value: PayeeModel) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.name,
            id: PayeeId(value.id.parse()?)
        })
    }
}

pub async fn get_payees(State(connection_pool): State<MySqlPool>) -> Result<Json<Box<[Payee]>>, AppError>
{
    let payees = query_as!(PayeeModel, "SELECT id, name FROM Payees")
        .fetch_all(&connection_pool)
        .await?
        .into_iter()
        .map(|payee| payee.try_into())
        .collect::<Result<Vec<Payee>, _>>()?;

    Ok(Json(payees.into_boxed_slice()))
}

#[derive(Serialize, Deserialize)]
pub struct CreatePayeeRequest
{
    name: String
}

pub async fn create_payee(
    State(connection_pool): State<MySqlPool>,
    Json(request): Json<CreatePayeeRequest>)
    -> Result<Json<Uuid>, AppError>
    where 
    {
        let id = PayeeId::new();

        sqlx::query!("INSERT INTO Payees(id, name) VALUE (?, ?)", id.0.as_simple(), request.name)
            .execute(&connection_pool)
            .await?;

        Ok(Json(id.0))
}

pub async fn get_payee(id: PayeeId, db_pool: &MySqlPool) -> Result<Option<Payee>, anyhow::Error> {
    let payee = sqlx::query_as!(PayeeModel, "SELECT id, name FROM Payees WHERE id = ?", id.0.as_simple())
        .fetch_optional(db_pool)
        .await?;

    let Some(payee): Option<Payee> = payee.map(|p| p.try_into().unwrap()) else {
        return Ok(None);
    };

    Ok(Some(payee))
}