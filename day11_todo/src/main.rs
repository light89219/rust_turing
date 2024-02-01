use axum::{
    extract::{Path, Query, State},
    http::StatusCode as HttpStatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    todos: Arc<Mutex<Vec<Todo>>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Todo {
    id: Uuid,
    title: String,
    done: bool,
}

#[derive(Debug, Deserialize)]
struct CreateTodo {
    title: String,
}

#[derive(Debug, Deserialize)]
struct UpdateTodo {
    title: Option<String>,
    done: Option<bool>,
}

#[derive(Debug, Serialize)]
struct ApiError {
    error: String,
}

#[derive(Debug, Deserialize)]
struct ListTodosQuery {
    offset: Option<usize>,
    limit: Option<usize>,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        todos: Arc::new(Mutex::new(Vec::new())),
    };

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "day11_todo=debug,tower_http=info".to_string()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = app(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("bind failed");
    println!("listening on http://127.0.0.1:3000");
    axum::serve(listener, app).await.expect("server failed");
}

fn app(state: AppState) -> Router {
    Router::new()
        .route("/todos", get(list_todos).post(create_todo))
        .route(
            "/todos/{id}",
            get(get_todo).patch(update_todo).delete(delete_todo),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

fn is_blank(s: &str) -> bool {
    s.trim().is_empty()
}

fn validation_error(message: &str) -> axum::response::Response {
    (
        HttpStatusCode::UNPROCESSABLE_ENTITY,
        Json(ApiError {
            error: message.to_string(),
        }),
    )
        .into_response()
}

async fn list_todos(
    State(state): State<AppState>,
    Query(query): Query<ListTodosQuery>,
) -> impl IntoResponse {
    let todos = state.todos.lock().expect("todo mutex poisoned").clone();
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(todos.len());
    let paged: Vec<Todo> = todos.into_iter().skip(offset).take(limit).collect();
    Json(paged)
}

async fn create_todo(
    State(state): State<AppState>,
    Json(body): Json<CreateTodo>,
) -> impl IntoResponse {
    if is_blank(&body.title) {
        return validation_error("title must not be empty");
    }

    let todo = Todo {
        id: Uuid::new_v4(),
        title: body.title.trim().to_string(),
        done: false,
    };
    state
        .todos
        .lock()
        .expect("todo mutex poisoned")
        .push(todo.clone());
    (HttpStatusCode::CREATED, Json(todo)).into_response()
}

async fn get_todo(Path(id): Path<Uuid>, State(state): State<AppState>) -> impl IntoResponse {
    let todos = state.todos.lock().expect("todo mutex poisoned");
    match todos.iter().find(|t| t.id == id) {
        Some(todo) => (HttpStatusCode::OK, Json(todo.clone())).into_response(),
        None => (
            HttpStatusCode::NOT_FOUND,
            Json(ApiError {
                error: "not found".to_string(),
            }),
        )
            .into_response(),
    }
}

async fn update_todo(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(body): Json<UpdateTodo>,
) -> impl IntoResponse {
    let mut todos = state.todos.lock().expect("todo mutex poisoned");
    match todos.iter_mut().find(|t| t.id == id) {
        Some(todo) => {
            if let Some(title) = body.title {
                if is_blank(&title) {
                    return validation_error("title must not be empty");
                }
                todo.title = title.trim().to_string();
            }
            if let Some(done) = body.done {
                todo.done = done;
            }
            (HttpStatusCode::OK, Json(todo.clone())).into_response()
        }
        None => (
            HttpStatusCode::NOT_FOUND,
            Json(ApiError {
                error: "not found".to_string(),
            }),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Client;
    use serde_json::json;

    async fn spawn_app() -> (String, tokio::task::JoinHandle<()>) {
        let state = AppState {
            todos: Arc::new(Mutex::new(Vec::new())),
        };
        let app = app(state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test listener");
        let addr = listener.local_addr().expect("listener address");
        let base = format!("http://{}", addr);
        let handle = tokio::spawn(async move {
            axum::serve(listener, app).await.expect("test server failed");
        });
        (base, handle)
    }

    #[tokio::test]
    async fn create_rejects_empty_title_with_422() {
        let (base, handle) = spawn_app().await;
        let client = Client::new();

        let resp = client
            .post(format!("{}/todos", base))
            .json(&json!({ "title": "   " }))
            .send()
            .await
            .expect("send request");

        assert_eq!(resp.status(), reqwest::StatusCode::UNPROCESSABLE_ENTITY);
        handle.abort();
    }

    #[tokio::test]
    async fn list_supports_offset_and_limit() {
        let (base, handle) = spawn_app().await;
        let client = Client::new();
        for title in ["a", "b", "c"] {
            let _ = client
                .post(format!("{}/todos", base))
                .json(&json!({ "title": title }))
                .send()
                .await
                .expect("create todo");
        }

        let resp = client
            .get(format!("{}/todos?offset=1&limit=1", base))
            .send()
            .await
            .expect("list todos");
        assert_eq!(resp.status(), reqwest::StatusCode::OK);
        let todos: Vec<Todo> = resp.json().await.expect("decode todos");
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].title, "b");
        handle.abort();
    }
}

async fn delete_todo(Path(id): Path<Uuid>, State(state): State<AppState>) -> impl IntoResponse {
    let mut todos = state.todos.lock().expect("todo mutex poisoned");
    match todos.iter().position(|t| t.id == id) {
        Some(idx) => {
            let removed = todos.remove(idx);
            (HttpStatusCode::OK, Json(removed)).into_response()
        }
        None => (
            HttpStatusCode::NOT_FOUND,
            Json(ApiError {
                error: "not found".to_string(),
            }),
        )
            .into_response(),
    }
}
