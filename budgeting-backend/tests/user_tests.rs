mod common;

use budgeting_backend::{
    db,
    models::{CreateUserRequest, User},
};
use common::*;
use sqlx::MySqlPool;
use uuid::Uuid;

#[sqlx::test]
pub async fn create_users(db_pool: MySqlPool) {
    let server = integration_test_init(db_pool.clone());

    let response = server
        .post("/api/users")
        .json(&CreateUserRequest {
            email: "someone@somewhere.com".to_owned(),
            name: "Someone".to_owned(),
        })
        .await;

    response.assert_created();
    let user_id = response.json::<Uuid>();

    let user = db::users::get_user(&db_pool, user_id).await.unwrap();

    assert_eq!(
        user,
        User {
            id: user_id,
            email: "someone@somewhere.com".to_owned(),
            name: "Someone".to_owned()
        }
    )
}

#[sqlx::test]
pub async fn get_users(db_pool: MySqlPool) {
    let server = integration_test_init(db_pool.clone());

    let mut users = vec![
        User {
            id: Uuid::new_v4(),
            email: "someone@somewhere.com".to_owned(),
            name: "Someone".to_owned(),
        },
        User {
            id: Uuid::new_v4(),
            email: "someone+1@somewhere.com".to_owned(),
            name: "Someone else".to_owned(),
        },
    ];
    users.sort_by_key(|x| x.id);

    for user in users.iter() {
        db::users::create_user(
            &db_pool,
            user.id,
            CreateUserRequest::new(user.name.clone(), user.email.clone()),
        )
        .await
        .unwrap();
    }

    let response = server.get("/api/users").await;
    response.assert_ok();
    response.assert_json(&users);
}

#[sqlx::test]
pub async fn get_user(db_pool: MySqlPool) {
    let test_server = integration_test_init(db_pool.clone());

    let user_id = Uuid::new_v4();
    let user = User::new(user_id, "Name".to_owned(), "email@email.com".to_owned());
    db::users::create_user(
        &db_pool,
        user_id,
        CreateUserRequest::new(user.name.clone(), user.email.clone()),
    )
    .await
    .unwrap();

    let response = test_server.get(&format!("/api/users/{user_id}")).await;

    response.assert_ok();
    response.assert_json(&user);
}
