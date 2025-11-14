use axum::{
    routing::{get, post},
    Router,
};
use tokio::net::TcpListener;


#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/foo", get(get_foo).post(post_foo))
        .route("/foo/bar", get(foo_bar));

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


// which calls one of these handlers
async fn root() -> String {
    String::from("hello axum")
}
async fn get_foo() -> String {
    String::from("get请求的foo")
}
async fn post_foo() -> String {
    String::from("post请求的foo")
}
async fn foo_bar() -> String {
    String::from("foo:bar")
}
