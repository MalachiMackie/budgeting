use std::{fmt::Display, str::FromStr};

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

#[derive(Deserialize, Serialize, Constructor, PartialEq, Debug, ToSchema, Clone)]
pub struct Transaction {
    pub id: Uuid,
    pub payee_id: Uuid,
    pub date: NaiveDate,
    #[schema(value_type = f32)]
    #[serde(with = "rust_decimal::serde::float")]
    pub amount: Decimal,
    pub bank_account_id: Uuid,
    pub budget_id: Uuid,
}

#[derive(Deserialize, Serialize, Constructor, ToSchema)]
pub struct CreateTransactionRequest {
    pub payee_id: Uuid,
    #[schema(value_type = f32)]
    #[serde(with = "rust_decimal::serde::float")]
    pub amount: Decimal,
    pub date: NaiveDate,
    pub budget_id: Uuid,
}

#[derive(Deserialize, Serialize, Constructor, ToSchema, Debug, PartialEq, Clone)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub pay_frequency: Option<Schedule>,
}

#[derive(Deserialize, Serialize, ToSchema, Constructor)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}

#[derive(Deserialize, Serialize, ToSchema, Constructor)]
pub struct UpdateUserRequest {
    pub name: String,
    pub pay_frequency: Option<UpdateScheduleRequest>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, ToSchema, Constructor, Clone)]
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

#[derive(Deserialize, Serialize, ToSchema, Constructor)]
pub struct UpdateBudgetRequest {
    pub name: String,
    pub target: Option<UpdateBudgetTargetRequest>,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
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

#[derive(Serialize, Deserialize, ToSchema, Clone)]
#[serde(tag = "type")]
pub enum UpdateBudgetTargetRequest {
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
        schedule: UpdateScheduleRequest,
    },
}

#[derive(Clone, Debug, PartialEq, ToSchema, Serialize, Deserialize, Constructor)]
pub struct Budget {
    pub id: Uuid,
    pub name: String,
    pub target: Option<BudgetTarget>,
    pub user_id: Uuid,
    pub assignments: Vec<BudgetAssignment>,
}

#[derive(Clone, Debug, PartialEq, ToSchema, Serialize, Deserialize, Constructor)]
pub struct GetBudgetResponse {
    pub id: Uuid,
    pub name: String,
    pub target: Option<BudgetTarget>,
    pub user_id: Uuid,
    pub assignments: Vec<BudgetAssignment>,
    #[schema(value_type = f32)]
    #[serde(with = "rust_decimal::serde::float")]
    pub total_assigned: Decimal,
}

impl From<Budget> for GetBudgetResponse {
    fn from(value: Budget) -> Self {
        GetBudgetResponse {
            total_assigned: value.total_assigned(),
            id: value.id,
            target: value.target,
            user_id: value.user_id,
            assignments: value.assignments,
            name: value.name,
        }
    }
}

impl Budget {
    pub fn total_assigned(&self) -> Decimal {
        self.assignments.iter().map(|x| x.amount).sum()
    }

    pub fn assign_from_transaction(&mut self, transaction: &Transaction) {
        self.assignments.push(BudgetAssignment {
            id: Uuid::new_v4(),
            amount: transaction.amount,
            date: transaction.date,
            source: BudgetAssignmentSource::Transaction {
                from_transaction_id: transaction.id,
            },
        });
    }

    pub fn move_between_budgets(
        from: &mut Budget,
        to: &mut Budget,
        amount: Decimal,
        date: NaiveDate,
    ) {
        let link_id = Uuid::new_v4();
        to.assignments.push(BudgetAssignment {
            id: Uuid::new_v4(),
            amount,
            date,
            source: BudgetAssignmentSource::OtherBudget {
                from_budget_id: from.id,
                link_id,
            },
        });
        // add reverse assignment
        from.assignments.push(BudgetAssignment {
            id: Uuid::new_v4(),
            amount: -amount,
            date,
            source: BudgetAssignmentSource::OtherBudget {
                from_budget_id: to.id,
                link_id,
            },
        });
    }
}

#[derive(Clone, Debug, PartialEq, ToSchema, Serialize, Deserialize, Constructor)]
pub struct BudgetAssignment {
    pub id: Uuid,
    #[schema(value_type = f32)]
    #[serde(with = "rust_decimal::serde::float")]
    pub amount: Decimal,
    pub date: NaiveDate,
    pub source: BudgetAssignmentSource,
}

#[derive(Clone, Debug, PartialEq, ToSchema, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BudgetAssignmentSource {
    OtherBudget { from_budget_id: Uuid, link_id: Uuid },
    Transaction { from_transaction_id: Uuid },
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, ToSchema)]
#[serde(tag = "type")]
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

impl Display for RepeatingTargetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BuildUpTo => write!(f, "BuildUpTo"),
            Self::RequireRepeating => write!(f, "RequireRepeating"),
        }
    }
}

impl Display for BudgetTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BudgetTarget::OneTime { .. } => write!(f, "OneTime"),
            BudgetTarget::Repeating { .. } => write!(f, "Repeating"),
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

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct UpdateScheduleRequest {
    pub period: SchedulePeriod,
}

#[derive(PartialEq, Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(tag = "type")]
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

impl Display for SchedulePeriodType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchedulePeriodType::Weekly => write!(f, "Weekly"),
            SchedulePeriodType::Fortnightly => write!(f, "Fortnightly"),
            SchedulePeriodType::Monthly => write!(f, "Monthly"),
            SchedulePeriodType::Yearly => write!(f, "Yearly"),
        }
    }
}

impl Display for SchedulePeriod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchedulePeriod::Weekly { .. } => write!(f, "Weekly"),
            SchedulePeriod::Fortnightly { .. } => write!(f, "Fortnightly"),
            SchedulePeriod::Monthly { .. } => write!(f, "Monthly"),
            SchedulePeriod::Yearly { .. } => write!(f, "Yearly"),
            SchedulePeriod::Custom { .. } => write!(f, "Custom"),
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema, Constructor)]
pub struct UpdateTransactionRequest {
    #[schema(value_type = f32)]
    #[serde(with = "rust_decimal::serde::float")]
    pub amount: Decimal,
    pub payee_id: Uuid,
    pub budget_id: Uuid,
    pub date: NaiveDate,
}

#[derive(Serialize, Deserialize, ToSchema, Constructor)]
pub struct UpdateBankAccountRequest {
    pub name: String,
}

#[derive(Serialize, Deserialize, ToSchema, Constructor)]
pub struct UpdatePayeeRequest {
    pub name: String,
}

#[derive(ToSchema, Serialize, Deserialize)]
pub struct TransferBudgetRequest {
    pub date: NaiveDate,
    #[schema(value_type = f32)]
    #[serde(with = "rust_decimal::serde::float")]
    pub amount: Decimal,
}

#[cfg(test)]
mod tests {
    use super::*;

    mod budget_into_get_budget_response {
        use super::*;
        use rust_decimal_macros::dec;
        use std::sync::LazyLock;

        static BUDGET_ID: LazyLock<Uuid> = LazyLock::new(Uuid::new_v4);
        static USER_ID: LazyLock<Uuid> = LazyLock::new(Uuid::new_v4);

        #[test]
        #[allow(non_snake_case)]
        pub fn into_get_budget_response__no_assignments() {
            let budget = Budget {
                id: *BUDGET_ID,
                name: "name".into(),
                user_id: *USER_ID,
                target: Some(BudgetTarget::OneTime {
                    target_amount: Decimal::ZERO,
                }),
                assignments: vec![],
            };

            let expected = GetBudgetResponse {
                id: *BUDGET_ID,
                name: "name".into(),
                user_id: *USER_ID,
                target: Some(BudgetTarget::OneTime {
                    target_amount: Decimal::ZERO,
                }),
                assignments: vec![],
                total_assigned: Decimal::ZERO,
            };

            let mapped: GetBudgetResponse = budget.into();

            assert_eq!(mapped, expected);
        }

        #[test]
        #[allow(non_snake_case)]
        pub fn into_get_budget_response__one_assignment() {
            let assignment_id = Uuid::new_v4();
            let link_id = Uuid::new_v4();
            let budget = Budget {
                id: *BUDGET_ID,
                name: "name".into(),
                user_id: *USER_ID,
                target: Some(BudgetTarget::OneTime {
                    target_amount: Decimal::ZERO,
                }),
                assignments: vec![BudgetAssignment {
                    id: assignment_id,
                    amount: dec!(10),
                    date: NaiveDate::from_ymd_opt(2024, 11, 30).unwrap(),
                    source: BudgetAssignmentSource::OtherBudget {
                        from_budget_id: *BUDGET_ID,
                        link_id,
                    },
                }],
            };

            let expected = GetBudgetResponse {
                id: *BUDGET_ID,
                name: "name".into(),
                user_id: *USER_ID,
                target: Some(BudgetTarget::OneTime {
                    target_amount: Decimal::ZERO,
                }),
                assignments: vec![BudgetAssignment {
                    id: assignment_id,
                    amount: dec!(10),
                    date: NaiveDate::from_ymd_opt(2024, 11, 30).unwrap(),
                    source: BudgetAssignmentSource::OtherBudget {
                        from_budget_id: *BUDGET_ID,
                        link_id,
                    },
                }],
                total_assigned: dec!(10),
            };

            let mapped: GetBudgetResponse = budget.into();

            assert_eq!(mapped, expected);
        }

        #[test]
        #[allow(non_snake_case)]
        pub fn into_get_budget_response__multiple_assignments() {
            let assignment_id1 = Uuid::new_v4();
            let assignment_id2 = Uuid::new_v4();
            let link_id = Uuid::new_v4();
            let budget = Budget {
                id: *BUDGET_ID,
                name: "name".into(),
                user_id: *USER_ID,
                target: Some(BudgetTarget::OneTime {
                    target_amount: Decimal::ZERO,
                }),
                assignments: vec![
                    BudgetAssignment {
                        id: assignment_id1,
                        amount: dec!(10),
                        date: NaiveDate::from_ymd_opt(2024, 11, 30).unwrap(),
                        source: BudgetAssignmentSource::OtherBudget {
                            from_budget_id: *BUDGET_ID,
                            link_id,
                        },
                    },
                    BudgetAssignment {
                        id: assignment_id2,
                        amount: dec!(-50),
                        date: NaiveDate::from_ymd_opt(2024, 11, 30).unwrap(),
                        source: BudgetAssignmentSource::OtherBudget {
                            from_budget_id: *BUDGET_ID,
                            link_id,
                        },
                    },
                ],
            };

            let expected = GetBudgetResponse {
                id: *BUDGET_ID,
                name: "name".into(),
                user_id: *USER_ID,
                target: Some(BudgetTarget::OneTime {
                    target_amount: Decimal::ZERO,
                }),
                assignments: vec![
                    BudgetAssignment {
                        id: assignment_id1,
                        amount: dec!(10),
                        date: NaiveDate::from_ymd_opt(2024, 11, 30).unwrap(),
                        source: BudgetAssignmentSource::OtherBudget {
                            from_budget_id: *BUDGET_ID,
                            link_id,
                        },
                    },
                    BudgetAssignment {
                        id: assignment_id2,
                        amount: dec!(-50),
                        date: NaiveDate::from_ymd_opt(2024, 11, 30).unwrap(),
                        source: BudgetAssignmentSource::OtherBudget {
                            from_budget_id: *BUDGET_ID,
                            link_id,
                        },
                    },
                ],
                total_assigned: dec!(-40),
            };

            let mapped: GetBudgetResponse = budget.into();

            assert_eq!(mapped, expected);
        }
    }
}
