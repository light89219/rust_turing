//! SQLite repository for todos (CRUD queries used by HTTP handlers).

use crate::{map_db_err, validate_title, ApiError, CreateTodo, Todo, UpdateTodo};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

/// Row shape stored in SQLite (`done` as integer flag).
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TodoRow {
    pub id: String,
    pub title: String,
    pub done: i64,
    pub created_at: String,
}

impl TryFrom<TodoRow> for Todo {
    type Error = ApiError;

    fn try_from(row: TodoRow) -> Result<Self, Self::Error> {
        let id = Uuid::parse_str(&row.id).map_err(|e| {
            tracing::error!(?e, id = %row.id, "invalid uuid in database");
            ApiError::Internal
        })?;
        Ok(Todo {
            id,
            title: row.title,
            done: row.done != 0,
        })
    }
}

pub async fn list_todos(
    pool: &SqlitePool,
    offset: Option<usize>,
    limit: Option<usize>,
) -> Result<Vec<Todo>, ApiError> {
    let rows = sqlx::query_as::<_, TodoRow>(
        "SELECT id, title, done, created_at FROM todos ORDER BY created_at DESC, rowid DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(map_db_err)?;

    let todos: Vec<Todo> = rows
        .into_iter()
        .map(Todo::try_from)
        .collect::<Result<_, _>>()?;

    let offset = offset.unwrap_or(0);
    let limit = limit.unwrap_or(todos.len());
    Ok(todos.into_iter().skip(offset).take(limit).collect())
}

pub async fn create_todo(pool: &SqlitePool, body: CreateTodo) -> Result<Todo, ApiError> {
    validate_title(&body.title)?;

    let id = Uuid::new_v4();
    let title = body.title.trim().to_string();
    let created_at = Utc::now().to_rfc3339();

    let row = sqlx::query_as::<_, TodoRow>(
        r#"INSERT INTO todos (id, title, done, created_at)
           VALUES (?, ?, ?, ?)
           RETURNING id, title, done, created_at"#,
    )
    .bind(id.to_string())
    .bind(&title)
    .bind(0i64)
    .bind(&created_at)
    .fetch_one(pool)
    .await
    .map_err(map_db_err)?;

    Todo::try_from(row)
}

pub async fn get_todo(pool: &SqlitePool, id: Uuid) -> Result<Todo, ApiError> {
    let row = sqlx::query_as::<_, TodoRow>(
        "SELECT id, title, done, created_at FROM todos WHERE id = ?",
    )
    .bind(id.to_string())
    .fetch_optional(pool)
    .await
    .map_err(map_db_err)?
    .ok_or_else(|| ApiError::NotFound("not found".to_string()))?;

    Todo::try_from(row)
}

pub async fn update_todo(pool: &SqlitePool, id: Uuid, body: &UpdateTodo) -> Result<Todo, ApiError> {
    let row = sqlx::query_as::<_, TodoRow>(
        "SELECT id, title, done, created_at FROM todos WHERE id = ?",
    )
    .bind(id.to_string())
    .fetch_optional(pool)
    .await
    .map_err(map_db_err)?
    .ok_or_else(|| ApiError::NotFound("not found".to_string()))?;

    let mut todo = Todo::try_from(row)?;

    if let Some(ref title) = body.title {
        validate_title(title)?;
        todo.title = title.trim().to_string();
    }
    if let Some(done) = body.done {
        todo.done = done;
    }

    let done_i64 = i64::from(todo.done);
    let result = sqlx::query("UPDATE todos SET title = ?, done = ? WHERE id = ?")
        .bind(&todo.title)
        .bind(done_i64)
        .bind(id.to_string())
        .execute(pool)
        .await
        .map_err(map_db_err)?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("not found".to_string()));
    }

    Ok(todo)
}

/// Deletes by id; returns `404` when no row matched (see plan: zero affected rows → not found).
pub async fn delete_todo(pool: &SqlitePool, id: Uuid) -> Result<Todo, ApiError> {
    let row = sqlx::query_as::<_, TodoRow>(
        "SELECT id, title, done, created_at FROM todos WHERE id = ?",
    )
    .bind(id.to_string())
    .fetch_optional(pool)
    .await
    .map_err(map_db_err)?
    .ok_or_else(|| ApiError::NotFound("not found".to_string()))?;

    let todo = Todo::try_from(row)?;
    let result = sqlx::query("DELETE FROM todos WHERE id = ?")
        .bind(id.to_string())
        .execute(pool)
        .await
        .map_err(map_db_err)?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("not found".to_string()));
    }

    Ok(todo)
}
