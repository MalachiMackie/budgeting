mod common;

use chrono::NaiveDate;
use common::{OnceLockExt, TestResponseExt};

use std::sync::OnceLock;

use budgeting_backend::{
    db,
    models::{
        BankAccount, Budget, CreateBankAccountRequest, CreatePayeeRequest, CreateTransactionRequest, CreateUserRequest
    },
};
use common::*;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use sqlx::MySqlPool;
use uuid::Uuid;

static USER_ID: OnceLock<Uuid> = OnceLock::new();
static BANK_ACCOUNT_ID: OnceLock<Uuid> = OnceLock::new();
static BUDGET_ID: OnceLock<Uuid> = OnceLock::new();

async fn test_init(db_pool: &MySqlPool) {
    let user_id = *USER_ID.get_or_init(|| Uuid::new_v4());
    let bank_account_id = *BANK_ACCOUNT_ID.get_or_init(|| Uuid::new_v4());
    let budget_id = *BUDGET_ID.get_or_init(|| Uuid::new_v4());

    db::users::create_user(
        db_pool,
        user_id,
        CreateUserRequest::new("name".to_owned(), "someone@email.com".to_owned()),
    )
    .await
    .unwrap();

    db::bank_accounts::create_bank_account(
        db_pool,
        bank_account_id,
        CreateBankAccountRequest::new(
            "My Bank Account".to_owned(),
            Decimal::from_f32(13.63).unwrap(),
            user_id,
        ),
    )
    .await
    .unwrap();
    
    db::budgets::create_budget(
        db_pool,
        Budget::new(budget_id, "Budget".into(), None, user_id))
        .await.unwrap();
}

#[sqlx::test]
pub async fn create_bank_account(db_pool: MySqlPool) {
    let test_server = integration_test_init(db_pool.clone());
    test_init(&db_pool).await;

    let user_id = *USER_ID.unwrap();

    let response = test_server
        .post("/api/bank-accounts")
        .json(&CreateBankAccountRequest {
            initial_amount: Decimal::from_f32(13.63).unwrap(),
            name: "My Bank Account".to_owned(),
            user_id,
        })
        .await;

    response.assert_created();
    let bank_account_id: Uuid = response.json();

    let found_bank_account =
        db::bank_accounts::get_bank_account(&db_pool, bank_account_id, user_id)
            .await
            .unwrap();

    assert_eq!(
        found_bank_account,
        BankAccount {
            id: bank_account_id,
            user_id,
            name: "My Bank Account".to_owned(),
            initial_amount: Decimal::from_f32(13.63).unwrap(),
            balance: Decimal::from_f32(13.63).unwrap()
        }
    )
}

#[sqlx::test]
pub async fn get_bank_account_without_transactions(db_pool: MySqlPool) {
    let test_server = integration_test_init(db_pool.clone());
    test_init(&db_pool).await;

    let bank_account_id = BANK_ACCOUNT_ID.unwrap();
    let user_id = USER_ID.unwrap();

    let response = test_server
        .get(&format!(
            "/api/bank-accounts/{}?user_id={}",
            bank_account_id, user_id
        ))
        .await;

    let expected_response = BankAccount::new(
        *bank_account_id,
        "My Bank Account".to_owned(),
        Decimal::from_f32(13.63).unwrap(),
        *user_id,
        Decimal::from_f32(13.63).unwrap(),
    );

    response.assert_ok();
    response.assert_json(&expected_response);
}

#[sqlx::test]
pub async fn get_bank_account_with_transactions(db_pool: MySqlPool) {
    let test_server = integration_test_init(db_pool.clone());
    test_init(&db_pool).await;

    let bank_account_id = *BANK_ACCOUNT_ID.unwrap();
    let user_id = *USER_ID.unwrap();
    let payee_id = Uuid::new_v4();
    let budget_id = *BUDGET_ID.unwrap();

    db::payees::create_payee(
        &db_pool,
        payee_id,
        CreatePayeeRequest::new("payee".into(), user_id),
    )
    .await
    .unwrap();

    db::transactions::create_transaction(
        &db_pool,
        Uuid::new_v4(),
        bank_account_id,
        CreateTransactionRequest::new(
            payee_id,
            Decimal::from_f32(12.34).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            budget_id
        ),
    )
    .await
    .unwrap();

    let response = test_server
        .get(&format!(
            "/api/bank-accounts/{bank_account_id}?user_id={user_id}"
        ))
        .await;

    response.assert_ok();
    response.assert_json(&BankAccount::new(
        bank_account_id,
        "My Bank Account".into(),
        Decimal::from_f32(13.63).unwrap(),
        user_id,
        Decimal::from_f32(13.63 + 12.34).unwrap(),
    ));
}
