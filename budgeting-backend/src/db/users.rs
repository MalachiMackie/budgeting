use std::{collections::HashMap, hash::RandomState};

use anyhow::bail;
use sqlx::{prelude::FromRow, MySql, MySqlPool};
use uuid::Uuid;

use crate::models::{Schedule, User};

use super::{schedule, Error};

#[derive(FromRow)]
struct UserDbModel {
    id: uuid::fmt::Simple,
    name: String,
    email: String,
    pay_schedule_id: Option<uuid::fmt::Simple>,
}

impl TryInto<User> for (Option<Schedule>, UserDbModel) {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<User, Self::Error> {
        let (maybe_schedule, user) = self;
        (user, maybe_schedule).try_into()
    }
}

impl TryInto<User> for (UserDbModel, Option<Schedule>) {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<User, Self::Error> {
        let (user_db_model, schedule) = self;

        if let (Some(pay_schedule_id), None) = (user_db_model.pay_schedule_id, &schedule) {
            bail!(
                "Missing schedule with id {pay_schedule_id} for user id {}",
                user_db_model.id
            );
        }

        Ok(User {
            email: user_db_model.email,
            id: user_db_model.id.into_uuid(),
            name: user_db_model.name,
            pay_frequency: schedule,
        })
    }
}

pub async fn get(db_pool: &MySqlPool) -> Result<Box<[User]>, Error> {
    let user_db_models =
        sqlx::query_as::<MySql, UserDbModel>("SELECT id, email, name, pay_schedule_id FROM Users")
            .fetch_all(db_pool)
            .await?;

    let schedule_ids: Vec<_> = user_db_models
        .iter()
        .filter_map(|x| x.pay_schedule_id.map(uuid::fmt::Simple::into_uuid))
        .collect();

    let schedules: Vec<_> = schedule::get_by_ids(db_pool, &schedule_ids).await?.into();

    let mut schedules_map = schedules
        .into_iter()
        .map(|x| (x.id, x))
        .collect::<HashMap<_, _, RandomState>>();

    let users = user_db_models
        .into_iter()
        .map(|user| {
            (
                user.pay_schedule_id
                    .and_then(|x| schedules_map.remove(x.as_uuid())),
                user,
            )
        })
        .map(TryInto::try_into)
        .collect::<Result<Box<[User]>, _>>()
        .map_err(|e| Error::MappingError { error: e })?;

    Ok(users)
}

pub async fn get_single(db_pool: &MySqlPool, user_id: Uuid) -> Result<User, Error> {
    let db_model = sqlx::query_as::<MySql, UserDbModel>(
        "SELECT id, name, email, pay_schedule_id FROM Users WHERE id = ?",
    )
    .bind(user_id.simple())
    .fetch_optional(db_pool)
    .await?
    .ok_or(Error::NotFound)?;

    let schedule = if let Some(schedule_id) = db_model.pay_schedule_id {
        Some(schedule::get_single(db_pool, schedule_id.into_uuid()).await?)
    } else {
        None
    };

    let user = (db_model, schedule)
        .try_into()
        .map_err(|e: anyhow::Error| Error::MappingError { error: e })?;

    Ok(user)
}

pub async fn create(db_pool: &MySqlPool, user: User) -> Result<(), Error> {
    sqlx::query!(
        "INSERT INTO Users(id, name, email, pay_schedule_id) VALUE (?, ?, ?, ?)",
        user.id.as_simple(),
        user.name,
        user.email,
        user.pay_frequency
            .map(|schedule| uuid::fmt::Simple::from(schedule.id))
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn update(db_pool: &MySqlPool, user: User) -> Result<(), Error> {
    sqlx::query!(
        "UPDATE Users SET name = ?, pay_schedule_id = ? WHERE id = ?",
        user.name,
        user.pay_frequency.map(|x| uuid::fmt::Simple::from(x.id)),
        user.id.as_simple()
    )
    .execute(db_pool)
    .await?;

    Ok(())
}
