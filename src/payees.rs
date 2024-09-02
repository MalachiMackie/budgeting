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

impl PayeeId {
    pub fn new() -> Self {
        PayeeId(Uuid::new_v4())
    }
}

pub async fn get_payees<'a>() -> Json<Box<[Payee<'a>]>>
{
    Json(Box::new([
        Payee
        {
            id: PayeeId::new(),
            name: "hello"
        }
    ]))
}
