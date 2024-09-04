
mod payees;
mod transactions;

use axum::{
    extract::{MatchedPath, Request}, http::StatusCode, response::IntoResponse, routing::get, Router
};
use payees::{create_payee, get_payees};
use sqlx::MySqlPool;
use tower_http::trace::TraceLayer;
use tracing::info_span;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use transactions::{create_transaction, get_transactions};

#[tokio::main]
async fn main() {
    init_logger();

    dotenvy::dotenv().unwrap();

    let db_url = std::env::var("DATABASE_URL").unwrap();

    tracing::info!("Connecting to db at {db_url}");

    let connection_pool = MySqlPool::connect(&db_url)
        .await
        .expect("to be able to connect to the database");

    // build our application with a single route
    let app = Router::new()
        .route("/transactions", get(get_transactions).post(create_transaction))
        .route("/payees", get(get_payees).post(create_payee))
        .with_state(connection_pool)
        .layer(TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
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
                }));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    tracing::info!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

fn init_logger() {
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

pub enum AppError
{
    NotFound(anyhow::Error),
    InternalServerError(anyhow::Error)
}

impl<E> From<E> for AppError
    where E : Into<anyhow::Error>
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
                format!("Something went rong: {}", e)
            ),
            AppError::NotFound(e) => (
                StatusCode::NOT_FOUND,
                format!("{e}")
            )
        }.into_response()
    }
}


