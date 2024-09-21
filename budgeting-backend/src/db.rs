use crate::AppError;

pub mod bank_accounts;
pub mod payees;
pub mod transactions;
pub mod users;

pub enum DbError {
    NotFound,
    Unknown,
}

impl DbError {
    pub fn to_app_error(self, error: anyhow::Error) -> AppError {
        match self {
            Self::NotFound => AppError::NotFound(error),
            Self::Unknown => AppError::InternalServerError(error),
        }
    }
}

impl From<sqlx::Error> for DbError {
    fn from(_value: sqlx::Error) -> Self {
        DbError::Unknown
    }
}
