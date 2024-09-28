use std::str::FromStr;

use anyhow::anyhow;
use chrono::NaiveDate;
use derive_more::derive::Constructor;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Constructor, PartialEq, Debug, ToSchema)]
pub struct Payee {
    pub id: Uuid,
    pub name: String,
    pub user_id: Uuid,
}

#[derive(Serialize, Deserialize, Constructor, ToSchema)]
pub struct CreatePayeeRequest {
    pub name: String,
    pub user_id: Uuid,
}

#[derive(Deserialize, Serialize, Constructor, PartialEq, Debug, ToSchema)]
pub struct Transaction {
    pub id: Uuid,
    pub payee_id: Uuid,
    pub date: NaiveDate,
    #[schema(value_type = f32)]
    #[serde(with = "rust_decimal::serde::float")]
    pub amount: Decimal,
    pub bank_account_id: Uuid,
}

#[derive(Deserialize, Serialize, Constructor, ToSchema)]
pub struct CreateTransactionRequest {
    pub payee_id: Uuid,
    #[schema(value_type = f32)]
    #[serde(with = "rust_decimal::serde::float")]
    pub amount: Decimal,
    pub date: NaiveDate,
}

#[derive(Deserialize, Serialize, Constructor, ToSchema, Debug, PartialEq)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
}

#[derive(Deserialize, Serialize, ToSchema, Constructor)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, ToSchema, Constructor)]
pub struct BankAccount {
    pub id: Uuid,
    pub name: String,
    #[schema(value_type = f32)]
    #[serde(with = "rust_decimal::serde::float")]
    pub initial_amount: Decimal,
    pub user_id: Uuid,
    #[schema(value_type = f32)]
    #[serde(with = "rust_decimal::serde::float")]
    pub balance: Decimal,
}

#[derive(Deserialize, Serialize, ToSchema, Constructor)]
pub struct CreateBankAccountRequest {
    pub name: String,
    #[schema(value_type = f32)]
    #[serde(with = "rust_decimal::serde::float")]
    pub initial_amount: Decimal,
    pub user_id: Uuid,
}

#[derive(Deserialize, Serialize, ToSchema, Constructor)]
pub struct CreateBudgetRequest {
    pub name: String,
    pub target: Option<CreateBudgetTargetRequest>,
    pub user_id: Uuid,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub enum CreateBudgetTargetRequest {
    OneTime {
        #[schema(value_type = f32)]
        #[serde(with = "rust_decimal::serde::float")]
        target_amount: Decimal,
    },
    Repeating {
        #[schema(value_type = f32)]
        #[serde(with = "rust_decimal::serde::float")]
        target_amount: Decimal,
        repeating_type: RepeatingTargetType,
        schedule: CreateScheduleRequest,
    },
}

#[derive(Clone, Debug, PartialEq, ToSchema, Serialize, Deserialize)]
pub struct Budget {
    pub id: Uuid,
    pub name: String,
    pub target: Option<BudgetTarget>,
    pub user_id: Uuid,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, ToSchema)]
pub enum BudgetTarget {
    OneTime {
        #[schema(value_type = f32)]
        #[serde(with = "rust_decimal::serde::float")]
        target_amount: Decimal,
    },
    Repeating {
        #[schema(value_type = f32)]
        #[serde(with = "rust_decimal::serde::float")]
        target_amount: Decimal,
        repeating_type: RepeatingTargetType,
        schedule: Schedule,
    },
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, ToSchema)]
pub enum RepeatingTargetType {
    BuildUpTo,
    RequireRepeating,
}

impl FromStr for RepeatingTargetType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BuildUpTo" => Ok(Self::BuildUpTo),
            "RequireRepeating" => Ok(Self::RequireRepeating),
            other => Err(anyhow!("Unexpected repeating_target_type {other}")),
        }
    }
}

impl ToString for RepeatingTargetType {
    fn to_string(&self) -> String {
        match self {
            Self::BuildUpTo => "BuildUpTo".into(),
            Self::RequireRepeating => "RequireRepeating".into(),
        }
    }
}

impl ToString for BudgetTarget {
    fn to_string(&self) -> String {
        match self {
            BudgetTarget::OneTime { .. } => "OneTime".into(),
            BudgetTarget::Repeating { .. } => "Repeating".into(),
        }
    }
}

#[derive(PartialEq, Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct Schedule {
    pub id: Uuid,
    pub period: SchedulePeriod,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreateScheduleRequest {
    pub period: SchedulePeriod,
}

#[derive(PartialEq, Debug, Clone, Deserialize, Serialize, ToSchema)]
pub enum SchedulePeriod {
    Weekly {
        starting_on: NaiveDate,
    },
    Fortnightly {
        starting_on: NaiveDate,
    },
    Monthly {
        starting_on: NaiveDate,
    },
    Yearly {
        starting_on: NaiveDate,
    },
    Custom {
        period: SchedulePeriodType,
        every_x_periods: u8,
    },
}

#[derive(PartialEq, Debug, Clone, Copy, Deserialize, Serialize, ToSchema)]
pub enum SchedulePeriodType {
    Weekly,
    Fortnightly,
    Monthly,
    Yearly,
}

impl FromStr for SchedulePeriodType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Weekly" => Ok(Self::Weekly),
            "Fortnightly" => Ok(Self::Fortnightly),
            "Monthly" => Ok(Self::Monthly),
            "Yearly" => Ok(Self::Yearly),
            other => Err(anyhow!("Unexpected SchedulePeriodType {other}")),
        }
    }
}

impl ToString for SchedulePeriodType {
    fn to_string(&self) -> String {
        match self {
            SchedulePeriodType::Weekly => "Weekly".into(),
            SchedulePeriodType::Fortnightly => "Fortnightly".into(),
            SchedulePeriodType::Monthly => "Monthly".into(),
            SchedulePeriodType::Yearly => "Yearly".into(),
        }
    }
}

impl ToString for SchedulePeriod {
    fn to_string(&self) -> String {
        match self {
            SchedulePeriod::Weekly { .. } => "Weekly".into(),
            SchedulePeriod::Fortnightly { .. } => "Fortnightly".into(),
            SchedulePeriod::Monthly { .. } => "Monthly".into(),
            SchedulePeriod::Yearly { .. } => "Yearly".into(),
            SchedulePeriod::Custom { .. } => "Custom".into(),
        }
    }
}
