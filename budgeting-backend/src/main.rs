#![warn(clippy::pedantic)]

use std::{env::args, path::PathBuf};

use anyhow::bail;
use budgeting_backend::{build_swagger_doc, build_swagger_ui, init_db, init_logger, new_app};

#[tokio::main]
async fn main() {
    let args: Vec<_> = args().collect();

    if args.len() == 1 {
        run_server().await.unwrap();
    } else if args[1] == "gen-swagger" {
        gen_swagger(if args.len() > 2 { &args[2] } else { "" })
            .await
            .unwrap();
    } else {
        panic!("Unknown command")
    }
}

async fn gen_swagger(path_str: &str) -> Result<(), anyhow::Error> {
    let api_doc = build_swagger_doc();

    let json = api_doc.to_json()?;

    if path_str.is_empty() {
        bail!("path cannot be empty");
    }

    let mut path = PathBuf::from(path_str);

    if path.is_dir() {
        path = path.with_file_name("api-doc.json");
    }

    tokio::fs::write(path, json.as_bytes()).await?;

    Ok(())
}

async fn run_server() -> Result<(), anyhow::Error> {
    init_logger();

    dotenvy::dotenv()?;

    let connection_pool = init_db().await;

    let app = new_app(connection_pool).merge(build_swagger_ui());

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    tracing::info!("listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}
