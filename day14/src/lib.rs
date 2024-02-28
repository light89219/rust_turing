//! Todo HTTP API with SQLite persistence (`sqlx`), migrations, tracing, request IDs, rate limiting, and OpenAPI.

mod repo;

use axum::{
    extract::{Path, Query, Request, State},
    http::StatusCode as HttpStatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tower_governor::{
    governor::GovernorConfigBuilder,
    key_extractor::GlobalKeyExtractor,
    GovernorLayer,
};
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

/// Shared application state: SQLite connection pool.
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}

/// Runtime configuration from the environment (12-factor style).
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
}

/// Load bind address and DB URL from the environment.
///
/// `HOST` defaults to `0.0.0.0`, `PORT` to `3000`. `DATABASE_URL` must be set.
pub fn load_config_from_env() -> Result<AppConfig, String> {
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000);
    let database_url = std::env::var("DATABASE_URL").map_err(|_| {
        "DATABASE_URL is not set. Set it to your SQLite URL (e.g. sqlite://data/app.db), or copy .env.example to .env for local development.".to_string()
    })?;
    if database_url.trim().is_empty() {
        return Err("DATABASE_URL is set but empty.".to_string());
    }
    Ok(AppConfig {
        host,
        port,
        database_url,
    })
}

/// Short, non-sensitive label for logs (never log full connection strings).
pub fn database_target_for_logs(database_url: &str) -> String {
    if let Some(rest) = database_url.strip_prefix("sqlite://") {
        if rest.starts_with(":memory:") || rest == "memory" {
            return "sqlite:memory".to_string();
        }
        let path_part = rest.split('?').next().unwrap_or(rest);
        let path = if let Some(stripped) = path_part.strip_prefix('/') {
            std::path::Path::new(stripped)
        } else {
            std::path::Path::new(path_part)
        };
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            return format!("sqlite:.../{name}");
        }
        return "sqlite:configured".to_string();
    }
    "database:configured".to_string()
}

/// Wait for Ctrl+C (and SIGTERM on Unix) to begin graceful shutdown.
pub async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(err) = tokio::signal::ctrl_c().await {
            tracing::error!(%err, "failed to listen for ctrl-c");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};
        match signal(SignalKind::terminate()) {
            Ok(mut sig) => {
                sig.recv().await;
            }
            Err(err) => tracing::error!(%err, "failed to listen for SIGTERM"),
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Todo {
    pub id: Uuid,
    pub title: String,
    pub done: bool,
}

pub use repo::TodoRow;

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
    /// Validation failures (e.g. empty title) → HTTP 422.
    Validation(String),
    Conflict(String),
    Internal,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct ErrorBody {
    pub error: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct HealthStatus {
    pub status: &'static str,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            ApiError::NotFound(msg) => (HttpStatusCode::NOT_FOUND, msg),
            ApiError::BadRequest(msg) => (HttpStatusCode::BAD_REQUEST, msg),
            ApiError::Validation(msg) => (HttpStatusCode::UNPROCESSABLE_ENTITY, msg),
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
    info(title = "Day 14 Todo API", version = "0.1.0"),
    paths(
        health,
        ready,
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
        HealthStatus,
        ListTodosQuery
    ))
)]
pub struct ApiDoc;

/// Create parent directories for a file-based `sqlite://...` URL when needed.
pub fn ensure_sqlite_parent_dir(database_url: &str) -> std::io::Result<()> {
    let Some(rest) = database_url.strip_prefix("sqlite://") else {
        return Ok(());
    };
    if rest.starts_with(":memory:") || rest == "memory" {
        return Ok(());
    }
    let path_part = rest.split('?').next().unwrap_or(rest);
    // Absolute URL form: sqlite:///C:/path
    let path = if let Some(stripped) = path_part.strip_prefix('/') {
        PathBuf::from(stripped)
    } else {
        PathBuf::from(path_part)
    };
    if let Some(parent) = path.parent() {
        if parent != std::path::Path::new("") {
            std::fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

/// Open the pool, run embedded migrations, return the pool.
pub async fn connect_and_migrate(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    ensure_sqlite_parent_dir(database_url)?;
    let options = SqliteConnectOptions::from_str(database_url)?;
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options.create_if_missing(true))
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

async fn enforce_request_timeout(req: Request, next: Next) -> Response {
    match tokio::time::timeout(Duration::from_secs(30), next.run(req)).await {
        Ok(response) => response,
        Err(_) => (
            HttpStatusCode::REQUEST_TIMEOUT,
            Json(ErrorBody {
                error: "request timed out".to_string(),
            }),
        )
            .into_response(),
    }
}

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

    // Axum runs the last `.layer()` first on the request.
    // Outer stack: request timeout → governor → request id → trace → propagate → routes.
    let api = Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/todos", get(list_todos).post(create_todo))
        .route(
            "/todos/{id}",
            get(get_todo).patch(update_todo).delete(delete_todo),
        )
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(TraceLayer::new_for_http())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid::default()))
        .layer(GovernorLayer::new(governor_conf))
        .layer(middleware::from_fn(enforce_request_timeout))
        .with_state(state);

    Router::new()
        .merge(api)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}

/// Initialize `tracing_subscriber` with `RUST_LOG` or sensible defaults for this crate.
pub fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("day13_api=info,tower_http=info,tower_governor=warn,sqlx=warn")
    });
    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}

pub(crate) fn validate_title(s: &str) -> Result<(), ApiError> {
    if s.trim().is_empty() {
        Err(ApiError::Validation("title must not be empty".to_string()))
    } else {
        Ok(())
    }
}

pub(crate) fn map_db_err(e: sqlx::Error) -> ApiError {
    match &e {
        sqlx::Error::RowNotFound => ApiError::NotFound("not found".to_string()),
        sqlx::Error::Database(db) if db.is_unique_violation() => {
            ApiError::Conflict("resource already exists".to_string())
        }
        _ => {
            tracing::error!(?e, "database error");
            ApiError::Internal
        }
    }
}

#[utoipa::path(
    get,
    path = "/health",
    responses((status = 200, body = HealthStatus))
)]
pub async fn health() -> Json<HealthStatus> {
    Json(HealthStatus { status: "ok" })
}

#[utoipa::path(
    get,
    path = "/ready",
    responses(
        (status = 200, body = HealthStatus, description = "Database reachable"),
        (status = 503, body = ErrorBody, description = "Database unavailable")
    )
)]
pub async fn ready(State(state): State<AppState>) -> impl IntoResponse {
    match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.pool)
        .await
    {
        Ok(_) => (
            HttpStatusCode::OK,
            Json(HealthStatus {
                status: "ready",
            }),
        )
            .into_response(),
        Err(e) => {
            tracing::warn!(error = %e, "readiness check failed");
            (
                HttpStatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorBody {
                    error: "database unavailable".to_string(),
                }),
            )
                .into_response()
        }
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
) -> Result<impl IntoResponse, ApiError> {
    let paged = repo::list_todos(&state.pool, query.offset, query.limit).await?;
    Ok(Json(paged))
}

#[utoipa::path(
    post,
    path = "/todos",
    request_body = CreateTodo,
    responses(
        (status = 201, description = "Created", body = Todo),
        (status = 422, body = ErrorBody)
    )
)]
pub async fn create_todo(
    State(state): State<AppState>,
    Json(body): Json<CreateTodo>,
) -> Result<impl IntoResponse, ApiError> {
    let todo = repo::create_todo(&state.pool, body).await?;
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
    let todo = repo::get_todo(&state.pool, id).await?;
    Ok((HttpStatusCode::OK, Json(todo)))
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
        (status = 422, body = ErrorBody),
        (status = 404, body = ErrorBody)
    )
)]
pub async fn update_todo(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(body): Json<UpdateTodo>,
) -> Result<impl IntoResponse, ApiError> {
    let todo = repo::update_todo(&state.pool, id, &body).await?;
    Ok((HttpStatusCode::OK, Json(todo)))
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
    let removed = repo::delete_todo(&state.pool, id).await?;
    Ok((HttpStatusCode::OK, Json(removed)))
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

    async fn test_pool() -> SqlitePool {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("test.db");
        let url = format!(
            "sqlite:///{}",
            path.display().to_string().replace('\\', "/")
        );
        connect_and_migrate(&url).await.expect("connect test db")
    }

    #[tokio::test]
    async fn migrations_apply_on_empty_sqlite_database() {
        let pool = test_pool().await;
        let exists: String = sqlx::query_scalar(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'todos'",
        )
        .fetch_one(&pool)
        .await
        .expect("todos table should exist after migrate");
        assert_eq!(exists, "todos");

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM todos")
            .fetch_one(&pool)
            .await
            .expect("count");
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn health_returns_ok_json() {
        let pool = test_pool().await;
        let state = AppState { pool };
        let app = app(state);

        let req = Request::builder()
            .method("GET")
            .uri("/health")
            .body(Body::empty())
            .expect("build request");

        let res = app.oneshot(req).await.expect("router oneshot");
        assert_eq!(res.status(), StatusCode::OK);
        let body = res
            .into_body()
            .collect()
            .await
            .expect("read body")
            .to_bytes();
        let v: serde_json::Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(v["status"], "ok");
    }

    #[tokio::test]
    async fn ready_returns_200_when_db_up() {
        let pool = test_pool().await;
        let state = AppState { pool };
        let app = app(state);

        let req = Request::builder()
            .method("GET")
            .uri("/ready")
            .body(Body::empty())
            .expect("build request");

        let res = app.oneshot(req).await.expect("router oneshot");
        assert_eq!(res.status(), StatusCode::OK);
        let body = res
            .into_body()
            .collect()
            .await
            .expect("read body")
            .to_bytes();
        let v: serde_json::Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(v["status"], "ready");
    }

    #[tokio::test]
    async fn repository_create_list_get_update_delete() {
        let pool = test_pool().await;
        let created = repo::create_todo(
            &pool,
            CreateTodo {
                title: "repo item".to_string(),
            },
        )
        .await
        .expect("create");

        let listed = repo::list_todos(&pool, None, None)
            .await
            .expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, created.id);

        let got = repo::get_todo(&pool, created.id).await.expect("get");
        assert_eq!(got.title, "repo item");

        let updated = repo::update_todo(
            &pool,
            created.id,
            &UpdateTodo {
                title: None,
                done: Some(true),
            },
        )
        .await
        .expect("update");
        assert!(updated.done);

        let removed = repo::delete_todo(&pool, created.id).await.expect("delete");
        assert_eq!(removed.id, created.id);

        let err = repo::get_todo(&pool, created.id).await.expect_err("gone");
        match err {
            ApiError::NotFound(_) => {}
            other => panic!("expected NotFound, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn list_todos_oneshot_returns_200_and_json_array() {
        let pool = test_pool().await;
        let state = AppState { pool };
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
        let pool = test_pool().await;
        let state = AppState { pool };
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
        let pool = test_pool().await;
        let state = AppState { pool };
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

        assert_eq!(
            resp.status(),
            reqwest::StatusCode::UNPROCESSABLE_ENTITY
        );
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

    #[tokio::test]
    async fn crud_round_trip_persists_in_sqlite() {
        let pool = test_pool().await;
        let state = AppState { pool: pool.clone() };
        let app = app(state);

        let client = Client::new();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let addr = listener.local_addr().expect("addr");
        let base = format!("http://{}", addr);
        let serve = tokio::spawn(async move {
            axum::serve(listener, app).await.expect("serve");
        });

        let create = client
            .post(format!("{}/todos", base))
            .json(&json!({ "title": "persist me" }))
            .send()
            .await
            .expect("create");
        assert_eq!(create.status(), reqwest::StatusCode::CREATED);
        let created: Todo = create.json().await.expect("json");

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM todos WHERE id = ?")
            .bind(created.id.to_string())
            .fetch_one(&pool)
            .await
            .expect("count");
        assert_eq!(count, 1);

        let get = client
            .get(format!("{}/todos/{}", base, created.id))
            .send()
            .await
            .expect("get");
        assert_eq!(get.status(), reqwest::StatusCode::OK);

        let patch = client
            .patch(format!("{}/todos/{}", base, created.id))
            .json(&json!({ "done": true }))
            .send()
            .await
            .expect("patch");
        assert_eq!(patch.status(), reqwest::StatusCode::OK);
        let updated: Todo = patch.json().await.expect("json");
        assert!(updated.done);

        let del = client
            .delete(format!("{}/todos/{}", base, created.id))
            .send()
            .await
            .expect("delete");
        assert_eq!(del.status(), reqwest::StatusCode::OK);

        let gone = client
            .get(format!("{}/todos/{}", base, created.id))
            .send()
            .await
            .expect("get after delete");
        assert_eq!(gone.status(), reqwest::StatusCode::NOT_FOUND);

        serve.abort();
    }
}
