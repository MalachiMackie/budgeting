use anyhow::anyhow;
use chrono::NaiveDate;
use sqlx::{prelude::FromRow, MySql, MySqlPool};
use uuid::Uuid;

use crate::models::{Schedule, SchedulePeriod};

use super::Error;

#[derive(FromRow, PartialEq, Debug, Clone)]
struct ScheduleDbModel {
    id: String,
    period_type: String,
    period_starting_on: Option<NaiveDate>,
    custom_period_type: Option<String>,
    custom_period_every_count: Option<i32>,
}

impl TryFrom<ScheduleDbModel> for Schedule {
    type Error = anyhow::Error;

    fn try_from(value: ScheduleDbModel) -> Result<Self, Self::Error> {
        Ok(Schedule {
            id: value.id.parse()?,
            period: match value.period_type.as_str() {
                "Weekly" => SchedulePeriod::Weekly {
                    starting_on: value.period_starting_on.ok_or(anyhow::anyhow!(
                        "period_starting_on must be set when type is weekly"
                    ))?,
                },
                "Fortnightly" => SchedulePeriod::Fortnightly {
                    starting_on: value.period_starting_on.ok_or(anyhow::anyhow!(
                        "period_starting_on must be set when type is fortnightly"
                    ))?,
                },
                "Monthly" => SchedulePeriod::Monthly {
                    starting_on: value.period_starting_on.ok_or(anyhow::anyhow!(
                        "period_starting_on must be set when type is Monthly"
                    ))?,
                },
                "Yearly" => SchedulePeriod::Yearly {
                    starting_on: value.period_starting_on.ok_or(anyhow::anyhow!(
                        "period_starting_on must be set when type is yearly"
                    ))?,
                },
                "Custom" => SchedulePeriod::Custom {
                    period: value
                        .custom_period_type
                        .as_deref()
                        .ok_or(anyhow!(
                            "custom_period_type must be set when period_type is custom"
                        ))?
                        .parse()?,
                    #[allow(clippy::cast_possible_truncation)]
                    #[allow(clippy::cast_sign_loss)]
                    every_x_periods: value.custom_period_every_count.ok_or(anyhow!(
                        "Expected custom_period_every_count to be set when period_type is custom"
                    ))? as u8,
                },
                _ => return Err(anyhow!("Unexpected period_type {}", value.period_type)),
            },
        })
    }
}

impl From<Schedule> for ScheduleDbModel {
    fn from(value: Schedule) -> Self {
        Self {
            id: value.id.as_simple().to_string(),
            period_type: value.period.to_string(),
            period_starting_on: match value.period {
                SchedulePeriod::Weekly { starting_on }
                | SchedulePeriod::Fortnightly { starting_on }
                | SchedulePeriod::Monthly { starting_on }
                | SchedulePeriod::Yearly { starting_on } => Some(starting_on),
                SchedulePeriod::Custom { .. } => None,
            },
            custom_period_type: if let SchedulePeriod::Custom { period, .. } = &value.period {
                Some(period.to_string())
            } else {
                None
            },
            custom_period_every_count: if let SchedulePeriod::Custom {
                every_x_periods, ..
            } = &value.period
            {
                Some(i32::from(*every_x_periods))
            } else {
                None
            },
        }
    }
}

pub async fn create(db_pool: &MySqlPool, schedule: Schedule) -> Result<(), Error> {
    let db_model: ScheduleDbModel = schedule.into();

    sqlx::query!(
        r"
INSERT INTO Schedules (id, period_type, period_starting_on, custom_period_type, custom_period_every_count)
VALUE (?, ?, ?, ?, ?)",
        db_model.id,
        db_model.period_type,
        db_model.period_starting_on,
        db_model.custom_period_type,
        db_model.custom_period_every_count
    ).execute(db_pool).await?;

    Ok(())
}

pub async fn get_single(db_pool: &MySqlPool, id: Uuid) -> Result<Schedule, Error> {
    sqlx::query_as!(
        ScheduleDbModel,
        r"
SELECT id, period_type, period_starting_on, custom_period_type, custom_period_every_count
FROM Schedules
WHERE id = ?",
        id.as_simple()
    )
    .fetch_optional(db_pool)
    .await?
    .map(TryInto::try_into)
    .ok_or(Error::NotFound)?
    .map_err(|e| Error::MappingError { error: e })
}

pub async fn get_by_ids(db_pool: &MySqlPool, ids: &[Uuid]) -> Result<Box<[Schedule]>, Error> {
    let params = vec!["?"; ids.len()];

    let query_string = format!(
        r"SELECT id, period_type, period_starting_on, custom_period_type, custom_period_every_count
FROM Schedules
WHERE id IN ({})",
        params.join(", ")
    );

    let mut query = sqlx::query_as::<MySql, ScheduleDbModel>(query_string.as_str());

    for id in ids {
        query = query.bind(id.as_simple());
    }

    query
        .fetch_all(db_pool)
        .await?
        .into_iter()
        .map(TryInto::try_into)
        .collect::<Result<Box<[Schedule]>, _>>()
        .map_err(|e| Error::MappingError { error: e })
}

pub async fn delete(db_pool: &MySqlPool, id: Uuid) -> Result<(), Error> {
    sqlx::query!("DELETE FROM Schedules WHERE id = ?", id.as_simple())
        .execute(db_pool)
        .await?;

    Ok(())
}

pub async fn update(db_pool: &MySqlPool, schedule: Schedule) -> Result<(), Error> {
    let db_model: ScheduleDbModel = schedule.into();
    sqlx::query!(
        "UPDATE Schedules
    SET period_type = ?,
    period_starting_on = ?,
    custom_period_type = ?,
    custom_period_every_count = ?
    WHERE Id = ?",
        db_model.period_type,
        db_model.period_starting_on,
        db_model.custom_period_type,
        db_model.custom_period_every_count,
        db_model.id
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod mapping_tests {
        use crate::models::SchedulePeriodType;

        use super::*;

        #[allow(clippy::too_many_lines)]
        #[test]
        pub fn test_mapping_success() {
            let id = Uuid::new_v4();
            let started_on = NaiveDate::from_ymd_opt(2024, 9, 27).unwrap();
            let non_custom_db_model = ScheduleDbModel {
                id: id.as_simple().to_string(),
                period_type: "Weekly".into(),
                period_starting_on: Some(started_on),
                custom_period_type: None,
                custom_period_every_count: None,
            };
            let custom_db_model = ScheduleDbModel {
                id: id.as_simple().to_string(),
                period_type: "Custom".into(),
                period_starting_on: None,
                custom_period_type: Some("Weekly".into()),
                custom_period_every_count: Some(1),
            };
            let pairs = vec![
                (
                    Schedule {
                        id,
                        period: SchedulePeriod::Weekly {
                            starting_on: started_on,
                        },
                    },
                    ScheduleDbModel {
                        period_type: "Weekly".into(),
                        ..non_custom_db_model.clone()
                    },
                ),
                (
                    Schedule {
                        id,
                        period: SchedulePeriod::Fortnightly {
                            starting_on: started_on,
                        },
                    },
                    ScheduleDbModel {
                        period_type: "Fortnightly".into(),
                        ..non_custom_db_model.clone()
                    },
                ),
                (
                    Schedule {
                        id,
                        period: SchedulePeriod::Monthly {
                            starting_on: started_on,
                        },
                    },
                    ScheduleDbModel {
                        period_type: "Monthly".into(),
                        ..non_custom_db_model.clone()
                    },
                ),
                (
                    Schedule {
                        id,
                        period: SchedulePeriod::Yearly {
                            starting_on: started_on,
                        },
                    },
                    ScheduleDbModel {
                        period_type: "Yearly".into(),
                        ..non_custom_db_model.clone()
                    },
                ),
                (
                    Schedule {
                        id,
                        period: SchedulePeriod::Custom {
                            period: SchedulePeriodType::Weekly,
                            every_x_periods: 1,
                        },
                    },
                    ScheduleDbModel {
                        custom_period_type: Some("Weekly".into()),
                        ..custom_db_model.clone()
                    },
                ),
                (
                    Schedule {
                        id,
                        period: SchedulePeriod::Custom {
                            period: SchedulePeriodType::Fortnightly,
                            every_x_periods: 1,
                        },
                    },
                    ScheduleDbModel {
                        custom_period_type: Some("Fortnightly".into()),
                        ..custom_db_model.clone()
                    },
                ),
                (
                    Schedule {
                        id,
                        period: SchedulePeriod::Custom {
                            period: SchedulePeriodType::Monthly,
                            every_x_periods: 1,
                        },
                    },
                    ScheduleDbModel {
                        custom_period_type: Some("Monthly".into()),
                        ..custom_db_model.clone()
                    },
                ),
                (
                    Schedule {
                        id,
                        period: SchedulePeriod::Custom {
                            period: SchedulePeriodType::Yearly,
                            every_x_periods: 1,
                        },
                    },
                    ScheduleDbModel {
                        custom_period_type: Some("Yearly".into()),
                        ..custom_db_model.clone()
                    },
                ),
            ];

            for (domain, db_model) in pairs {
                let mapped_domain: ScheduleDbModel = domain.clone().into();
                assert_eq!(mapped_domain, db_model);

                let mapped_db_model: Result<Schedule, _> = db_model.try_into();
                match mapped_db_model {
                    Ok(mapped_db_model) => assert_eq!(mapped_db_model, domain),
                    Err(e) => panic!("{}", e),
                }
            }
        }

        #[test]
        pub fn test_mapping_failures() {
            let id = Uuid::new_v4();
            let started_on = NaiveDate::from_ymd_opt(2024, 9, 27).unwrap();
            let non_custom_db_model = ScheduleDbModel {
                id: id.as_simple().to_string(),
                period_type: "Weekly".into(),
                period_starting_on: Some(started_on),
                custom_period_type: None,
                custom_period_every_count: None,
            };
            let custom_db_model = ScheduleDbModel {
                id: id.as_simple().to_string(),
                period_type: "Custom".into(),
                period_starting_on: None,
                custom_period_type: Some("Weekly".into()),
                custom_period_every_count: Some(1),
            };
            let invalid_period_type: Result<Schedule, _> = ScheduleDbModel {
                period_type: "aoeu".into(),
                ..non_custom_db_model.clone()
            }
            .try_into();
            assert!(invalid_period_type.is_err());
            let missing_custom_type: Result<Schedule, _> = ScheduleDbModel {
                custom_period_type: None,
                ..custom_db_model.clone()
            }
            .try_into();
            assert!(missing_custom_type.is_err());
            let invalid_custom_type: Result<Schedule, _> = ScheduleDbModel {
                custom_period_type: Some("aoeu".into()),
                ..custom_db_model.clone()
            }
            .try_into();
            assert!(invalid_custom_type.is_err());
            let missing_custom_every_count: Result<Schedule, _> = ScheduleDbModel {
                custom_period_every_count: None,
                ..custom_db_model.clone()
            }
            .try_into();
            assert!(missing_custom_every_count.is_err());
        }
    }

    mod db_tests {
        use crate::models::SchedulePeriodType;

        use super::*;

        #[sqlx::test]
        pub async fn create_schedule_test(db_pool: MySqlPool) {
            let id = Uuid::new_v4();
            let result = create(
                &db_pool,
                Schedule {
                    id,
                    period: SchedulePeriod::Weekly {
                        starting_on: NaiveDate::from_ymd_opt(2024, 9, 27).unwrap(),
                    },
                },
            )
            .await;

            assert!(result.is_ok());

            let schedules = sqlx::query_as!(
                ScheduleDbModel,
                r"
SELECT id, period_type, period_starting_on, custom_period_type, custom_period_every_count
FROM Schedules"
            )
            .fetch_all(&db_pool)
            .await
            .unwrap();

            assert_eq!(
                schedules,
                vec![ScheduleDbModel {
                    id: id.as_simple().to_string(),
                    period_type: "Weekly".into(),
                    period_starting_on: Some(NaiveDate::from_ymd_opt(2024, 9, 27).unwrap()),
                    custom_period_type: None,
                    custom_period_every_count: None
                }]
            );
        }

        #[sqlx::test]
        pub async fn delete_schedule_test(db_pool: MySqlPool) {
            let id = Uuid::new_v4();
            create(
                &db_pool,
                Schedule {
                    id,
                    period: SchedulePeriod::Weekly {
                        starting_on: NaiveDate::from_ymd_opt(2024, 9, 27).unwrap(),
                    },
                },
            )
            .await
            .unwrap();

            let result = delete(&db_pool, id).await;
            assert!(result.is_ok());

            let find_result = get_single(&db_pool, id).await;

            assert!(matches!(find_result, Err(Error::NotFound)));
        }

        #[sqlx::test]
        pub async fn update_schedule_test(db_pool: MySqlPool) {
            let id = Uuid::new_v4();
            create(
                &db_pool,
                Schedule {
                    id,
                    period: SchedulePeriod::Weekly {
                        starting_on: NaiveDate::from_ymd_opt(2024, 9, 27).unwrap(),
                    },
                },
            )
            .await
            .unwrap();

            let updated = Schedule {
                id,
                period: SchedulePeriod::Custom {
                    period: SchedulePeriodType::Monthly,
                    every_x_periods: 4,
                },
            };

            let result = update(&db_pool, updated.clone()).await;
            assert!(result.is_ok());

            let find_result = get_single(&db_pool, id).await.unwrap();

            assert_eq!(updated, find_result);
        }
    }
}
