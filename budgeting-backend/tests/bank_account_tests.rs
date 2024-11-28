mod common;

use common::{OnceLockExt, TestResponseExt};
use rust_decimal_macros::dec;

use std::sync::OnceLock;

use budgeting_backend::{
    db::{self, Error},
    models::{BankAccount, Budget, CreateBankAccountRequest, UpdateBankAccountRequest, User},
};
use common::*;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use sqlx::MySqlPool;
use uuid::Uuid;

static USER_ID: OnceLock<Uuid> = OnceLock::new();
static BANK_ACCOUNT_ID: OnceLock<Uuid> = OnceLock::new();
static BUDGET_ID: OnceLock<Uuid> = OnceLock::new();

async fn test_init(db_pool: &MySqlPool) {
    let user_id = *USER_ID.get_or_init(Uuid::new_v4);
    let bank_account_id = *BANK_ACCOUNT_ID.get_or_init(Uuid::new_v4);
    let budget_id = *BUDGET_ID.get_or_init(Uuid::new_v4);

    db::users::create(
        db_pool,
        User::new(
            user_id,
            "name".to_owned(),
            "someone@email.com".to_owned(),
            None,
        ),
    )
    .await
    .unwrap();

    db::bank_accounts::create(
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

    db::budgets::create(
        db_pool,
        Budget::new(budget_id, "Budget".into(), None, user_id, vec![]),
    )
    .await
    .unwrap();
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

    let found_bank_account = db::bank_accounts::get_single(&db_pool, bank_account_id, user_id)
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
pub async fn get_bank_account(db_pool: MySqlPool) {
    let test_server = integration_test_init(db_pool.clone());
    test_init(&db_pool).await;

    let bank_account_id = *BANK_ACCOUNT_ID.unwrap();
    let user_id = *USER_ID.unwrap();

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
        Decimal::from_f32(13.63).unwrap(),
    ));
}

#[sqlx::test]
pub async fn update_bank_account(db_pool: MySqlPool) {
    let test_server = integration_test_init(db_pool.clone());
    test_init(&db_pool).await;

    let user_id = *USER_ID.unwrap();
    let id = Uuid::new_v4();

    db::bank_accounts::create(
        &db_pool,
        id,
        CreateBankAccountRequest::new("name".into(), dec!(0), user_id),
    )
    .await
    .unwrap();

    let response = test_server
        .put(&format!("/api/bank-accounts/{id}?user_id={user_id}"))
        .json(&UpdateBankAccountRequest::new("newName".into()))
        .await;

    response.assert_ok();

    let get_result = db::bank_accounts::get_single(&db_pool, id, user_id)
        .await
        .unwrap();

    let expected = BankAccount::new(id, "newName".into(), dec!(0), user_id, dec!(0));

    assert_eq!(get_result, expected);
}

#[sqlx::test]
pub async fn delete_bank_account(db_pool: MySqlPool) {
    let test_server = integration_test_init(db_pool.clone());
    test_init(&db_pool).await;

    let user_id = *USER_ID.unwrap();
    let id = Uuid::new_v4();

    db::bank_accounts::create(
        &db_pool,
        id,
        CreateBankAccountRequest::new("name".into(), dec!(0), user_id),
    )
    .await
    .unwrap();

    let response = test_server
        .delete(&format!("/api/bank-accounts/{id}?user_id={user_id}"))
        .await;

    response.assert_ok();

    let get_result = db::bank_accounts::get_single(&db_pool, id, user_id).await;

    assert!(matches!(get_result, Err(Error::NotFound)));
}
