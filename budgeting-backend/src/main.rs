use budgeting_backend::{build_swagger_ui, init_db, init_logger, new_app};

#[tokio::main]
async fn main() {
    init_logger();

    dotenvy::dotenv().unwrap();

    let connection_pool = init_db().await;

    let app = new_app(connection_pool).merge(build_swagger_ui());

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    tracing::info!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
