use crate::AppError;

pub mod bank_accounts;
pub mod budgets;
pub mod payees;
pub mod schedule;
pub mod transactions;
pub mod users;

#[derive(Debug)]
pub enum DbError {
    NotFound,
    MappingError { error: anyhow::Error },
    Unknown { error: anyhow::Error },
}

impl DbError {
    pub fn to_app_error(self, error: anyhow::Error) -> AppError {
        match self {
            Self::NotFound => AppError::NotFound(error),
            Self::Unknown { .. } => AppError::InternalServerError(error),
            Self::MappingError { error } => AppError::InternalServerError(error),
        }
    }
}

impl From<sqlx::Error> for DbError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => Self::NotFound,
            err => DbError::Unknown { error: err.into() },
        }
    }
}
