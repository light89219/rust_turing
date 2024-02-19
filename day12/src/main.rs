use day12_api::{app, init_tracing, AppState};
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() {
    init_tracing();

    let state = AppState {
        todos: Arc::new(Mutex::new(Vec::new())),
    };

    let app = app(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("bind failed");
    println!("listening on http://127.0.0.1:3000");
    println!("OpenAPI UI: http://127.0.0.1:3000/swagger-ui");
    axum::serve(listener, app).await.expect("server failed");
}
