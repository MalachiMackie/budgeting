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

impl TransactionId {
    pub fn new() -> Self
    {
        Self(Uuid::new_v4())
    }
}

pub async fn get_transactions() -> Json<Box<[Transaction]>>
{
    Json(Box::new([
        Transaction {
            id: TransactionId::new(),
            amount_cents: 0,
            amount_dollars: 100,
            payee_id: PayeeId::new()
        }
    ]))
}
