use std::sync::OnceLock;

use axum_test::{TestResponse, TestServer};
use budgeting_backend::new_app;
use http::StatusCode;
use sqlx::MySqlPool;

pub fn integration_test_init(db_pool: MySqlPool) -> TestServer {
    let server = TestServer::new(new_app(db_pool)).unwrap();

    server
}

pub trait OnceLockExt<'a, T> {
    #[allow(unused)]
    fn unwrap(&'a self) -> &'a T;
}

impl<'a, T> OnceLockExt<'a, T> for OnceLock<T> {
    fn unwrap(&'a self) -> &'a T {
        self.get().unwrap()
    }
}

#[allow(unused)]
pub trait TestResponseExt {
    fn assert_ok(&self);
    fn assert_created(&self);
    fn assert_successful(&self, expected_status_code: StatusCode);
}

impl TestResponseExt for TestResponse {
    fn assert_created(&self) {
        self.assert_successful(StatusCode::CREATED);
    }

    fn assert_ok(&self) {
        self.assert_successful(StatusCode::OK);
    }

    fn assert_successful(&self, expected_status_code: StatusCode) {
        let status_code = self.status_code();
        if status_code == expected_status_code {
            return;
        }

        if status_code.is_success() {
            panic!(
                "Expected {} {}, but found {} {}",
                expected_status_code.as_u16(),
                expected_status_code.canonical_reason().unwrap(),
                status_code.as_u16(),
                status_code.canonical_reason().unwrap()
            )
        }

        panic!(
            "Expected {} {}, but found {} {}. Body: {}",
            expected_status_code.as_u16(),
            expected_status_code.canonical_reason().unwrap(),
            status_code.as_u16(),
            status_code.canonical_reason().unwrap(),
            self.text(),
        )
    }
}
