use rust_decimal::Decimal;
use sqlx::{prelude::FromRow, MySql, MySqlPool};
use uuid::Uuid;

use crate::models::{BankAccount, CreateBankAccountRequest};

use super::Error;

#[derive(FromRow)]
struct BankAccountDbModel {
    id: uuid::fmt::Simple,
    name: String,
    initial_amount: Decimal,
    user_id: uuid::fmt::Simple,
    transaction_total: Option<Decimal>,
}

impl TryFrom<BankAccountDbModel> for BankAccount {
    type Error = anyhow::Error;

    fn try_from(value: BankAccountDbModel) -> Result<Self, Self::Error> {
        let id: Uuid = value.id.into_uuid();
        let user_id: Uuid = value.user_id.into_uuid();

        Ok(BankAccount {
            id,
            user_id,
            initial_amount: value.initial_amount,
            name: value.name,
            balance: value.initial_amount + value.transaction_total.unwrap_or(Decimal::ZERO),
        })
    }
}

pub async fn get(db_pool: &MySqlPool, user_id: Uuid) -> Result<Box<[BankAccount]>, Error> {
    let bank_accounts: Vec<BankAccount> = sqlx::query_as::<MySql, BankAccountDbModel>(
        r"
         SELECT ba.id, ba.name, ba.initial_amount, ba.user_id, SUM(t.amount) as transaction_total
         FROM BankAccounts ba
         LEFT JOIN Transactions t ON ba.id = t.bank_account_id
         WHERE user_id = ?
         GROUP BY ba.id, ba.name, ba.initial_amount, ba.user_id",
    )
    .bind(user_id.simple())
    .fetch_all(db_pool)
    .await?
    .into_iter()
    .map(|bank_account| bank_account.try_into().unwrap())
    .collect();

    Ok(bank_accounts.into_boxed_slice())
}

pub async fn get_single(
    db_pool: &MySqlPool,
    account_id: Uuid,
    user_id: Uuid,
) -> Result<BankAccount, Error> {
    sqlx::query_as::<MySql, BankAccountDbModel>(
        r"
        SELECT ba.id, ba.name, ba.initial_amount, ba.user_id, SUM(t.amount) as transaction_total
         FROM BankAccounts ba
         LEFT JOIN Transactions t ON ba.id = t.bank_account_id
         WHERE user_id = ?
         AND ba.id = ?
         GROUP BY ba.id, ba.name, ba.initial_amount, ba.user_id",
    )
    .bind(user_id.simple())
    .bind(account_id.simple())
    .fetch_optional(db_pool)
    .await?
    .map(|account| account.try_into().unwrap())
    .ok_or(Error::NotFound)
}

pub async fn create(
    db_pool: &MySqlPool,
    id: Uuid,
    request: CreateBankAccountRequest,
) -> Result<(), Error> {
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

pub async fn delete(db_pool: &MySqlPool, account_id: Uuid) -> Result<(), Error> {
    sqlx::query!(
        "DELETE FROM BankAccounts WHERE id = ?",
        account_id.as_simple()
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn update(db_pool: &MySqlPool, id: Uuid, name: &str) -> Result<(), Error> {
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
        extensions::{decimal::DecimalExt, once_lock::OnceLockExt},
        models::{Budget, CreatePayeeRequest, CreateTransactionRequest, User},
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

        db::users::create(
            db_pool,
            User::new(user_id, "name".into(), "email@email.com".into(), None),
        )
        .await
        .unwrap();

        db::payees::create(
            db_pool,
            payee_id,
            CreatePayeeRequest::new("name".into(), user_id),
        )
        .await
        .unwrap();

        db::budgets::create(
            db_pool,
            Budget::new(budget_id, "name".into(), None, user_id, vec![]),
        )
        .await
        .unwrap();
    }

    #[sqlx::test]
    pub async fn test_create_and_get_all(db_pool: MySqlPool) {
        test_init(&db_pool).await;

        let bank_account_id = *BANK_ACCOUNT_ID.get().unwrap();
        let user_id = *USER_ID.get().unwrap();

        let result = create(
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

        let get_all = get(&db_pool, user_id).await.unwrap();

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

        let mut get_single = get_single(&db_pool, bank_account_id, user_id)
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

        let result = create(
            &db_pool,
            bank_account_id,
            CreateBankAccountRequest::new("Account".into(), dec!(10.3), user_id),
        )
        .await;

        db::transactions::create(
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

        let get_all = get(&db_pool, user_id).await.unwrap();

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

        let mut get_single = get_single(&db_pool, bank_account_id, user_id)
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

        create(
            &db_pool,
            id,
            CreateBankAccountRequest::new("name".into(), dec!(1), user_id),
        )
        .await
        .unwrap();

        delete(&db_pool, id).await.unwrap();

        let get_result = get_single(&db_pool, id, user_id).await;

        assert!(matches!(get_result, Err(Error::NotFound)));
    }

    #[sqlx::test]
    pub async fn test_update(db_pool: MySqlPool) {
        test_init(&db_pool).await;

        let user_id = *USER_ID.get().unwrap();
        let id = Uuid::new_v4();

        create(
            &db_pool,
            id,
            CreateBankAccountRequest::new("name".into(), dec!(1), user_id),
        )
        .await
        .unwrap();

        let updated = BankAccount::new(id, "newName".into(), dec!(1), user_id, dec!(1));

        update(&db_pool, id, "newName").await.unwrap();

        let get_result = get_single(&db_pool, id, user_id).await.unwrap();

        assert_eq!(get_result, updated);
    }
}
