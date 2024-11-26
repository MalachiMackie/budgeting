mod common;
use std::sync::LazyLock;

use budgeting_backend::{
    db,
    models::{CreatePayeeRequest, Payee, UpdatePayeeRequest, User},
};
use common::*;
use sqlx::MySqlPool;
use uuid::Uuid;

static USER_ID: LazyLock<Uuid> = LazyLock::new(Uuid::new_v4);

async fn test_init(db_pool: &MySqlPool) {
    let user_id = *USER_ID;

    db::users::create(
        db_pool,
        User::new(
            user_id,
            "name".to_owned(),
            "someone@somewhere.com".to_owned(),
            None,
        ),
    )
    .await
    .unwrap();
}

#[sqlx::test]
pub async fn create_payee(db_pool: MySqlPool) {
    let test_server = integration_test_init(db_pool.clone());
    test_init(&db_pool).await;

    let user_id = *USER_ID;

    let response = test_server
        .post("/api/payees")
        .json(&CreatePayeeRequest::new("Payee".to_owned(), user_id))
        .await;

    response.assert_created();
    let payee_id: Uuid = response.json();

    let found_payee = db::payees::get_single(&db_pool, payee_id).await.unwrap();

    assert_eq!(
        found_payee,
        Payee::new(payee_id, "Payee".to_owned(), user_id)
    )
}

#[sqlx::test]
pub async fn get_payees(db_pool: MySqlPool) {
    let test_server = integration_test_init(db_pool.clone());
    test_init(&db_pool).await;

    let payee_id = Uuid::new_v4();
    let user_id = *USER_ID;

    db::payees::create(
        &db_pool,
        payee_id,
        CreatePayeeRequest::new("Name".to_owned(), user_id),
    )
    .await
    .unwrap();

    let response = test_server
        .get(&format!("/api/payees?user_id={user_id}"))
        .await;

    response.assert_ok();
    response.assert_json(&[Payee::new(payee_id, "Name".to_owned(), user_id)]);
}

#[sqlx::test]
pub async fn update_payee(db_pool: MySqlPool) {
    let test_server = integration_test_init(db_pool.clone());
    test_init(&db_pool).await;

    let payee_id = Uuid::new_v4();
    let user_id = *USER_ID;

    db::payees::create(
        &db_pool,
        payee_id,
        CreatePayeeRequest::new("Name".to_owned(), user_id),
    )
    .await
    .unwrap();

    let response = test_server
        .put(&format!("/api/payees/{payee_id}"))
        .json(&UpdatePayeeRequest::new("NewName".into()))
        .await;

    response.assert_ok();

    let fetched = db::payees::get_single(&db_pool, payee_id).await.unwrap();

    assert_eq!(fetched.name, "NewName");
}

#[sqlx::test]
pub async fn delete_payee(db_pool: MySqlPool) {
    let test_server = integration_test_init(db_pool.clone());
    test_init(&db_pool).await;

    let id = Uuid::new_v4();
    let user_id = *USER_ID;

    db::payees::create(
        &db_pool,
        id,
        CreatePayeeRequest::new("name".into(), user_id),
    )
    .await
    .unwrap();

    let response = test_server.delete(&format!("/api/payees/{id}")).await;

    response.assert_ok();

    let fetched = db::payees::get_single(&db_pool, id).await;

    assert!(matches!(fetched, Err(db::Error::NotFound)));
}
