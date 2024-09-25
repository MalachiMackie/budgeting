use chrono::NaiveDate;
use derive_more::derive::Constructor;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Constructor, PartialEq, Debug, ToSchema)]
pub struct Payee {
    pub id: Uuid,
    pub name: String,
    pub user_id: Uuid,
}

#[derive(Serialize, Deserialize, Constructor, ToSchema)]
pub struct CreatePayeeRequest {
    pub name: String,
    pub user_id: Uuid,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct Transaction {
    pub id: Uuid,
    pub payee_id: Uuid,
    pub date: NaiveDate,
    #[schema(value_type = f32)]
    #[serde(with = "rust_decimal::serde::float")]
    pub amount: Decimal,
    pub bank_account_id: Uuid,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateTransactionRequest {
    pub payee_id: Uuid,
    #[schema(value_type = f32)]
    #[serde(with = "rust_decimal::serde::float")]
    pub amount: Decimal,
    pub date: NaiveDate,
}

#[derive(Deserialize, Serialize, Constructor, ToSchema, Debug, PartialEq)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
}

#[derive(Deserialize, Serialize, ToSchema, Constructor)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, ToSchema, Constructor)]
pub struct BankAccount {
    pub id: Uuid,
    pub name: String,
    #[schema(value_type = f32)]
    #[serde(with = "rust_decimal::serde::float")]
    pub initial_amount: Decimal,
    pub user_id: Uuid,
    #[schema(value_type = f32)]
    #[serde(with = "rust_decimal::serde::float")]
    pub balance: Decimal,
}

#[derive(Deserialize, Serialize, ToSchema, Constructor)]
pub struct CreateBankAccountRequest {
    pub name: String,
    #[schema(value_type = f32)]
    #[serde(with = "rust_decimal::serde::float")]
    pub initial_amount: Decimal,
    pub user_id: Uuid,
}
