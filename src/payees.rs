use axum::{extract::State, Json, debug_handler};
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

#[debug_handler]
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
