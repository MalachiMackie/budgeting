use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::{prelude::FromRow, MySql, MySqlPool};
use uuid::Uuid;

use crate::models::{CreateTransactionRequest, Transaction};

use super::Error;

#[derive(FromRow)]
struct TransactionModel {
    id: uuid::fmt::Simple,
    payee_id: uuid::fmt::Simple,
    date: NaiveDate,
    amount: Decimal,
    bank_account_id: uuid::fmt::Simple,
    budget_id: uuid::fmt::Simple,
}

impl From<TransactionModel> for Transaction {
    fn from(value: TransactionModel) -> Self {
        Self {
            id: value.id.into_uuid(),
            date: value.date,
            payee_id: value.payee_id.into_uuid(),
            amount: value.amount,
            bank_account_id: value.bank_account_id.into(),
            budget_id: value.budget_id.into(),
        }
    }
}

pub async fn create(
    db_pool: &MySqlPool,
    id: Uuid,
    bank_account_id: Uuid,
    request: CreateTransactionRequest,
) -> Result<(), Error> {
    sqlx::query!(
        r"
            INSERT INTO Transactions (id, payee_id, date, amount, bank_account_id, budget_id)
            VALUE (?, ?, ?, ?, ?, ?)",
        id.as_simple(),
        request.payee_id.as_simple(),
        request.date,
        request.amount,
        bank_account_id.as_simple(),
        request.budget_id.as_simple()
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn update(db_pool: &MySqlPool, transaction: Transaction) -> Result<(), Error> {
    sqlx::query!(
        "UPDATE Transactions
    SET amount = ?,
    date = ?,
    payee_id = ?,
    budget_id = ?
    WHERE id = ?",
        transaction.amount,
        transaction.date,
        transaction.payee_id.as_simple(),
        transaction.budget_id.as_simple(),
        transaction.id.as_simple()
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn get(db_pool: &MySqlPool, bank_account_id: Uuid) -> Result<Box<[Transaction]>, Error> {
    let transactions = sqlx::query_as::<MySql, TransactionModel>(
        "SELECT id, amount, date, payee_id, bank_account_id, budget_id FROM Transactions WHERE bank_account_id = ?").bind(bank_account_id.simple())
        .fetch_all(db_pool)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();

    Ok(transactions)
}

pub async fn get_single(db_pool: &MySqlPool, transaction_id: Uuid) -> Result<Transaction, Error> {
    Ok(sqlx::query_as::<MySql, TransactionModel>(
        "SELECT id, amount, date, payee_id, bank_account_id, budget_id FROM Transactions WHERE id = ?").bind(transaction_id.simple())
        .fetch_optional(db_pool)
        .await?
        .ok_or(Error::NotFound)?
        .into())
}

pub async fn delete(db_pool: &MySqlPool, transaction_id: Uuid) -> Result<(), Error> {
    sqlx::query!(
        "DELETE FROM Transactions WHERE id = ?",
        transaction_id.as_simple()
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;

    use rust_decimal::prelude::FromPrimitive;
    use rust_decimal_macros::dec;

    use crate::{
        db,
        extensions::decimal::DecimalExt,
        models::{Budget, CreateBankAccountRequest, CreatePayeeRequest, User},
    };

    use super::*;

    static USER_ID: OnceLock<Uuid> = OnceLock::new();
    static BANK_ACCOUNT_ID: OnceLock<Uuid> = OnceLock::new();
    static BUDGET_ID: OnceLock<Uuid> = OnceLock::new();
    static PAYEE_ID: OnceLock<Uuid> = OnceLock::new();

    async fn test_init(db_pool: &MySqlPool) {
        let user_id = *USER_ID.get_or_init(Uuid::new_v4);
        let bank_account_id = *BANK_ACCOUNT_ID.get_or_init(Uuid::new_v4);
        let budget_id = *BUDGET_ID.get_or_init(Uuid::new_v4);
        let payee_id = *PAYEE_ID.get_or_init(Uuid::new_v4);

        db::users::create(
            db_pool,
            User::new(user_id, "User".into(), "email@email.com".into(), None),
        )
        .await
        .unwrap();

        db::bank_accounts::create(
            db_pool,
            bank_account_id,
            CreateBankAccountRequest::new("BankAccount".into(), Decimal::default(), user_id),
        )
        .await
        .unwrap();

        db::budgets::create(
            db_pool,
            Budget::new(budget_id, "Budget".into(), None, user_id, vec![]),
        )
        .await
        .unwrap();

        db::payees::create(
            db_pool,
            payee_id,
            CreatePayeeRequest::new("Payee".into(), user_id),
        )
        .await
        .unwrap();
    }

    #[sqlx::test]
    pub async fn create_and_get_test(db_pool: MySqlPool) {
        test_init(&db_pool).await;

        let transaction_id = Uuid::new_v4();
        let bank_account_id = *BANK_ACCOUNT_ID.get().unwrap();
        let payee_id = *PAYEE_ID.get().unwrap();
        let budget_id = *BUDGET_ID.get().unwrap();

        let result = create(
            &db_pool,
            transaction_id,
            bank_account_id,
            CreateTransactionRequest::new(
                payee_id,
                Decimal::from_f32(1.2).unwrap(),
                NaiveDate::from_ymd_opt(2024, 10, 5).unwrap(),
                budget_id,
            ),
        )
        .await;

        assert!(result.is_ok());

        let transactions = get(&db_pool, bank_account_id).await;

        assert!(transactions.is_ok());

        assert!(transactions.as_ref().unwrap().len() == 1);

        let mut transaction = transactions.unwrap()[0].clone();
        let amount = transaction.amount;
        transaction.amount = dec!(0);

        assert_eq!(
            transaction,
            Transaction::new(
                transaction_id,
                payee_id,
                NaiveDate::from_ymd_opt(2024, 10, 5).unwrap(),
                Decimal::ZERO,
                bank_account_id,
                budget_id
            )
        );
        assert!(amount.approximately_eq(dec!(1.2), dec!(0.001)));
    }

    #[sqlx::test]
    pub async fn update_test(db_pool: MySqlPool) {
        test_init(&db_pool).await;
        let transaction_id = Uuid::new_v4();

        let user_id = *USER_ID.get().unwrap();
        let bank_account_id = *BANK_ACCOUNT_ID.get().unwrap();
        let payee_id_1 = *PAYEE_ID.get().unwrap();
        let budget_id_1 = *BUDGET_ID.get().unwrap();

        let payee_id_2 = Uuid::new_v4();
        let budget_id_2 = Uuid::new_v4();

        db::payees::create(
            &db_pool,
            payee_id_2,
            CreatePayeeRequest::new("Payee2".into(), user_id),
        )
        .await
        .unwrap();

        db::budgets::create(
            &db_pool,
            Budget::new(budget_id_2, "Budget2".into(), None, user_id, vec![]),
        )
        .await
        .unwrap();

        create(
            &db_pool,
            transaction_id,
            bank_account_id,
            CreateTransactionRequest::new(
                payee_id_1,
                Decimal::from_f32(1.2).unwrap(),
                NaiveDate::from_ymd_opt(2024, 10, 5).unwrap(),
                budget_id_1,
            ),
        )
        .await
        .unwrap();

        let mut updated = Transaction::new(
            transaction_id,
            payee_id_2,
            NaiveDate::from_ymd_opt(2024, 10, 4).unwrap(),
            Decimal::from_f32(-1.2).unwrap(),
            bank_account_id,
            budget_id_2,
        );

        let result = update(&db_pool, updated.clone()).await;

        assert!(result.is_ok());

        let found_transactions = get(&db_pool, bank_account_id).await.unwrap();

        assert!(found_transactions.len() == 1);
        let mut found_transaction = found_transactions[0].clone();

        let found_amount = found_transaction.amount;
        found_transaction.amount = Decimal::ZERO;

        let updated_amount = updated.amount;
        updated.amount = Decimal::ZERO;

        assert_eq!(found_transaction, updated);
        assert!(found_amount.approximately_eq(updated_amount, dec!(0.001)));
    }

    #[sqlx::test]
    pub async fn delete_test(db_pool: MySqlPool) {
        test_init(&db_pool).await;

        let transaction_id = Uuid::new_v4();
        let bank_account_id = *BANK_ACCOUNT_ID.get().unwrap();
        let payee_id = *PAYEE_ID.get().unwrap();
        let budget_id = *BUDGET_ID.get().unwrap();

        create(
            &db_pool,
            transaction_id,
            bank_account_id,
            CreateTransactionRequest::new(
                payee_id,
                Decimal::from_f32(1.2).unwrap(),
                NaiveDate::from_ymd_opt(2024, 10, 5).unwrap(),
                budget_id,
            ),
        )
        .await
        .unwrap();

        let result = delete(&db_pool, transaction_id).await;

        assert!(result.is_ok());

        let find_result = get_single(&db_pool, transaction_id).await;

        assert!(matches!(find_result, Err(Error::NotFound)));
    }
}
