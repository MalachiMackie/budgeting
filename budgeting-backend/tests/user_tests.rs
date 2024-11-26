mod common;

use budgeting_backend::{
    db,
    models::{
        CreateUserRequest, Schedule, SchedulePeriod, UpdateScheduleRequest, UpdateUserRequest, User,
    },
};
use chrono::NaiveDate;
use common::*;
use sqlx::MySqlPool;
use uuid::{uuid, Uuid};

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

    let user = db::users::get_single(&db_pool, user_id).await.unwrap();

    assert_eq!(
        user,
        User {
            id: user_id,
            email: "someone@somewhere.com".to_owned(),
            name: "Someone".to_owned(),
            pay_frequency: None
        }
    )
}

#[sqlx::test]
pub async fn get_users(db_pool: MySqlPool) {
    let server = integration_test_init(db_pool.clone());

    let schedule = Schedule {
        id: Uuid::new_v4(),
        period: SchedulePeriod::Weekly {
            starting_on: NaiveDate::from_ymd_opt(2024, 11, 24).unwrap(),
        },
    };

    db::schedule::create(&db_pool, schedule.clone())
        .await
        .unwrap();

    let mut users = vec![
        // seeded user
        User {
            id: uuid!("33e96445-9e8a-4ca1-b8d6-390cdfde698f"),
            name: "super-user".into(),
            email: "super.user@email.com".into(),
            pay_frequency: None,
        },
        User {
            id: Uuid::new_v4(),
            email: "someone@somewhere.com".to_owned(),
            name: "Someone".to_owned(),
            pay_frequency: Some(schedule),
        },
        User {
            id: Uuid::new_v4(),
            email: "someone+1@somewhere.com".to_owned(),
            name: "Someone else".to_owned(),
            pay_frequency: None,
        },
    ];
    users.sort_by_key(|x| x.id);

    for user in users.iter().filter(|u| u.email != "super.user@email.com") {
        db::users::create(&db_pool, user.clone()).await.unwrap();
    }

    let response = server.get("/api/users").await;
    response.assert_ok();
    response.assert_json(&users);
}

#[sqlx::test]
pub async fn get_user(db_pool: MySqlPool) {
    let test_server = integration_test_init(db_pool.clone());

    let schedule_id = Uuid::new_v4();
    let schedule = Schedule {
        id: schedule_id,
        period: SchedulePeriod::Fortnightly {
            starting_on: NaiveDate::from_ymd_opt(2024, 11, 24).unwrap(),
        },
    };

    db::schedule::create(&db_pool, schedule.clone())
        .await
        .unwrap();

    let user_id = Uuid::new_v4();
    let user = User::new(
        user_id,
        "Name".to_owned(),
        "email@email.com".to_owned(),
        Some(schedule),
    );
    db::users::create(
        &db_pool,
        User::new(user_id, user.name.clone(), user.email.clone(), None),
    )
    .await
    .unwrap();

    db::users::update(&db_pool, user.clone()).await.unwrap();

    let response = test_server.get(&format!("/api/users/{user_id}")).await;

    response.assert_ok();
    response.assert_json(&user);
}

#[sqlx::test]
pub async fn update_user(db_pool: MySqlPool) {
    let test_server = integration_test_init(db_pool.clone());

    let user_id = Uuid::new_v4();

    db::users::create(
        &db_pool,
        User::new(user_id, "name".into(), "email@email.com".into(), None),
    )
    .await
    .unwrap();

    let response = test_server
        .put(&format!("/api/users/{user_id}"))
        .json(&UpdateUserRequest::new(
            "new name".into(),
            Some(UpdateScheduleRequest {
                period: SchedulePeriod::Weekly {
                    starting_on: NaiveDate::from_ymd_opt(2024, 11, 26).unwrap(),
                },
            }),
        ))
        .await;

    response.assert_ok();

    let mut fetched_user = db::users::get_single(&db_pool, user_id).await.unwrap();

    if let Some(schedule) = &mut fetched_user.pay_frequency {
        // clear the id because we don't need to assert on it
        schedule.id = Uuid::nil();
    }

    assert_eq!(
        fetched_user,
        User::new(
            user_id,
            "new name".into(),
            "email@email.com".into(),
            Some(Schedule {
                id: Uuid::nil(),
                period: SchedulePeriod::Weekly {
                    starting_on: NaiveDate::from_ymd_opt(2024, 11, 26).unwrap()
                }
            })
        )
    )
}
