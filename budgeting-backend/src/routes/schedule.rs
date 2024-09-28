use std::str::FromStr;

use anyhow::anyhow;
use chrono::NaiveDate;
use uuid::Uuid;

#[derive(PartialEq, Debug, Clone)]
pub struct Schedule {
    pub id: Uuid,
    pub period: SchedulePeriod,
}

#[derive(PartialEq, Debug, Clone)]
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

#[derive(PartialEq, Debug, Clone, Copy)]
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
            other => Err(anyhow!("Unexpected SchedulePeriodType {other}"))
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
