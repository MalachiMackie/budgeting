use rust_decimal::Decimal;
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::models::{BankAccount, CreateBankAccountRequest};

use super::DbError;

struct BankAccountDbModel {
    id: String,
    name: String,
    initial_amount: Decimal,
    user_id: String,
    transaction_total: Option<Decimal>,
}

impl TryFrom<BankAccountDbModel> for BankAccount {
    type Error = anyhow::Error;

    fn try_from(value: BankAccountDbModel) -> Result<Self, Self::Error> {
        let id: Uuid = value.id.parse()?;
        let user_id: Uuid = value.user_id.parse()?;

        Ok(BankAccount {
            id,
            user_id,
            initial_amount: value.initial_amount,
            name: value.name,
            balance: value.initial_amount + value.transaction_total.unwrap_or(Decimal::ZERO),
        })
    }
}

pub async fn get_bank_accounts(
    db_pool: &MySqlPool,
    user_id: Uuid,
) -> Result<Box<[BankAccount]>, DbError> {
    let bank_accounts: Vec<BankAccount> = sqlx::query_as!(
        BankAccountDbModel,
        r"
         SELECT ba.id, ba.name, ba.initial_amount, ba.user_id, SUM(t.amount) as transaction_total
         FROM BankAccounts ba
         LEFT JOIN Transactions t ON ba.id = t.bank_account_id
         WHERE user_id = ?
         GROUP BY ba.id, ba.name, ba.initial_amount, ba.user_id",
        user_id.as_simple()
    )
    .fetch_all(db_pool)
    .await?
    .into_iter()
    .map(|bank_account| bank_account.try_into().unwrap())
    .collect();

    Ok(bank_accounts.into_boxed_slice())
}

pub async fn get_bank_account(
    db_pool: &MySqlPool,
    account_id: Uuid,
    user_id: Uuid,
) -> Result<BankAccount, DbError> {
    sqlx::query_as!(
        BankAccountDbModel,
        r"
        SELECT ba.id, ba.name, ba.initial_amount, ba.user_id, SUM(t.amount) as transaction_total
         FROM BankAccounts ba
         LEFT JOIN Transactions t ON ba.id = t.bank_account_id
         WHERE user_id = ?
         AND ba.id = ?
         GROUP BY ba.id, ba.name, ba.initial_amount, ba.user_id",
        user_id.as_simple(),
        account_id.as_simple()
    )
    .fetch_optional(db_pool)
    .await?
    .map(|account| account.try_into().unwrap())
    .ok_or(DbError::NotFound)
}

pub async fn create_bank_account(
    db_pool: &MySqlPool,
    id: Uuid,
    request: CreateBankAccountRequest,
) -> Result<(), DbError> {
    sqlx::query!(
        "INSERT INTO BankAccounts (id, name, user_id, initial_amount) VALUE(?, ?, ?, ?)",
        id.as_simple(),
        request.name,
        request.user_id.as_simple(),
        request.initial_amount
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn delete_bank_account(db_pool: &MySqlPool, account_id: Uuid) -> Result<(), DbError> {
    sqlx::query!(
        "DELETE FROM BankAccounts WHERE id = ?",
        account_id.as_simple()
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn update_bank_account(db_pool: &MySqlPool, id: Uuid, name: &str) -> Result<(), DbError> {
    sqlx::query!(
        "UPDATE BankAccounts
    SET name = ?
    WHERE id = ?",
        name,
        id.as_simple()
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;

    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    use crate::{
        db,
        extensions::{decimal_extensions::DecimalExt, once_lock_extensions::OnceLockExt},
        models::{Budget, CreatePayeeRequest, CreateTransactionRequest, CreateUserRequest},
    };

    use super::*;

    static USER_ID: OnceLock<Uuid> = OnceLock::new();
    static BANK_ACCOUNT_ID: OnceLock<Uuid> = OnceLock::new();
    static PAYEE_ID: OnceLock<Uuid> = OnceLock::new();
    static BUDGET_ID: OnceLock<Uuid> = OnceLock::new();

    async fn test_init(db_pool: &MySqlPool) {
        let user_id = *USER_ID.init_uuid();
        let payee_id = *PAYEE_ID.init_uuid();
        let budget_id = *BUDGET_ID.init_uuid();
        _ = BANK_ACCOUNT_ID.init_uuid();

        db::users::create_user(
            db_pool,
            user_id,
            CreateUserRequest::new("name".into(), "email@email.com".into()),
        )
        .await
        .unwrap();

        db::payees::create_payee(
            db_pool,
            payee_id,
            CreatePayeeRequest::new("name".into(), user_id),
        )
        .await
        .unwrap();

        db::budgets::create_budget(
            db_pool,
            Budget::new(budget_id, "name".into(), None, user_id),
        )
        .await
        .unwrap();
    }

    #[sqlx::test]
    pub async fn test_create_and_get_all(db_pool: MySqlPool) {
        test_init(&db_pool).await;

        let bank_account_id = *BANK_ACCOUNT_ID.get().unwrap();
        let user_id = *USER_ID.get().unwrap();

        let result = create_bank_account(
            &db_pool,
            bank_account_id,
            CreateBankAccountRequest::new("Account".into(), dec!(10.3), user_id),
        )
        .await;

        let expected =
            BankAccount::new(bank_account_id, "Account".into(), dec!(0), user_id, dec!(0));
        let expected_balance = dec!(10.3);
        let expected_initial_amount = dec!(10.3);

        assert!(result.is_ok());

        let get_all = get_bank_accounts(&db_pool, user_id).await.unwrap();

        assert_eq!(get_all.len(), 1);
        let mut get_all_single = get_all[0].clone();
        let get_all_single_initial_amount = get_all_single.initial_amount;
        let get_all_single_balance = get_all_single.balance;

        get_all_single.balance = dec!(0);
        get_all_single.initial_amount = dec!(0);
        assert_eq!(get_all_single, expected);
        assert!(
            get_all_single_initial_amount.approximately_eq(expected_initial_amount, dec!(0.001))
        );
        assert!(get_all_single_balance.approximately_eq(expected_balance, dec!(0.001)));

        let mut get_single = get_bank_account(&db_pool, bank_account_id, user_id)
            .await
            .unwrap();

        let get_single_initial_amount = get_single.initial_amount;
        let get_single_balance = get_single.balance;
        get_single.balance = dec!(0);
        get_single.initial_amount = dec!(0);

        assert_eq!(get_single, expected);
        assert!(get_single_initial_amount.approximately_eq(expected_initial_amount, dec!(0.001)));
        assert!(get_single_balance.approximately_eq(expected_balance, dec!(0.001)));
    }

    #[sqlx::test]
    pub async fn test_create_and_get_single(db_pool: MySqlPool) {
        test_init(&db_pool).await;

        let bank_account_id = *BANK_ACCOUNT_ID.get().unwrap();
        let user_id = *USER_ID.get().unwrap();
        let payee_id = *PAYEE_ID.get().unwrap();
        let budget_id = *BUDGET_ID.get().unwrap();

        let result = create_bank_account(
            &db_pool,
            bank_account_id,
            CreateBankAccountRequest::new("Account".into(), dec!(10.3), user_id),
        )
        .await;

        db::transactions::create_transaction(
            &db_pool,
            Uuid::new_v4(),
            bank_account_id,
            CreateTransactionRequest::new(
                payee_id,
                dec!(3.13),
                NaiveDate::from_ymd_opt(2024, 10, 6).unwrap(),
                budget_id,
            ),
        )
        .await
        .unwrap();

        let expected =
            BankAccount::new(bank_account_id, "Account".into(), dec!(0), user_id, dec!(0));
        let expected_balance = dec!(13.43);
        let expected_initial_amount = dec!(10.3);

        assert!(result.is_ok());

        let get_all = get_bank_accounts(&db_pool, user_id).await.unwrap();

        assert_eq!(get_all.len(), 1);
        let mut get_all_single = get_all[0].clone();
        let get_all_single_initial_amount = get_all_single.initial_amount;
        let get_all_single_balance = get_all_single.balance;

        get_all_single.balance = dec!(0);
        get_all_single.initial_amount = dec!(0);
        assert_eq!(get_all_single, expected);
        assert!(
            get_all_single_initial_amount.approximately_eq(expected_initial_amount, dec!(0.001))
        );
        assert!(get_all_single_balance.approximately_eq(expected_balance, dec!(0.001)));

        let mut get_single = get_bank_account(&db_pool, bank_account_id, user_id)
            .await
            .unwrap();

        let get_single_initial_amount = get_single.initial_amount;
        let get_single_balance = get_single.balance;
        get_single.balance = dec!(0);
        get_single.initial_amount = dec!(0);

        assert_eq!(get_single, expected);
        assert!(get_single_initial_amount.approximately_eq(expected_initial_amount, dec!(0.001)));
        assert!(get_single_balance.approximately_eq(expected_balance, dec!(0.001)));
    }

    #[sqlx::test]
    pub async fn test_delete(db_pool: MySqlPool) {
        test_init(&db_pool).await;

        let user_id = *USER_ID.get().unwrap();
        let id = Uuid::new_v4();

        create_bank_account(
            &db_pool,
            id,
            CreateBankAccountRequest::new("name".into(), dec!(1), user_id),
        )
        .await
        .unwrap();

        delete_bank_account(&db_pool, id).await.unwrap();

        let get_result = get_bank_account(&db_pool, id, user_id).await;

        assert!(matches!(get_result, Err(DbError::NotFound)));
    }

    #[sqlx::test]
    pub async fn test_update(db_pool: MySqlPool) {
        test_init(&db_pool).await;

        let user_id = *USER_ID.get().unwrap();
        let id = Uuid::new_v4();

        create_bank_account(
            &db_pool,
            id,
            CreateBankAccountRequest::new("name".into(), dec!(1), user_id),
        )
        .await
        .unwrap();

        let updated = BankAccount::new(id, "newName".into(), dec!(1), user_id, dec!(1));

        update_bank_account(&db_pool, id, "newName").await.unwrap();

        let get_result = get_bank_account(&db_pool, id, user_id).await.unwrap();

        assert_eq!(get_result, updated);
    }
}
