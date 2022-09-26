mod contracts;
mod logging;
mod process_control;

use axum::{http::StatusCode, routing::get, Json, Router};
use contracts::HelloResponse;
use process_control::shutdown_signal;
use std::{error::Error, net::SocketAddr};
use tower_http::trace::TraceLayer;
use tracing::{info, instrument, span, Level};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    logging::setup()?;
    let app_span = span!(Level::ERROR, "application", commit_id = "bla");
    let app_span_main = app_span.clone();
    let app = Router::new()
        .route("/", get(root))
        .layer(TraceLayer::new_for_http().make_span_with(app_span));

    app_span_main.in_scope(|| info!("Application started"));
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
    app_span_main.in_scope(|| info!("Application shutdown"));
    logging::teardown();
    Ok(())
}

#[instrument(level = "info")]
async fn root() -> Result<Json<HelloResponse>, StatusCode> {
    Ok(Json(HelloResponse {
        message: String::from("Hello world"),
    }))
}
