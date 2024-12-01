mod common;

use std::sync::OnceLock;

use budgeting_backend::models::{BudgetAssignment, BudgetAssignmentSource};
use budgeting_backend::{
    db::{self, Error},
    models::{
        Budget, CreateBankAccountRequest, CreatePayeeRequest, CreateTransactionRequest,
        Transaction, UpdateTransactionRequest, User,
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
        User::new(user_id, "name".into(), "email@email.com".into(), None),
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
        Budget::new(budget_id, "Budget".into(), None, user_id, vec![]),
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

    let mut budgets = db::budgets::get_single(&db_pool, budget_id).await.unwrap();

    assert_eq!(budgets.assignments.len(), 1);

    let mut assignment = budgets.assignments.remove(0);
    assignment.id = Uuid::nil();

    assert_eq!(
        assignment,
        BudgetAssignment {
            id: Uuid::nil(),
            amount: dec!(10.15),
            date: NaiveDate::from_ymd_opt(2024, 9, 25).unwrap(),
            source: BudgetAssignmentSource::Transaction {
                from_transaction_id: transaction_id
            }
        }
    );

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
    db::transactions::create(&db_pool, transaction.clone())
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

    let mut budget1 = db::budgets::get_single(&db_pool, budget_id).await.unwrap();
    let budget2 = Budget::new(budget_id_2, "name".into(), None, user_id, vec![]);

    db::budgets::create(&db_pool, budget2.clone())
        .await
        .unwrap();

    let mut transaction = Transaction::new(
        transaction_id,
        payee_id,
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        dec!(1.3),
        bank_account_id,
        budget_id,
    );
    db::transactions::create(&db_pool, transaction.clone())
        .await
        .unwrap();

    let assignment = BudgetAssignment {
        id: Uuid::new_v4(),
        amount: transaction.amount,
        date: transaction.date,
        source: BudgetAssignmentSource::Transaction {
            from_transaction_id: transaction_id,
        },
    };

    budget1.assignments.push(assignment.clone());

    db::budgets::update(&db_pool, budget1).await.unwrap();

    let response = test_server
        .put(&format!("/api/transactions/{transaction_id}"))
        .json(&UpdateTransactionRequest::new(
            dec!(-1.2),
            payee_id_2,
            budget_id_2,
            NaiveDate::from_ymd_opt(2024, 10, 5).unwrap(),
        ))
        .await;

    transaction.amount = dec!(-1.2);
    transaction.payee_id = payee_id_2;
    transaction.date = NaiveDate::from_ymd_opt(2024, 10, 5).unwrap();
    transaction.budget_id = budget_id_2;

    response.assert_ok();

    let fetched_transaction = db::transactions::get_single(&db_pool, transaction_id)
        .await
        .unwrap();
    let fetched_budget_1 = db::budgets::get_single(&db_pool, budget_id).await.unwrap();
    let mut fetched_budget_2 = db::budgets::get_single(&db_pool, budget_id_2)
        .await
        .unwrap();

    assert_eq!(fetched_transaction, transaction);

    assert_eq!(fetched_budget_1.assignments.len(), 0);
    assert_eq!(fetched_budget_2.assignments.len(), 1);
    let mut fetched_assignment = fetched_budget_2.assignments.remove(0);
    fetched_assignment.id = Uuid::nil();

    assert_eq!(
        fetched_assignment,
        BudgetAssignment {
            id: Uuid::nil(),
            amount: dec!(-1.2),
            date: NaiveDate::from_ymd_opt(2024, 10, 5).unwrap(),
            ..assignment
        }
    )
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
    db::transactions::create(&db_pool, transaction.clone())
        .await
        .unwrap();

    let mut budget = db::budgets::get_single(&db_pool, budget_id).await.unwrap();
    budget.assign_from_transaction(&transaction);
    db::budgets::update(&db_pool, budget).await.unwrap();

    let response = test_server
        .delete(&format!(
            "/api/transactions/{}?user_id={}",
            transaction.id, user_id
        ))
        .await;

    response.assert_ok();

    let find_response = db::transactions::get_single(&db_pool, transaction.id).await;

    assert!(matches!(find_response, Err(Error::NotFound)));

    let fetched_budget = db::budgets::get_single(&db_pool, budget_id).await.unwrap();
    assert!(fetched_budget.assignments.is_empty());
}
