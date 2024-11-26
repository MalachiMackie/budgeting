use anyhow::anyhow;
use axum::{
    extract::{Path, State},
    Json,
};
use email_address::EmailAddress;
use http::StatusCode;
use sqlx::MySqlPool;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{
    db,
    models::{
        CreateUserRequest, Schedule, SchedulePeriod, SchedulePeriodType, UpdateScheduleRequest,
        UpdateUserRequest, User,
    },
    AppError,
};

#[derive(OpenApi)]
#[openapi(
    paths(get, get_single, create, update),
    components(schemas(
        User,
        Schedule,
        CreateUserRequest,
        UpdateUserRequest,
        UpdateScheduleRequest,
        SchedulePeriod,
        SchedulePeriodType
    ))
)]
pub struct Api;

const API_TAG: &str = "Users";

#[utoipa::path(
    post,
    path = "/api/users",
    responses(
        (status = CREATED, description = "Success", body = Uuid, content_type = "application/json")
    ),
    request_body = CreateUserRequest,
    tag = API_TAG,
    operation_id = "createUser"
)]
pub async fn create(
    State(db_pool): State<MySqlPool>,
    Json(request): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<Uuid>), AppError> {
    if request.name.trim().is_empty() {
        return Err(AppError::BadRequest(anyhow!("User name must not be empty")));
    }

    if !EmailAddress::is_valid(&request.email) {
        return Err(AppError::BadRequest(anyhow!(
            "\"{}\" is not a valid email address",
            request.email
        )));
    }

    let id = Uuid::new_v4();

    db::users::create(
        &db_pool,
        User::new(
            id,
            request.name.trim().into(),
            request.email.trim().into(),
            None,
        ),
    )
    .await
    .map_err(|e| e.to_app_error(anyhow!("Could not create user")))?;

    Ok((StatusCode::CREATED, Json(id)))
}

#[utoipa::path(
    get,
    path = "/api/users",
    responses(
        (status = OK, description = "Success", body = Box<[User]>, content_type = "application/json")
    ),
    tag = API_TAG,
    operation_id = "getUsers"
)]
pub async fn get(State(db_pool): State<MySqlPool>) -> Result<Json<Box<[User]>>, AppError> {
    db::users::get(&db_pool)
        .await
        .map(Json)
        .map_err(|e| e.to_app_error(anyhow!("Could not get users")))
}

#[utoipa::path(
    get,
    path = "/api/users/{userId}",
    responses(
        (status = OK, description = "Success", body = User, content_type = "application/json")
    ),
    params(
        ("userId" = Uuid, Path,)
    ),
    tag = API_TAG,
    operation_id = "getUser"
)]
pub async fn get_single(
    State(db_pool): State<MySqlPool>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<User>, AppError> {
    db::users::get_single(&db_pool, user_id)
        .await
        .map_err(|e| e.to_app_error(anyhow!("Could not get user")))
        .map(Json)
}

#[utoipa::path(
    put,
    path = "/api/users/{userId}",
    responses(
        (status = OK, description = "Success")
    ),
    params(
        ("userId" = Uuid, Path,)
    ),
    request_body = UpdateUserRequest,
    tag = API_TAG,
    operation_id = "updateUser"
)]
pub async fn update(
    Path(user_id): Path<Uuid>,
    State(db_pool): State<MySqlPool>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<StatusCode, AppError> {
    if request.name.trim().is_empty() {
        return Err(AppError::BadRequest(anyhow!("User name must not be empty")));
    }

    let existing = db::users::get_single(&db_pool, user_id)
        .await
        .map_err(|e| e.to_app_error(anyhow!("Could not update user")))?;

    let (schedule, schedule_id_to_delete) = match (existing.pay_frequency, request.pay_frequency) {
        // update schedule
        (Some(existing_schedule), Some(updated_schedule)) => {
            let schedule = Schedule {
                id: existing_schedule.id,
                period: updated_schedule.period,
            };
            db::schedule::update(&db_pool, schedule.clone())
                .await
                .map_err(|e| e.to_app_error(anyhow!("Failed to update user")))?;

            (Some(schedule), None)
        }
        // create schedule
        (None, Some(updated_schedule)) => {
            let schedule = Schedule {
                id: Uuid::new_v4(),
                period: updated_schedule.period,
            };
            db::schedule::create(&db_pool, schedule.clone())
                .await
                .map_err(|e| e.to_app_error(anyhow!("Failed to update user")))?;

            (Some(schedule), None)
        }
        // delete schedule
        (Some(existing_schedule), None) => (None, Some(existing_schedule.id)),
        _ => (None, None),
    };

    db::users::update(
        &db_pool,
        User::new(existing.id, request.name, existing.email, schedule),
    )
    .await
    .map_err(|e| e.to_app_error(anyhow!("Could not update user")))?;

    if let Some(schedule_id) = schedule_id_to_delete {
        db::schedule::delete(&db_pool, schedule_id)
            .await
            .map_err(|e| e.to_app_error(anyhow!("Could not update user")))?;
    }

    Ok(StatusCode::OK)
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use chrono::NaiveDate;

    use super::*;

    static USER_ID: LazyLock<Uuid> = LazyLock::new(Uuid::new_v4);
    static SCHEDULE_ID: LazyLock<Uuid> = LazyLock::new(Uuid::new_v4);

    async fn init(db_pool: &MySqlPool) {
        db::schedule::create(
            db_pool,
            Schedule {
                id: *SCHEDULE_ID,
                period: SchedulePeriod::Weekly {
                    starting_on: NaiveDate::from_ymd_opt(2024, 11, 26).unwrap(),
                },
            },
        )
        .await
        .unwrap();
    }

    #[sqlx::test]
    pub async fn update_no_schedule(db_pool: MySqlPool) {
        init(&db_pool).await;
        let user_id = *USER_ID;
        db::users::create(
            &db_pool,
            User::new(user_id, "name".into(), "email@email.com".into(), None),
        )
        .await
        .unwrap();

        let response = update(
            Path(user_id),
            State(db_pool.clone()),
            Json(UpdateUserRequest::new("new_name".into(), None)),
        )
        .await
        .unwrap();

        assert_eq!(response, StatusCode::OK);

        let fetched = db::users::get_single(&db_pool, user_id).await.unwrap();

        assert_eq!(
            fetched,
            User::new(user_id, "new_name".into(), "email@email.com".into(), None)
        );
    }

    #[sqlx::test]
    pub async fn update_delete_schedule(db_pool: MySqlPool) {
        init(&db_pool).await;
        let user_id = *USER_ID;
        db::users::create(
            &db_pool,
            User::new(
                user_id,
                "name".into(),
                "email@email.com".into(),
                Some(Schedule {
                    id: *SCHEDULE_ID,
                    period: SchedulePeriod::Weekly {
                        starting_on: NaiveDate::from_ymd_opt(2024, 11, 26).unwrap(),
                    },
                }),
            ),
        )
        .await
        .unwrap();

        let response = update(
            Path(user_id),
            State(db_pool.clone()),
            Json(UpdateUserRequest::new("new_name".into(), None)),
        )
        .await
        .unwrap();

        assert_eq!(response, StatusCode::OK);

        let fetched = db::users::get_single(&db_pool, user_id).await.unwrap();

        assert_eq!(
            fetched,
            User::new(user_id, "new_name".into(), "email@email.com".into(), None)
        );
    }

    #[sqlx::test]
    pub async fn update_user_update_schedule(db_pool: MySqlPool) {
        init(&db_pool).await;
        let user_id = *USER_ID;
        db::users::create(
            &db_pool,
            User::new(
                user_id,
                "name".into(),
                "email@email.com".into(),
                Some(Schedule {
                    id: *SCHEDULE_ID,
                    period: SchedulePeriod::Weekly {
                        starting_on: NaiveDate::from_ymd_opt(2024, 11, 26).unwrap(),
                    },
                }),
            ),
        )
        .await
        .unwrap();

        let response = update(
            Path(user_id),
            State(db_pool.clone()),
            Json(UpdateUserRequest::new(
                "new_name".into(),
                Some(UpdateScheduleRequest {
                    period: SchedulePeriod::Fortnightly {
                        starting_on: NaiveDate::from_ymd_opt(2024, 11, 27).unwrap(),
                    },
                }),
            )),
        )
        .await
        .unwrap();

        assert_eq!(response, StatusCode::OK);

        let fetched = db::users::get_single(&db_pool, user_id).await.unwrap();

        assert_eq!(
            fetched,
            User::new(
                user_id,
                "new_name".into(),
                "email@email.com".into(),
                Some(Schedule {
                    id: *SCHEDULE_ID,
                    period: SchedulePeriod::Fortnightly {
                        starting_on: NaiveDate::from_ymd_opt(2024, 11, 27).unwrap(),
                    },
                })
            )
        );
    }

    #[sqlx::test]
    pub async fn update_create_schedule(db_pool: MySqlPool) {
        init(&db_pool).await;
        let user_id = *USER_ID;
        db::users::create(
            &db_pool,
            User::new(user_id, "name".into(), "email@email.com".into(), None),
        )
        .await
        .unwrap();

        let response = update(
            Path(user_id),
            State(db_pool.clone()),
            Json(UpdateUserRequest::new(
                "new_name".into(),
                Some(UpdateScheduleRequest {
                    period: SchedulePeriod::Fortnightly {
                        starting_on: NaiveDate::from_ymd_opt(2024, 11, 27).unwrap(),
                    },
                }),
            )),
        )
        .await
        .unwrap();

        assert_eq!(response, StatusCode::OK);

        let mut fetched = db::users::get_single(&db_pool, user_id).await.unwrap();
        
        if let Some(schedule) = &mut fetched.pay_frequency {
            schedule.id = Uuid::nil();
        }

        assert_eq!(
            fetched,
            User::new(
                user_id,
                "new_name".into(),
                "email@email.com".into(),
                Some(Schedule {
                    id: Uuid::nil(),
                    period: SchedulePeriod::Fortnightly {
                        starting_on: NaiveDate::from_ymd_opt(2024, 11, 27).unwrap(),
                    },
                })
            )
        );
    }
}
