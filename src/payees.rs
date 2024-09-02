use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Payee<'a>
{
    id: PayeeId,
    name: &'a str
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct PayeeId(pub Uuid);

pub async fn get_payees<'a>() -> Json<Box<[Payee<'a>]>>
{
    Json(Box::new([]))
}