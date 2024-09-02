use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::{payees::PayeeId, AppError};

#[derive(Deserialize, Serialize)]
pub struct Transaction
{
    id: TransactionId,
    payee_id: PayeeId,
    amount_dollars: i32,
    amount_cents: u8
}

#[derive(Deserialize, Serialize, Clone, Copy)]
pub struct TransactionId(pub Uuid);

impl TransactionId {
    pub fn new() -> Self
    {
        Self(Uuid::new_v4())
    }
}

struct TransactionModel
{
    id: String,
    payee_id: String,
    amount_cents: i32,
    amount_dollars: i32
}

impl TryFrom<TransactionModel> for Transaction {
    type Error = anyhow::Error;

    fn try_from(value: TransactionModel) -> Result<Self, Self::Error> {
        Ok(Self {
            id: TransactionId(value.id.parse()?),
            payee_id: PayeeId(value.payee_id.parse()?),
            amount_cents: value.amount_cents as u8,
            amount_dollars: value.amount_dollars
        })
    }
}

pub async fn get_transactions(State(db_pool): State<MySqlPool>) -> Result<Json<Box<[Transaction]>>, AppError>
{
    let transactions = sqlx::query_as!(TransactionModel, "SELECT id, amount_dollars, amount_cents, payee_id FROM Transactions")
        .fetch_all(&db_pool)
        .await?
        .into_iter()
        .map(|transaction| transaction.try_into())
        .collect::<Result<Vec<Transaction>, _>>()?;

    Ok(Json(transactions.into_boxed_slice()))
}
