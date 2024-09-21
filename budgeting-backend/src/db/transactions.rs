use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::transactions::{CreateTransactionRequest, Transaction};

use super::DbError;

struct TransactionModel {
    id: String,
    payee_id: String,
    date: NaiveDate,
    amount: Decimal,
    bank_account_id: String,
}

impl TryFrom<TransactionModel> for Transaction {
    type Error = anyhow::Error;

    fn try_from(value: TransactionModel) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.parse()?,
            date: value.date,
            payee_id: value.payee_id.parse()?,
            amount: value.amount,
            bank_account_id: value.bank_account_id.parse()?,
        })
    }
}

pub async fn create_transaction(db_pool: &MySqlPool, id: Uuid, bank_account_id: Uuid, request: CreateTransactionRequest) -> Result<(), DbError> {
    sqlx::query!(r"
            INSERT INTO Transactions (id, payee_id, date, amount, bank_account_id)
            VALUE (?, ?, ?, ?, ?)",
            id.as_simple(),
            request.payee_id.as_simple(),
            request.date,
            request.amount,
            bank_account_id.as_simple())
        .execute(db_pool)
        .await?;

    Ok(())
}

pub async fn get_transactions(db_pool: &MySqlPool, bank_account_id: Uuid) -> Result<Box<[Transaction]>, DbError> {
    let transactions = sqlx::query_as!(TransactionModel, "SELECT id, amount, date, payee_id, bank_account_id FROM Transactions WHERE bank_account_id = ?", bank_account_id.as_simple())
        .fetch_all(db_pool)
        .await?
        .into_iter()
        .map(|transaction| transaction.try_into().unwrap())
        .collect();

    Ok(transactions)
}