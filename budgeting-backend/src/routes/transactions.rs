use anyhow::anyhow;
use axum::{
    extract::{Path, State},
    Json,
};
use http::StatusCode;
use sqlx::MySqlPool;
use tokio::join;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::models::BudgetAssignmentSource;
use crate::{
    db::{self, Error},
    models::{CreateTransactionRequest, Transaction, UpdateTransactionRequest},
    AppError,
};

#[derive(OpenApi)]
#[openapi(
    paths(get, create, update, delete),
    components(schemas(Transaction, CreateTransactionRequest, UpdateTransactionRequest))
)]
pub struct Api;

const API_TAG: &str = "Transactions";

#[utoipa::path(
    get,
    path = "/api/bank-accounts/{bankAccountId}/transactions",
    responses(
        (status = OK, description = "Success", body = Box<[Transaction]>, content_type = "application/json")
    ),
    params(
        ("bankAccountId" = Uuid, Path,)
    ),
    tag = API_TAG,
    operation_id = "getTransactions"
)]
pub async fn get(
    State(db_pool): State<MySqlPool>,
    Path(bank_account_id): Path<Uuid>,
) -> Result<Json<Box<[Transaction]>>, AppError> {
    if bank_account_id.is_nil() {
        return Err(AppError::BadRequest(anyhow!("Bank account id must be set")));
    }

    db::transactions::get(&db_pool, bank_account_id)
        .await
        .map(Json)
        .map_err(|e| e.to_app_error(anyhow!("Could not get transactions")))
}

#[utoipa::path(
    post,
    path = "/api/bank-accounts/{bankAccountId}/transactions",
    responses(
        (status = CREATED, description = "Success", body = Uuid, content_type = "application/json")
    ),
    request_body = CreateTransactionRequest,
    params(
        ("bankAccountId" = Uuid, Path,)
    ),
    tag = API_TAG,
    operation_id = "createTransaction"
)]
pub async fn create(
    State(db_pool): State<MySqlPool>,
    Path(bank_account_id): Path<Uuid>,
    Json(request): Json<CreateTransactionRequest>,
) -> Result<(StatusCode, Json<Uuid>), AppError> {
    if request.payee_id.is_nil() {
        return Err(AppError::BadRequest(anyhow!("Payee Id must be set")));
    }

    if bank_account_id.is_nil() {
        return Err(AppError::BadRequest(anyhow!("Bank Account Id must be set")));
    }

    if request.budget_id.is_nil() {
        return Err(AppError::BadRequest(anyhow!("Budget Id must be set")));
    }

    let id = Uuid::new_v4();

    let (budget_result, payee_result) = join!(
        db::budgets::get_single(&db_pool, request.budget_id),
        db::payees::get_single(&db_pool, request.payee_id)
    );

    let mut budget = budget_result.map_err(|e| e.to_app_error(anyhow!("Could not get budget")))?;

    match payee_result {
        Ok(_) => (),
        Err(Error::NotFound) => {
            return Err(AppError::NotFound(anyhow::anyhow!(
                "Payee not found with id {}",
                request.payee_id
            )));
        }
        Err(e) => return Err(e.to_app_error(anyhow!("Could not create transaction"))),
    }

    let transaction = Transaction {
        id,
        date: request.date,
        amount: request.amount,
        payee_id: request.payee_id,
        bank_account_id,
        budget_id: request.budget_id,
    };

    budget.assign_from_transaction(&transaction);

    db::transactions::create(&db_pool, transaction)
        .await
        .map_err(|e| e.to_app_error(anyhow!("Could not create transaction")))?;

    // update budget must happen after transaction create because the budget assignment
    // has a foreign key to the transaction
    db::budgets::update(&db_pool, budget)
        .await
        .map_err(|e| e.to_app_error(anyhow!("Could not update budget")))?;

    Ok((StatusCode::CREATED, Json(id)))
}

#[utoipa::path(
    put,
    path = "/api/transactions/{transactionId}",
    responses(
        (status = OK, description = "Success",)
    ),
    request_body = UpdateTransactionRequest,
    params(
        ("bankAccountId" = Uuid, Path,),
        ("transactionId" = Uuid, Path,)
    ),
    tag = API_TAG,
    operation_id = "updateTransaction"
)]
pub async fn update(
    State(db_pool): State<MySqlPool>,
    Path(transaction_id): Path<Uuid>,
    Json(request): Json<UpdateTransactionRequest>,
) -> Result<(), AppError> {
    let (transaction_result, budget_by_transaction_id_result, budget_by_id_result) = join!(
        db::transactions::get_single(&db_pool, transaction_id),
        db::budgets::get_by_assignment_transaction_id(&db_pool, transaction_id),
        db::budgets::get_single(&db_pool, request.budget_id)
    );

    let mut transaction =
        transaction_result.map_err(|e| e.to_app_error(anyhow!("Failed to update transaction")))?;
    let budget_by_transaction_id = budget_by_transaction_id_result
        .map_err(|e| e.to_app_error(anyhow!("Failed to get budget with transaction assignment")))?;
    let mut budget_by_id =
        budget_by_id_result.map_err(|e| e.to_app_error(anyhow!("Failed to get budget by id")))?;

    let original_budget_id = transaction.budget_id;
    
    transaction.amount = request.amount;
    transaction.date = request.date;
    transaction.payee_id = request.payee_id;
    transaction.budget_id = request.budget_id;

    if let Some(mut budget_by_transaction_id) = budget_by_transaction_id {
        if original_budget_id != request.budget_id {
            budget_by_transaction_id.assignments.retain(|assignment| !matches!(
                assignment.source,
                BudgetAssignmentSource::Transaction { from_transaction_id } if from_transaction_id == transaction_id
            ));

            budget_by_id.assign_from_transaction(&transaction);

            let (result1, result2) = join!(
                db::budgets::update(&db_pool, budget_by_transaction_id),
                db::budgets::update(&db_pool, budget_by_id),
            );

            result1.map_err(|e| e.to_app_error(anyhow!("Failed to update budget")))?;
            result2.map_err(|e| e.to_app_error(anyhow!("Failed to update budget")))?;
        } else if let Some(assignment) = budget_by_transaction_id.assignments.iter_mut()
            .find(|assignment| matches!(
                assignment.source,
                BudgetAssignmentSource::Transaction { from_transaction_id } if from_transaction_id == transaction_id
            )) {
            assignment.amount = transaction.amount;
            assignment.date = transaction.date;
            
            db::budgets::update(&db_pool, budget_by_transaction_id).await.map_err(|e| 
                e.to_app_error(anyhow!("Failed to update budget")))?;
        }
    }

    db::transactions::update(&db_pool, transaction)
        .await
        .map_err(|e| e.to_app_error(anyhow!("Failed to update transaction")))?;

    Ok(())
}

#[utoipa::path(
    delete,
    path = "/api/transactions/{transactionId}",
    responses(
        (status = OK, description = "Success",)
    ),
    params(
        ("transactionId" = Uuid, Path,)
    ),
    tag = API_TAG,
    operation_id = "deleteTransaction"
)]
pub async fn delete(
    State(db_pool): State<MySqlPool>,
    Path(transaction_id): Path<Uuid>,
) -> Result<(), AppError> {
    let budget = db::budgets::get_by_assignment_transaction_id(&db_pool, transaction_id)
        .await
        .map_err(|e| e.to_app_error(anyhow!("Failed to get budget with transaction assignment")))?;

    if let Some(mut budget) = budget {
        budget.assignments.retain(|assignment| !matches!(
            assignment.source,
            BudgetAssignmentSource::Transaction {from_transaction_id} if from_transaction_id == transaction_id));

        db::budgets::update(&db_pool, budget)
            .await
            .map_err(|e| e.to_app_error(anyhow!("Failed to update transaction")))?;
    }

    db::transactions::delete(&db_pool, transaction_id)
        .await
        .map_err(|e| e.to_app_error(anyhow!("Failed to delete transaction")))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        Budget, BudgetAssignment, CreateBankAccountRequest, CreatePayeeRequest, User,
    };
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    #[sqlx::test]
    pub async fn delete_should_remove_transaction_assignment(db_pool: MySqlPool) {
        let payee_id = Uuid::new_v4();
        let bank_account_id = Uuid::new_v4();
        let transaction_id_1 = Uuid::new_v4();
        let transaction_id_2 = Uuid::new_v4();
        let budget_id_1 = Uuid::new_v4();
        let budget_id_2 = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let assignment_id_1 = Uuid::new_v4();
        let assignment_id_2 = Uuid::new_v4();
        let assignment_id_3 = Uuid::new_v4();

        db::users::create(
            &db_pool,
            User {
                id: user_id,
                name: "name".into(),
                email: "email@email.com".into(),
                pay_frequency: None,
            },
        )
        .await
        .unwrap();
        db::payees::create(
            &db_pool,
            payee_id,
            CreatePayeeRequest {
                user_id,
                name: "payee".into(),
            },
        )
        .await
        .unwrap();
        db::bank_accounts::create(
            &db_pool,
            bank_account_id,
            CreateBankAccountRequest {
                name: "name".into(),
                user_id,
                initial_amount: Decimal::ZERO,
            },
        )
        .await
        .unwrap();
        let budget_1 = Budget {
            id: budget_id_1,
            name: "budget 1".into(),
            user_id,
            target: None,
            assignments: vec![],
        };
        let mut budget_2 = Budget {
            id: budget_id_2,
            name: "budget 2".into(),
            user_id,
            target: None,
            assignments: vec![],
        };
        db::budgets::create(&db_pool, budget_1).await.unwrap();
        db::budgets::create(&db_pool, budget_2.clone())
            .await
            .unwrap();
        db::transactions::create(
            &db_pool,
            Transaction {
                id: transaction_id_1,
                amount: Decimal::ZERO,
                bank_account_id,
                payee_id,
                budget_id: budget_id_2,
                date: NaiveDate::from_ymd_opt(2024, 12, 1).unwrap(),
            },
        )
        .await
        .unwrap();
        db::transactions::create(
            &db_pool,
            Transaction {
                id: transaction_id_2,
                amount: Decimal::ZERO,
                bank_account_id,
                payee_id,
                budget_id: budget_id_2,
                date: NaiveDate::from_ymd_opt(2024, 12, 1).unwrap(),
            },
        )
        .await
        .unwrap();
        budget_2.assignments.extend([
            BudgetAssignment {
                id: assignment_id_1,
                date: NaiveDate::from_ymd_opt(2024, 12, 1).unwrap(),
                amount: Decimal::ZERO,
                source: BudgetAssignmentSource::OtherBudget {
                    from_budget_id: budget_id_1,
                    link_id: Uuid::new_v4(),
                },
            },
            BudgetAssignment {
                id: assignment_id_2,
                date: NaiveDate::from_ymd_opt(2024, 12, 1).unwrap(),
                amount: Decimal::ZERO,
                source: BudgetAssignmentSource::Transaction {
                    from_transaction_id: transaction_id_1,
                },
            },
            BudgetAssignment {
                id: assignment_id_3,
                date: NaiveDate::from_ymd_opt(2024, 12, 1).unwrap(),
                amount: Decimal::ZERO,
                source: BudgetAssignmentSource::Transaction {
                    from_transaction_id: transaction_id_2,
                },
            },
        ]);
        db::budgets::update(&db_pool, budget_2.clone())
            .await
            .unwrap();

        delete(State(db_pool.clone()), Path(transaction_id_1))
            .await
            .unwrap();

        let fetched_transaction = db::transactions::get_single(&db_pool, transaction_id_1).await;
        assert!(matches!(fetched_transaction, Err(db::Error::NotFound)));

        let mut fetched_budget = db::budgets::get_single(&db_pool, budget_id_2)
            .await
            .unwrap();

        budget_2.assignments.remove(1);
        budget_2.assignments.sort_by_key(|a| a.id);
        fetched_budget.assignments.sort_by_key(|a| a.id);

        assert_eq!(fetched_budget, budget_2);
    }

    #[sqlx::test]
    pub async fn update_should_update_transaction_assignment(db_pool: MySqlPool) {
        let payee_id = Uuid::new_v4();
        let bank_account_id = Uuid::new_v4();
        let transaction_id_1 = Uuid::new_v4();
        let transaction_id_2 = Uuid::new_v4();
        let budget_id_1 = Uuid::new_v4();
        let budget_id_2 = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let assignment_id_1 = Uuid::new_v4();
        let assignment_id_2 = Uuid::new_v4();
        let assignment_id_3 = Uuid::new_v4();

        db::users::create(
            &db_pool,
            User {
                id: user_id,
                name: "name".into(),
                email: "email@email.com".into(),
                pay_frequency: None,
            },
        )
        .await
        .unwrap();
        db::payees::create(
            &db_pool,
            payee_id,
            CreatePayeeRequest {
                user_id,
                name: "payee".into(),
            },
        )
        .await
        .unwrap();
        db::bank_accounts::create(
            &db_pool,
            bank_account_id,
            CreateBankAccountRequest {
                name: "name".into(),
                user_id,
                initial_amount: Decimal::ZERO,
            },
        )
        .await
        .unwrap();
        let budget_1 = Budget {
            id: budget_id_1,
            name: "budget 1".into(),
            user_id,
            target: None,
            assignments: vec![],
        };
        let mut budget_2 = Budget {
            id: budget_id_2,
            name: "budget 2".into(),
            user_id,
            target: None,
            assignments: vec![],
        };
        db::budgets::create(&db_pool, budget_1).await.unwrap();
        db::budgets::create(&db_pool, budget_2.clone())
            .await
            .unwrap();
        db::transactions::create(
            &db_pool,
            Transaction {
                id: transaction_id_1,
                amount: Decimal::ZERO,
                bank_account_id,
                payee_id,
                budget_id: budget_id_2,
                date: NaiveDate::from_ymd_opt(2024, 12, 1).unwrap(),
            },
        )
        .await
        .unwrap();
        db::transactions::create(
            &db_pool,
            Transaction {
                id: transaction_id_2,
                amount: Decimal::ZERO,
                bank_account_id,
                payee_id,
                budget_id: budget_id_2,
                date: NaiveDate::from_ymd_opt(2024, 12, 1).unwrap(),
            },
        )
        .await
        .unwrap();
        budget_2.assignments.extend([
            BudgetAssignment {
                id: assignment_id_1,
                date: NaiveDate::from_ymd_opt(2024, 12, 1).unwrap(),
                amount: Decimal::ZERO,
                source: BudgetAssignmentSource::OtherBudget {
                    from_budget_id: budget_id_1,
                    link_id: Uuid::new_v4(),
                },
            },
            BudgetAssignment {
                id: assignment_id_2,
                date: NaiveDate::from_ymd_opt(2024, 12, 1).unwrap(),
                amount: Decimal::ZERO,
                source: BudgetAssignmentSource::Transaction {
                    from_transaction_id: transaction_id_1,
                },
            },
            BudgetAssignment {
                id: assignment_id_3,
                date: NaiveDate::from_ymd_opt(2024, 12, 1).unwrap(),
                amount: Decimal::ZERO,
                source: BudgetAssignmentSource::Transaction {
                    from_transaction_id: transaction_id_2,
                },
            },
        ]);
        db::budgets::update(&db_pool, budget_2.clone())
            .await
            .unwrap();

        update(
            State(db_pool.clone()),
            Path(transaction_id_1),
            Json(UpdateTransactionRequest {
                amount: dec!(10),
                date: NaiveDate::from_ymd_opt(2024, 12, 2).unwrap(),
                payee_id,
                budget_id: budget_id_2,
            }),
        )
        .await
        .unwrap();

        let fetched_transaction = db::transactions::get_single(&db_pool, transaction_id_1)
            .await
            .unwrap();
        assert_eq!(
            fetched_transaction,
            Transaction {
                id: transaction_id_1,
                amount: dec!(10),
                date: NaiveDate::from_ymd_opt(2024, 12, 2).unwrap(),
                payee_id,
                budget_id: budget_id_2,
                bank_account_id
            }
        );

        let mut fetched_budget = db::budgets::get_single(&db_pool, budget_id_2)
            .await
            .unwrap();

        let assignment = &mut budget_2.assignments[1];
        assignment.amount = dec!(10);
        assignment.date = NaiveDate::from_ymd_opt(2024, 12, 2).unwrap();
        
        fetched_budget.assignments.sort_by_key(|x| x.id);
        budget_2.assignments.sort_by_key(|x| x.id);

        assert_eq!(fetched_budget, budget_2);
    }

    #[sqlx::test]
    pub async fn update_should_remove_assignment_when_budget_changes(db_pool: MySqlPool) {
        let payee_id = Uuid::new_v4();
        let bank_account_id = Uuid::new_v4();
        let transaction_id_1 = Uuid::new_v4();
        let transaction_id_2 = Uuid::new_v4();
        let budget_id_1 = Uuid::new_v4();
        let budget_id_2 = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let assignment_id_1 = Uuid::new_v4();
        let assignment_id_2 = Uuid::new_v4();
        let assignment_id_3 = Uuid::new_v4();

        db::users::create(
            &db_pool,
            User {
                id: user_id,
                name: "name".into(),
                email: "email@email.com".into(),
                pay_frequency: None,
            },
        )
        .await
        .unwrap();
        db::payees::create(
            &db_pool,
            payee_id,
            CreatePayeeRequest {
                user_id,
                name: "payee".into(),
            },
        )
        .await
        .unwrap();
        db::bank_accounts::create(
            &db_pool,
            bank_account_id,
            CreateBankAccountRequest {
                name: "name".into(),
                user_id,
                initial_amount: Decimal::ZERO,
            },
        )
        .await
        .unwrap();
        let mut budget_1 = Budget {
            id: budget_id_1,
            name: "budget 1".into(),
            user_id,
            target: None,
            assignments: vec![],
        };
        let mut budget_2 = Budget {
            id: budget_id_2,
            name: "budget 2".into(),
            user_id,
            target: None,
            assignments: vec![],
        };
        db::budgets::create(&db_pool, budget_1.clone())
            .await
            .unwrap();
        db::budgets::create(&db_pool, budget_2.clone())
            .await
            .unwrap();
        db::transactions::create(
            &db_pool,
            Transaction {
                id: transaction_id_1,
                amount: Decimal::ZERO,
                bank_account_id,
                payee_id,
                budget_id: budget_id_2,
                date: NaiveDate::from_ymd_opt(2024, 12, 1).unwrap(),
            },
        )
        .await
        .unwrap();
        db::transactions::create(
            &db_pool,
            Transaction {
                id: transaction_id_2,
                amount: Decimal::ZERO,
                bank_account_id,
                payee_id,
                budget_id: budget_id_2,
                date: NaiveDate::from_ymd_opt(2024, 12, 1).unwrap(),
            },
        )
        .await
        .unwrap();
        budget_2.assignments.extend([
            BudgetAssignment {
                id: assignment_id_1,
                date: NaiveDate::from_ymd_opt(2024, 12, 1).unwrap(),
                amount: Decimal::ZERO,
                source: BudgetAssignmentSource::OtherBudget {
                    from_budget_id: budget_id_1,
                    link_id: Uuid::new_v4(),
                },
            },
            BudgetAssignment {
                id: assignment_id_2,
                date: NaiveDate::from_ymd_opt(2024, 12, 1).unwrap(),
                amount: Decimal::ZERO,
                source: BudgetAssignmentSource::Transaction {
                    from_transaction_id: transaction_id_1,
                },
            },
            BudgetAssignment {
                id: assignment_id_3,
                date: NaiveDate::from_ymd_opt(2024, 12, 1).unwrap(),
                amount: Decimal::ZERO,
                source: BudgetAssignmentSource::Transaction {
                    from_transaction_id: transaction_id_2,
                },
            },
        ]);
        db::budgets::update(&db_pool, budget_2.clone())
            .await
            .unwrap();

        update(
            State(db_pool.clone()),
            Path(transaction_id_1),
            Json(UpdateTransactionRequest {
                amount: dec!(10),
                date: NaiveDate::from_ymd_opt(2024, 12, 2).unwrap(),
                payee_id,
                budget_id: budget_id_1,
            }),
        )
        .await
        .unwrap();

        let fetched_transaction = db::transactions::get_single(&db_pool, transaction_id_1)
            .await
            .unwrap();
        assert_eq!(
            fetched_transaction,
            Transaction {
                id: transaction_id_1,
                amount: dec!(10),
                date: NaiveDate::from_ymd_opt(2024, 12, 2).unwrap(),
                payee_id,
                budget_id: budget_id_1,
                bank_account_id
            }
        );

        let mut fetched_budget_1 = db::budgets::get_single(&db_pool, budget_id_1)
            .await
            .unwrap();
        let mut fetched_budget_2 = db::budgets::get_single(&db_pool, budget_id_2)
            .await
            .unwrap();

        budget_2.assignments.remove(1);
        budget_1.assignments.push(BudgetAssignment {
            id: Uuid::nil(),
            amount: dec!(10),
            date: NaiveDate::from_ymd_opt(2024, 12, 2).unwrap(),
            source: BudgetAssignmentSource::Transaction {
                from_transaction_id: transaction_id_1,
            },
        });

        assert_eq!(fetched_budget_1.assignments.len(), 1);
        fetched_budget_1.assignments[0].id = Uuid::nil();
        
        fetched_budget_1.assignments.sort_by_key(|x| x.id);
        fetched_budget_2.assignments.sort_by_key(|x| x.id);
        budget_1.assignments.sort_by_key(|x| x.id);
        budget_2.assignments.sort_by_key(|x| x.id);

        assert_eq!(fetched_budget_1, budget_1);
        assert_eq!(fetched_budget_2, budget_2);
    }
}
