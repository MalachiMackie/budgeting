mod common;

use std::sync::OnceLock;

use budgeting_backend::{
    db::{self, Error},
    models::{
        Budget, CreateBankAccountRequest, CreatePayeeRequest, CreateTransactionRequest,
        CreateUserRequest, Transaction, UpdateTransactionRequest,
    },
};
use chrono::NaiveDate;
use common::*;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use rust_decimal_macros::dec;
use sqlx::MySqlPool;
use uuid::Uuid;

static USER_ID: OnceLock<Uuid> = OnceLock::new();
static BANK_ACCOUNT_ID: OnceLock<Uuid> = OnceLock::new();
static PAYEE_ID: OnceLock<Uuid> = OnceLock::new();
static BUDGET_ID: OnceLock<Uuid> = OnceLock::new();

async fn test_init(db_pool: &MySqlPool) {
    let user_id = *USER_ID.get_or_init(Uuid::new_v4);
    let bank_account_id = *BANK_ACCOUNT_ID.get_or_init(Uuid::new_v4);
    let payee_id = *PAYEE_ID.get_or_init(Uuid::new_v4);
    let budget_id = *BUDGET_ID.get_or_init(Uuid::new_v4);

    db::users::create(
        db_pool,
        user_id,
        CreateUserRequest::new("name".into(), "email@email.com".into()),
    )
    .await
    .unwrap();

    db::payees::create(
        db_pool,
        payee_id,
        CreatePayeeRequest::new("name".into(), user_id),
    )
    .await
    .unwrap();

    db::bank_accounts::create(
        db_pool,
        bank_account_id,
        CreateBankAccountRequest::new("name".into(), Decimal::from_i32(0).unwrap(), user_id),
    )
    .await
    .unwrap();

    db::budgets::create(
        db_pool,
        Budget::new(budget_id, "Budget".into(), None, user_id),
    )
    .await
    .unwrap();
}

#[sqlx::test]
pub async fn create_transaction(db_pool: MySqlPool) {
    let test_server = integration_test_init(db_pool.clone());
    test_init(&db_pool).await;

    let bank_account_id = *BANK_ACCOUNT_ID.unwrap();
    let payee_id = *PAYEE_ID.unwrap();
    let budget_id = *BUDGET_ID.unwrap();

    let response = test_server
        .post(&format!(
            "/api/bank-accounts/{}/transactions",
            bank_account_id
        ))
        .json(&CreateTransactionRequest::new(
            payee_id,
            Decimal::from_f32(10.15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 9, 25).unwrap(),
            budget_id,
        ))
        .await;

    response.assert_created();
    let transaction_id: Uuid = response.json();

    let transactions = db::transactions::get(&db_pool, bank_account_id)
        .await
        .unwrap();

    let expected = vec![Transaction::new(
        transaction_id,
        payee_id,
        NaiveDate::from_ymd_opt(2024, 9, 25).unwrap(),
        Decimal::from_f32(10.15).unwrap(),
        bank_account_id,
        budget_id,
    )]
    .into_boxed_slice();

    assert_eq!(transactions, expected);
}

#[sqlx::test]
pub async fn get_transactions(db_pool: MySqlPool) {
    let test_server = integration_test_init(db_pool.clone());
    test_init(&db_pool).await;

    let payee_id = *PAYEE_ID.unwrap();
    let bank_account_id = *BANK_ACCOUNT_ID.unwrap();
    let budget_id = *BUDGET_ID.unwrap();

    let transaction = Transaction::new(
        Uuid::new_v4(),
        payee_id,
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        Decimal::from_f32(1.3).unwrap(),
        bank_account_id,
        budget_id,
    );
    db::transactions::create(
        &db_pool,
        transaction.id,
        bank_account_id,
        CreateTransactionRequest::new(
            transaction.payee_id,
            transaction.amount,
            transaction.date,
            budget_id,
        ),
    )
    .await
    .unwrap();

    let response = test_server
        .get(&format!(
            "/api/bank-accounts/{bank_account_id}/transactions"
        ))
        .await;

    response.assert_ok();
    response.assert_json(&vec![transaction])
}

#[sqlx::test]
pub async fn update_transaction(db_pool: MySqlPool) {
    let test_server = integration_test_init(db_pool.clone());
    test_init(&db_pool).await;

    let user_id = *USER_ID.unwrap();
    let payee_id = *PAYEE_ID.unwrap();
    let bank_account_id = *BANK_ACCOUNT_ID.unwrap();
    let budget_id = *BUDGET_ID.unwrap();
    let transaction_id = Uuid::new_v4();
    let payee_id_2 = Uuid::new_v4();
    let budget_id_2 = Uuid::new_v4();

    db::payees::create(
        &db_pool,
        payee_id_2,
        CreatePayeeRequest::new("name".into(), user_id),
    )
    .await
    .unwrap();
    db::budgets::create(
        &db_pool,
        Budget::new(budget_id_2, "name".into(), None, user_id),
    )
    .await
    .unwrap();

    let transaction = Transaction::new(
        transaction_id,
        payee_id,
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        dec!(1.3),
        bank_account_id,
        budget_id,
    );
    db::transactions::create(
        &db_pool,
        transaction.id,
        bank_account_id,
        CreateTransactionRequest::new(
            transaction.payee_id,
            transaction.amount,
            transaction.date,
            budget_id,
        ),
    )
    .await
    .unwrap();

    let response = test_server
        .put(&format!(
            "/api/transactions/{transaction_id}"
        ))
        .json(&UpdateTransactionRequest::new(
            dec!(-1.2),
            payee_id_2,
            budget_id_2,
            NaiveDate::from_ymd_opt(2024, 10, 5).unwrap(),
        ))
        .await;
    
    response.assert_ok();
}

#[sqlx::test]
pub async fn delete_transaction(db_pool: MySqlPool) {
    let test_server = integration_test_init(db_pool.clone());
    test_init(&db_pool).await;

    let payee_id = *PAYEE_ID.unwrap();
    let bank_account_id = *BANK_ACCOUNT_ID.unwrap();
    let budget_id = *BUDGET_ID.unwrap();
    let user_id = *USER_ID.unwrap();

    let transaction = Transaction::new(
        Uuid::new_v4(),
        payee_id,
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        Decimal::from_f32(1.3).unwrap(),
        bank_account_id,
        budget_id,
    );
    db::transactions::create(
        &db_pool,
        transaction.id,
        bank_account_id,
        CreateTransactionRequest::new(
            transaction.payee_id,
            transaction.amount,
            transaction.date,
            budget_id,
        ),
    )
    .await
    .unwrap();   
    
    let response = test_server.delete(&format!("/api/transactions/{}?user_id={}", transaction.id, user_id))
        .await;
    
    response.assert_ok();
    
    let find_response = db::transactions::get_single(
        &db_pool,
        transaction.id).await;
    
    assert!(matches!(find_response, Err(Error::NotFound)));
}