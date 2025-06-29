mod api;
mod utils;
mod logger;

use log::{info, LevelFilter};
use axum::{
    Router,
    routing::{get, post},
};

use crate::logger::Logger;

static LOGGER: Logger = Logger;

#[tokio::main]
async fn main() {
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(LevelFilter::Info);

    info!(target: "main", "Starting server");
    let app: Router = Router::new()
        .route("/", get(api::index::handler))
        .route("/import", post(api::import::handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await.unwrap();

    info!(target: "main", "Server is running on port 80");
    axum::serve(listener, app).await.unwrap();
}