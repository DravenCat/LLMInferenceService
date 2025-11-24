use axum::Router;
use std::sync::Arc;
use axum::routing::get;
use crate::model::ModelManager;

pub fn routes() -> Router<Arc<ModelManager>> {
    Router::new()
        .route("/", get(|| async { "Hello, world!" }))
        .route("/:name", get(|| async { "Hello, world!" }))
}