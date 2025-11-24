use axum::Router;
use axum::routing::get;
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(|| async { "Hello, world!" }))
        .route("/:name", get(|| async { "Hello, world!" }))
}