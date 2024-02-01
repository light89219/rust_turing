use std::sync::{Arc, Mutex};

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};

#[tokio::main]
async fn main() {
    let counter: Arc<Mutex<i64>> = Arc::new(Mutex::new(0));

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/items/{id}", get(item_by_id))
        .route("/echo", post(echo))
        .route("/inc", post(inc))
        .route("/count", get(count))
        .with_state(counter);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("bind failed");
    println!("listening on http://127.0.0.1:3000");
    axum::serve(listener, app).await.expect("server failed");
}

async fn root() -> &'static str {
    "ok"
}

#[derive(serde::Serialize)]
struct HealthResponse {
    status: &'static str,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "healthy" })
}

#[derive(serde::Serialize)]
struct ItemResponse {
    id: String,
}

async fn item_by_id(Path(id): Path<String>) -> Json<ItemResponse> {
    Json(ItemResponse { id })
}

#[derive(serde::Deserialize, serde::Serialize)]
struct EchoBody {
    message: String,
}

async fn echo(Json(body): Json<EchoBody>) -> Json<EchoBody> {
    Json(body)
}

#[derive(serde::Serialize)]
struct CountResponse {
    count: i64,
}

async fn inc(State(counter): State<Arc<Mutex<i64>>>) -> Json<CountResponse> {
    let mut n = counter.lock().expect("counter mutex poisoned");
    *n += 1;
    Json(CountResponse { count: *n })
}

async fn count(State(counter): State<Arc<Mutex<i64>>>) -> Json<CountResponse> {
    let n = counter.lock().expect("counter mutex poisoned");
    Json(CountResponse { count: *n })
}
