use rust_decimal::Decimal;
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::models::{BankAccount, CreateBankAccountRequest};

use super::DbError;

struct BankAccountDbModel {
    id: String,
    name: String,
    initial_amount: Decimal,
    user_id: String,
    transaction_total: Option<Decimal>,
}

impl TryFrom<BankAccountDbModel> for BankAccount {
    type Error = anyhow::Error;

    fn try_from(value: BankAccountDbModel) -> Result<Self, Self::Error> {
        let id: Uuid = value.id.parse()?;
        let user_id: Uuid = value.user_id.parse()?;

        Ok(BankAccount {
            id,
            user_id,
            initial_amount: value.initial_amount,
            name: value.name,
            balance: value.initial_amount + value.transaction_total.unwrap_or(Decimal::ZERO),
        })
    }
}

pub async fn get_bank_accounts(
    db_pool: &MySqlPool,
    user_id: Uuid,
) -> Result<Box<[BankAccount]>, DbError> {
    let bank_accounts: Vec<BankAccount> = sqlx::query_as!(
        BankAccountDbModel,
        r"
         SELECT ba.id, ba.name, ba.initial_amount, ba.user_id, SUM(t.amount) as transaction_total
         FROM BankAccounts ba
         LEFT JOIN Transactions t ON ba.id = t.bank_account_id
         WHERE user_id = ?
         GROUP BY ba.id, ba.name, ba.initial_amount, ba.user_id",
        user_id.as_simple()
    )
    .fetch_all(db_pool)
    .await?
    .into_iter()
    .map(|bank_account| bank_account.try_into().unwrap())
    .collect();

    Ok(bank_accounts.into_boxed_slice())
}

pub async fn get_bank_account(
    db_pool: &MySqlPool,
    account_id: Uuid,
    user_id: Uuid,
) -> Result<BankAccount, DbError> {
    sqlx::query_as!(
        BankAccountDbModel,
        r"
        SELECT ba.id, ba.name, ba.initial_amount, ba.user_id, SUM(t.amount) as transaction_total
         FROM BankAccounts ba
         LEFT JOIN Transactions t ON ba.id = t.bank_account_id
         WHERE user_id = ?
         AND ba.id = ?
         GROUP BY ba.id, ba.name, ba.initial_amount, ba.user_id",
        user_id.as_simple(),
        account_id.as_simple()
    )
    .fetch_optional(db_pool)
    .await?
    .map(|account| account.try_into().unwrap())
    .ok_or(DbError::NotFound)
}

pub async fn create_bank_account(
    db_pool: &MySqlPool,
    id: Uuid,
    request: CreateBankAccountRequest,
) -> Result<(), DbError> {
    sqlx::query!(
        "INSERT INTO BankAccounts (id, name, user_id, initial_amount) VALUE(?, ?, ?, ?)",
        id.as_simple(),
        request.name,
        request.user_id.as_simple(),
        request.initial_amount
    )
    .execute(db_pool)
    .await?;

    Ok(())
}
