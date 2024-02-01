//! In-memory todo HTTP API: errors, tracing, request IDs, rate limiting, and OpenAPI.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode as HttpStatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tower_governor::{
    governor::GovernorConfigBuilder,
    key_extractor::GlobalKeyExtractor,
    GovernorLayer,
};
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

/// Shared application state (in-memory todos).
#[derive(Clone)]
pub struct AppState {
    pub todos: Arc<Mutex<Vec<Todo>>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Todo {
    pub id: Uuid,
    pub title: String,
    pub done: bool,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateTodo {
    pub title: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateTodo {
    pub title: Option<String>,
    pub done: Option<bool>,
}

/// Unified API failures: JSON body `{"error":"..."}` and matching HTTP status.
#[derive(Debug)]
pub enum ApiError {
    NotFound(String),
    BadRequest(String),
    #[allow(dead_code)]
    Conflict(String),
    #[allow(dead_code)]
    Internal,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct ErrorBody {
    pub error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            ApiError::NotFound(msg) => (HttpStatusCode::NOT_FOUND, msg),
            ApiError::BadRequest(msg) => (HttpStatusCode::BAD_REQUEST, msg),
            ApiError::Conflict(msg) => (HttpStatusCode::CONFLICT, msg),
            ApiError::Internal => (
                HttpStatusCode::INTERNAL_SERVER_ERROR,
                "internal server error".to_string(),
            ),
        };
        (status, Json(ErrorBody { error: message })).into_response()
    }
}

#[derive(Debug, Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
#[into_params(parameter_in = Query)]
pub struct ListTodosQuery {
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(OpenApi)]
#[openapi(
    info(title = "Day 12 Todo API", version = "0.1.0"),
    paths(
        list_todos,
        create_todo,
        get_todo,
        update_todo,
        delete_todo
    ),
    components(schemas(
        Todo,
        CreateTodo,
        UpdateTodo,
        ErrorBody,
        ListTodosQuery
    ))
)]
pub struct ApiDoc;

/// Build the Axum router (routes, state, middleware, Swagger UI).
pub fn app(state: AppState) -> Router {
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(1000)
            .burst_size(50_000)
            .key_extractor(GlobalKeyExtractor)
            .finish()
            .expect("governor config"),
    );

    // Axum runs the last `.layer()` first on the request. Order here: governor → set id → trace → propagate → routes.
    let api = Router::new()
        .route("/todos", get(list_todos).post(create_todo))
        .route(
            "/todos/{id}",
            get(get_todo).patch(update_todo).delete(delete_todo),
        )
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(TraceLayer::new_for_http())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid::default()))
        .layer(GovernorLayer::new(governor_conf))
        .with_state(state);

    Router::new()
        .merge(api)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}

/// Initialize `tracing_subscriber` with `RUST_LOG` or sensible defaults for this crate.
pub fn init_tracing() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| {
                "day12_api=debug,tower_http=debug,tower_governor=info".to_string()
            }),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
}

fn validate_title(s: &str) -> Result<(), ApiError> {
    if s.trim().is_empty() {
        Err(ApiError::BadRequest("title must not be empty".to_string()))
    } else {
        Ok(())
    }
}

#[utoipa::path(
    get,
    path = "/todos",
    params(ListTodosQuery),
    responses((status = 200, body = [Todo]))
)]
pub async fn list_todos(
    State(state): State<AppState>,
    Query(query): Query<ListTodosQuery>,
) -> impl IntoResponse {
    let todos = state.todos.lock().expect("todo mutex poisoned").clone();
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(todos.len());
    let paged: Vec<Todo> = todos.into_iter().skip(offset).take(limit).collect();
    Json(paged)
}

#[utoipa::path(
    post,
    path = "/todos",
    request_body = CreateTodo,
    responses(
        (status = 201, description = "Created", body = Todo),
        (status = 400, body = ErrorBody)
    )
)]
pub async fn create_todo(
    State(state): State<AppState>,
    Json(body): Json<CreateTodo>,
) -> Result<impl IntoResponse, ApiError> {
    validate_title(&body.title)?;

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
    Ok((HttpStatusCode::CREATED, Json(todo)))
}

#[utoipa::path(
    get,
    path = "/todos/{id}",
    params(
        ("id" = Uuid, Path, description = "Todo identifier")
    ),
    responses(
        (status = 200, body = Todo),
        (status = 404, body = ErrorBody)
    )
)]
pub async fn get_todo(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let todos = state.todos.lock().expect("todo mutex poisoned");
    match todos.iter().find(|t| t.id == id) {
        Some(todo) => Ok((HttpStatusCode::OK, Json(todo.clone()))),
        None => Err(ApiError::NotFound("not found".to_string())),
    }
}

#[utoipa::path(
    patch,
    path = "/todos/{id}",
    params(
        ("id" = Uuid, Path, description = "Todo identifier")
    ),
    request_body = UpdateTodo,
    responses(
        (status = 200, body = Todo),
        (status = 400, body = ErrorBody),
        (status = 404, body = ErrorBody)
    )
)]
pub async fn update_todo(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(body): Json<UpdateTodo>,
) -> Result<impl IntoResponse, ApiError> {
    let mut todos = state.todos.lock().expect("todo mutex poisoned");
    match todos.iter_mut().find(|t| t.id == id) {
        Some(todo) => {
            if let Some(title) = body.title {
                validate_title(&title)?;
                todo.title = title.trim().to_string();
            }
            if let Some(done) = body.done {
                todo.done = done;
            }
            Ok((HttpStatusCode::OK, Json(todo.clone())))
        }
        None => Err(ApiError::NotFound("not found".to_string())),
    }
}

#[utoipa::path(
    delete,
    path = "/todos/{id}",
    params(
        ("id" = Uuid, Path, description = "Todo identifier")
    ),
    responses(
        (status = 200, body = Todo),
        (status = 404, body = ErrorBody)
    )
)]
pub async fn delete_todo(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let mut todos = state.todos.lock().expect("todo mutex poisoned");
    match todos.iter().position(|t| t.id == id) {
        Some(idx) => {
            let removed = todos.remove(idx);
            Ok((HttpStatusCode::OK, Json(removed)))
        }
        None => Err(ApiError::NotFound("not found".to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use reqwest::Client;
    use serde_json::json;
    use tower::ServiceExt;

    #[tokio::test]
    async fn list_todos_oneshot_returns_200_and_json_array() {
        let state = AppState {
            todos: Arc::new(Mutex::new(Vec::new())),
        };
        let app = app(state);

        let req = Request::builder()
            .method("GET")
            .uri("/todos")
            .body(Body::empty())
            .expect("build request");

        let res = app.oneshot(req).await.expect("router oneshot");
        assert_eq!(res.status(), StatusCode::OK);
        assert!(
            res.headers().get("x-request-id").is_some(),
            "x-request-id should be set or propagated on responses"
        );

        let body = res
            .into_body()
            .collect()
            .await
            .expect("read body")
            .to_bytes();
        let todos: Vec<Todo> = serde_json::from_slice(&body).expect("json array of todos");
        assert!(todos.is_empty());
    }

    #[tokio::test]
    async fn request_id_header_is_echoed_on_response() {
        let state = AppState {
            todos: Arc::new(Mutex::new(Vec::new())),
        };
        let app = app(state);
        let custom_id = "client-req-123";

        let req = Request::builder()
            .method("GET")
            .uri("/todos")
            .header("x-request-id", custom_id)
            .body(Body::empty())
            .expect("build request");

        let res = app.oneshot(req).await.expect("router oneshot");
        assert_eq!(
            res.headers()
                .get("x-request-id")
                .and_then(|v| v.to_str().ok()),
            Some(custom_id)
        );
    }

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
    async fn create_rejects_empty_title_with_400() {
        let (base, handle) = spawn_app().await;
        let client = Client::new();

        let resp = client
            .post(format!("{}/todos", base))
            .json(&json!({ "title": "   " }))
            .send()
            .await
            .expect("send request");

        assert_eq!(resp.status(), reqwest::StatusCode::BAD_REQUEST);
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
