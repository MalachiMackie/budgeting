use anyhow::anyhow;
use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{query_as, MySqlPool};
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

use crate::AppError;

#[derive(OpenApi)]
#[openapi(paths(get_payees, create_payee), components(schemas(Payee, CreatePayeeRequest)))]
pub struct PayeesApi;

const API_TAG: &str = "Payees";

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Payee {
    id: Uuid,
    name: String,
    user_id: Uuid,
}

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

#[derive(Deserialize)]
pub struct GetPayeesQuery
{
    user_id: Uuid
}

#[utoipa::path(
    get,
    path = "/api/payees",
    responses(
        (status = OK, description = "Success", body = Box<[Payee]>, content_type = "application/json")
    ),
    params(
        ("user_id" = Uuid, Query,),
    ),
    tag = API_TAG
)]
pub async fn get_payees(
    State(connection_pool): State<MySqlPool>,
    Query(query): Query<GetPayeesQuery>,
) -> Result<Json<Box<[Payee]>>, AppError> {
    let payees = query_as!(
        PayeeModel,
        "SELECT id, name, user_id FROM Payees WHERE user_id = ?",
        query.user_id.as_simple()
    )
    .fetch_all(&connection_pool)
    .await?
    .into_iter()
    .map(|payee| payee.try_into())
    .collect::<Result<Vec<Payee>, _>>()?;

    Ok(Json(payees.into_boxed_slice()))
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreatePayeeRequest {
    name: String,
    user_id: Uuid,
}

#[utoipa::path(
    post,
    path = "/api/payees",
    responses(
        (status = OK, description = "Success", body = Uuid, content_type = "application/json")
    ),
    request_body = CreatePayeeRequest,
    tag = API_TAG
)]
pub async fn create_payee(
    State(connection_pool): State<MySqlPool>,
    Json(request): Json<CreatePayeeRequest>,
) -> Result<Json<Uuid>, AppError> {
    if request.user_id.is_nil() {
        return Err(AppError::BadRequest(anyhow!("User Id must be set")));
    }

    let id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO Payees(id, name, user_id) VALUE (?, ?, ?)",
        id.as_simple(),
        request.name,
        request.user_id.as_simple()
    )
    .execute(&connection_pool)
    .await?;

    Ok(Json(id))
}

pub async fn get_payee(id: Uuid, db_pool: &MySqlPool) -> Result<Option<Payee>, anyhow::Error> {
    let payee = sqlx::query_as!(
        PayeeModel,
        "SELECT id, name, user_id FROM Payees WHERE id = ?",
        id.as_simple()
    )
    .fetch_optional(db_pool)
    .await?;

    let Some(payee): Option<Payee> = payee.map(|p| p.try_into().unwrap()) else {
        return Ok(None);
    };

    Ok(Some(payee))
}
