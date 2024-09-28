use std::str::FromStr;

use anyhow::anyhow;
use rust_decimal::Decimal;
use uuid::Uuid;

use super::schedule::Schedule;

#[derive(Clone, Debug, PartialEq)]
pub struct Budget {
    pub id: Uuid,
    pub name: String,
    pub target: Option<BudgetTarget>,
    pub user_id: Uuid,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BudgetTarget {
    OneTime {
        target_amount: Decimal,
    },
    Repeating {
        target_amount: Decimal,
        repeating_type: RepeatingTargetType,
        schedule: Schedule,
    },
}

#[derive(Clone, Debug, PartialEq)]
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
            other => Err(anyhow!("Unexpected repeating_target_type {other}"))
        }
    }
}

impl ToString for RepeatingTargetType {
    fn to_string(&self) -> String {
        match self {
            Self::BuildUpTo => "BuildUpTo".into(),
            Self::RequireRepeating => "RequireRepeating".into()
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
