
mod payees;
mod transactions;

use axum::{
    extract::{MatchedPath, Request}, http::{HeaderValue, Method, StatusCode}, response::IntoResponse, routing::get, Router
};
use http::header::{ACCEPT, CONTENT_TYPE};
use payees::{create_payee, get_payees};
use sqlx::MySqlPool;
use tower::ServiceBuilder;
use tower_http::{cors::{Any, CorsLayer}, services::{ServeDir, ServeFile}, trace::TraceLayer};
use tracing::info_span;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use transactions::{create_transaction, get_transactions};

#[tokio::main]
async fn main() {
    init_logger();

    dotenvy::dotenv().unwrap();

    let db_url = std::env::var("DATABASE_URL").unwrap();
    let dist_path = std::env::var("FRONTEND_DIST_PATH").unwrap();
    let allow_origin = std::env::var("CORS_ALLOW_ORIGIN").unwrap();

    tracing::info!("Connecting to db at {db_url}");

    let connection_pool = MySqlPool::connect(&db_url)
        .await
        .expect("to be able to connect to the database");

    let mut cors_layer = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([ACCEPT, CONTENT_TYPE]);

    if &allow_origin == "Any" {
        cors_layer = cors_layer.allow_origin(Any);
    } else {
        cors_layer = cors_layer.allow_origin(allow_origin.parse::<HeaderValue>().unwrap());
    }

    // build our application with a single route
    let app = Router::new()
        .route("/api/transactions", get(get_transactions).post(create_transaction))
        .route("/api/payees", get(get_payees).post(create_payee))
        .nest_service("/assets", ServeDir::new(format!("{dist_path}/assets")))
        .nest_service("/", ServeFile::new(format!("{dist_path}/index.html")))
        .with_state(connection_pool)
        .layer(ServiceBuilder::new()
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
                }))
            .layer(cors_layer)
    );

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

