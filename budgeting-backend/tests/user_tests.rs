use budgeting_backend::{db, models::{CreateUserRequest, User}};
use common::test_init;
use http::StatusCode;
use sqlx::MySqlPool;
use uuid::Uuid;

mod common;

#[sqlx::test]
pub async fn create_users(db_pool: MySqlPool) {
    let server = test_init(db_pool.clone()).await;

    let response = server.post("/api/users")
        .json(&CreateUserRequest{email: "someone@somewhere.com".to_owned(), name: "Someone".to_owned()})
        .await;

    response.assert_status(StatusCode::CREATED);
    let user_id = response.json::<Uuid>();

    let user = db::users::get_user(&db_pool, user_id)
        .await.unwrap();

    assert_eq!(user, User{id: user_id, email: "someone@somewhere.com".to_owned(), name: "Someone".to_owned()})
}

#[sqlx::test]
pub async fn get_users(db_pool: MySqlPool) {
    let server = test_init(db_pool.clone()).await;

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
        db::users::create_user(&db_pool, user.id, CreateUserRequest{email: user.email.clone(), name: user.name.clone()})
            .await.unwrap();
    }

    let response = server.get("/api/users").await;
    response.assert_status_ok();
    response.assert_json(&users);
}
