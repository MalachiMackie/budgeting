use sqlx::MySqlPool;
use uuid::Uuid;

use crate::models::{CreatePayeeRequest, Payee};

use super::Error;

#[derive(PartialEq, Debug)]
struct PayeeModel {
    id: String,
    name: String,
    user_id: String,
}

impl TryFrom<PayeeModel> for Payee {
    type Error = anyhow::Error;

    fn try_from(value: PayeeModel) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.name,
            id: value.id.parse()?,
            user_id: value.user_id.parse()?,
        })
    }
}

pub async fn get(db_pool: &MySqlPool, user_id: Uuid) -> Result<Box<[Payee]>, Error> {
    let payees: Box<[Payee]> = sqlx::query_as!(
        PayeeModel,
        "SELECT id, name, user_id FROM Payees WHERE user_id = ?",
        user_id.as_simple()
    )
    .fetch_all(db_pool)
    .await?
    .into_iter()
    .map(|payee| payee.try_into().unwrap())
    .collect();

    Ok(payees)
}

pub async fn create(
    db_pool: &MySqlPool,
    id: Uuid,
    request: CreatePayeeRequest,
) -> Result<(), Error> {
    sqlx::query!(
        "INSERT INTO Payees(id, name, user_id) VALUE (?, ?, ?)",
        id.as_simple(),
        request.name,
        request.user_id.as_simple()
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn get_single(db_pool: &MySqlPool, id: Uuid) -> Result<Payee, Error> {
    sqlx::query_as!(
        PayeeModel,
        "SELECT id, name, user_id FROM Payees WHERE id = ?",
        id.as_simple()
    )
    .fetch_one(db_pool)
    .await?
    .try_into()
    .map_err(|e| Error::MappingError { error: e })
}

pub async fn update(db_pool: &MySqlPool, payee: Payee) -> Result<(), Error> {
    sqlx::query!(
        "UPDATE Payees SET name = ? WHERE id = ?",
        payee.name,
        payee.id.as_simple()
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn delete(db_pool: &MySqlPool, id: Uuid) -> Result<(), Error> {
    sqlx::query!("DELETE FROM Payees WHERE id = ?", id.as_simple())
        .execute(db_pool)
        .await?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use crate::{db::users, models::CreateUserRequest};

    use super::*;

    static USER_ID1: LazyLock<Uuid> = LazyLock::new(Uuid::new_v4);
    static USER_ID2: LazyLock<Uuid> = LazyLock::new(Uuid::new_v4);

    async fn test_init(db_pool: &MySqlPool) {
        let user_id1 = *USER_ID1;
        let user_id2 = *USER_ID2;

        users::create(
            db_pool,
            user_id1,
            CreateUserRequest::new("name".into(), "email@email.com".into()),
        )
        .await
        .unwrap();

        users::create(
            db_pool,
            user_id2,
            CreateUserRequest::new("other name".into(), "email@email.com".into()),
        )
        .await
        .unwrap();
    }

    #[sqlx::test]
    pub async fn get_payees_test(db_pool: MySqlPool) {
        test_init(&db_pool).await;

        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let user_id1 = *USER_ID1;
        let user_id2 = *USER_ID2;

        sqlx::query!(
            "INSERT INTO Payees (id, name, user_id)
            VALUES (?, ?, ?),
                (?, ?, ?)",
            // payee 1
            id1.as_simple(),
            "name",
            user_id1.as_simple(),
            // payee 2
            id2.as_simple(),
            "other name",
            user_id2.as_simple()
        )
        .execute(&db_pool)
        .await
        .unwrap();

        let get_result = self::get(&db_pool, user_id1).await.unwrap();

        assert_eq!(
            get_result,
            vec![Payee::new(id1, "name".into(), user_id1)].into_boxed_slice()
        );
    }

    #[sqlx::test]
    pub async fn get_payee_test(db_pool: MySqlPool) {
        test_init(&db_pool).await;

        let id = Uuid::new_v4();
        let user_id = *USER_ID1;

        sqlx::query!(
            "INSERT INTO Payees (id, name, user_id) VALUE (?, ?, ?)",
            id.as_simple(),
            "name",
            user_id.as_simple()
        )
        .execute(&db_pool)
        .await
        .unwrap();

        let single_result = get_single(&db_pool, id).await.unwrap();

        assert_eq!(single_result, Payee::new(id, "name".into(), user_id));
    }

    #[sqlx::test]
    pub async fn create_test(db_pool: MySqlPool) {
        test_init(&db_pool).await;

        let id = Uuid::new_v4();
        let user_id = *USER_ID1;

        create(
            &db_pool,
            id,
            CreatePayeeRequest::new("name".into(), user_id),
        )
        .await
        .unwrap();

        let fetched = sqlx::query_as!(
            PayeeModel,
            "SELECT id, name, user_id FROM Payees WHERE id = ?",
            id.as_simple()
        )
        .fetch_one(&db_pool)
        .await
        .unwrap();

        assert_eq!(
            fetched,
            PayeeModel {
                id: id.as_simple().to_string(),
                name: "name".into(),
                user_id: user_id.as_simple().to_string()
            }
        );
    }

    #[sqlx::test]
    pub async fn update_test(db_pool: MySqlPool) {
        test_init(&db_pool).await;

        let id = Uuid::new_v4();
        let user_id = *USER_ID1;

        sqlx::query!(
            "INSERT INTO Payees (id, name, user_id) VALUE (?, ?, ?)",
            id.as_simple(),
            "name",
            user_id.as_simple()
        )
        .execute(&db_pool)
        .await
        .unwrap();

        let updated = Payee::new(id, "newName".into(), user_id);

        update(&db_pool, updated).await.unwrap();

        let fetched = sqlx::query_as!(
            PayeeModel,
            "SELECT id, name, user_id FROM Payees WHERE id = ?",
            id.as_simple()
        )
        .fetch_one(&db_pool)
        .await
        .unwrap();

        assert_eq!(
            fetched,
            PayeeModel {
                id: id.as_simple().to_string(),
                name: "newName".into(),
                user_id: user_id.as_simple().to_string()
            }
        );
    }

    #[sqlx::test]
    pub async fn delete_test(db_pool: MySqlPool) {
        test_init(&db_pool).await;

        let user_id = *USER_ID1;
        let id = Uuid::new_v4();

        sqlx::query!(
            "INSERT INTO Payees (id, name, user_id) VALUE (?, ?, ?)",
            id.as_simple(),
            "name",
            user_id.as_simple()
        )
        .execute(&db_pool)
        .await
        .unwrap();

        delete(&db_pool, id).await.unwrap();

        let fetched = sqlx::query!("SELECT COUNT(*) as count FROM Payees WHERE id = ?", id)
            .fetch_one(&db_pool)
            .await.unwrap();
        
        assert_eq!(fetched.count, 0);
    }
}
