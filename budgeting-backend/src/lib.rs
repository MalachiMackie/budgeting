#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::must_use_candidate)]
pub mod db;
pub mod extensions;
pub mod models;
mod routes;

use axum::{
    extract::{MatchedPath, Request},
    http::{HeaderValue, Method, StatusCode},
    response::IntoResponse,
    routing::{get, put},
    Router,
};
use http::header::{ACCEPT, CONTENT_TYPE};
use routes::{
    bank_accounts::{self},
    budgets::{self},
    payees::{self},
    transactions::{self},
    users::{self},
};
use sqlx::MySqlPool;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info_span;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn new_app(db_pool: MySqlPool) -> Router {
    let cors_layer = build_cors();

    Router::new()
        .route("/api/payees", get(payees::get).post(payees::create))
        .route(
            "/api/payees/:payeeId",
            put(payees::update).delete(payees::delete),
        )
        .route("/api/users", get(users::get).post(users::create))
        .route(
            "/api/users/:userId",
            get(users::get_single).put(users::update),
        )
        .route(
            "/api/bank-accounts",
            get(bank_accounts::get).post(bank_accounts::create),
        )
        .route(
            "/api/bank-accounts/:accountId",
            get(bank_accounts::get_single)
                .delete(bank_accounts::delete)
                .put(bank_accounts::update),
        )
        .route(
            "/api/bank-accounts/:bankAccountId/transactions",
            get(transactions::get).post(transactions::create),
        )
        .route(
            "/api/transactions/:transactionId",
            put(transactions::update).delete(transactions::delete),
        )
        .route("/api/budgets", get(budgets::get).post(budgets::create))
        .route(
            "/api/budgets/:budgetId",
            put(budgets::update).delete(budgets::delete),
        )
        .route(
            "/api/budgets/:budgetId/transfer-to/:otherBudgetId",
            put(budgets::transfer_between),
        )
        .with_state(db_pool)
        .layer(
            ServiceBuilder::new()
                .layer(
                    TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                        // Log the matched route's path (with placeholders not filled in).
                        // Use request.uri() or OriginalUri if you want the real path.
                        let matched_path = request
                            .extensions()
                            .get::<MatchedPath>()
                            .map(MatchedPath::as_str);

                        info_span!(
                            "http_request",
                            method = ?request.method(),
                            matched_path,
                            // some_other_field = tracing::field::Empty,
                        )
                    }),
                )
                .layer(cors_layer),
        )
}

fn build_cors() -> CorsLayer {
    let allow_origin = std::env::var("CORS_ALLOW_ORIGIN").unwrap_or("localhost".to_owned());

    CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([ACCEPT, CONTENT_TYPE])
        .allow_origin(allow_origin.parse::<HeaderValue>().unwrap())
}

#[derive(OpenApi)]
#[openapi()]
struct ApiDoc;

pub fn build_swagger_doc() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(payees::Api::openapi());
    openapi.merge(transactions::Api::openapi());
    openapi.merge(bank_accounts::Api::openapi());
    openapi.merge(users::Api::openapi());
    openapi.merge(budgets::Api::openapi());

    openapi
}

pub fn build_swagger_ui() -> SwaggerUi {
    let openapi = build_swagger_doc();

    SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi)
}

pub async fn init_db() -> MySqlPool {
    let db_url = std::env::var("DATABASE_URL").unwrap();

    tracing::info!("Connecting to db at {db_url}");

    MySqlPool::connect(&db_url)
        .await
        .expect("to be able to connect to the database")
}

pub fn init_logger() {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        // axum logs rejections from built-in extractors with the `axum::rejection`
        // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
        format!(
            "{}=debug,tower_http=trace,axum::rejection=trace,sqlx=debug",
            env!("CARGO_CRATE_NAME")
        )
        .into()
    });

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}

#[derive(Debug)]
pub enum AppError {
    NotFound(anyhow::Error),
    BadRequest(anyhow::Error),
    InternalServerError(anyhow::Error),
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(value: E) -> Self {
        Self::InternalServerError(value.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AppError::InternalServerError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Something went wrong: {e}"),
            ),
            AppError::BadRequest(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            AppError::NotFound(e) => (StatusCode::NOT_FOUND, e.to_string()),
        }
        .into_response()
    }
}
