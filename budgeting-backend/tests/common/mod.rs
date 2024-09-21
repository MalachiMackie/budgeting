use axum_test::TestServer;
use budgeting_backend::new_app;
use sqlx::MySqlPool;

pub async fn test_init(db_pool: MySqlPool) -> TestServer {
    let server = TestServer::new(new_app(db_pool)).unwrap();

    server
}
