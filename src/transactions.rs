use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::payees::PayeeId;

#[derive(Deserialize, Serialize)]
pub struct Transaction
{
    id: TransactionId,
    payee_id: PayeeId,
    amount_dollars: u32,
    amount_cents: u8
}

#[derive(Deserialize, Serialize, Clone, Copy)]
pub struct TransactionId(pub Uuid);

pub async fn get_transactions() -> Json<Box<[Transaction]>>
{
    Json(Box::new([]))
}