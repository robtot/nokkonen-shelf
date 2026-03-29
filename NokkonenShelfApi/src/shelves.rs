use aide::axum::{routing::{get, patch, put}, ApiRouter};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::CurrentUser;
use crate::error::AppError;
use crate::AppState;

pub fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/api/bookcases/{bookcase_id}/shelves", get(list).post(create))
        .api_route("/api/bookcases/{bookcase_id}/shelves/reorder", put(reorder))
        .api_route("/api/shelves/{id}", patch(update).delete(delete))
}

#[derive(Deserialize, JsonSchema)]
struct BookcasePath {
    bookcase_id: Uuid,
}

#[derive(Deserialize, JsonSchema)]
struct ShelfPath {
    id: Uuid,
}

#[derive(Serialize, JsonSchema)]
struct Shelf {
    id: Uuid,
    bookcase_id: Uuid,
    name: String,
    position: i32,
    created_at: DateTime<Utc>,
}

async fn list(
    CurrentUser(user_id): CurrentUser,
    State(pool): State<PgPool>,
    Path(BookcasePath { bookcase_id }): Path<BookcasePath>,
) -> Result<Json<Vec<Shelf>>, AppError> {
    let shelves = sqlx::query_as!(
        Shelf,
        "SELECT s.id, s.bookcase_id, s.name, s.position, s.created_at
         FROM shelves s
         JOIN bookcases b ON b.id = s.bookcase_id
         WHERE s.bookcase_id = $1 AND b.user_id = $2
         ORDER BY s.position",
        bookcase_id,
        user_id
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(shelves))
}

#[derive(Deserialize, JsonSchema)]
struct CreateShelf {
    name: String,
}

async fn create(
    CurrentUser(user_id): CurrentUser,
    State(pool): State<PgPool>,
    Path(BookcasePath { bookcase_id }): Path<BookcasePath>,
    Json(body): Json<CreateShelf>,
) -> Result<(StatusCode, Json<Shelf>), AppError> {
    // Verify the bookcase belongs to the user
    let owned = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM bookcases WHERE id = $1 AND user_id = $2)",
        bookcase_id,
        user_id
    )
    .fetch_one(&pool)
    .await?
    .unwrap_or(false);

    if !owned {
        return Err(AppError::bad_request(anyhow::anyhow!("Bookcase not found")));
    }

    let shelf = sqlx::query_as!(
        Shelf,
        "INSERT INTO shelves (bookcase_id, name, position)
         VALUES ($1, $2, COALESCE((SELECT MAX(position) + 1 FROM shelves WHERE bookcase_id = $1), 0))
         RETURNING id, bookcase_id, name, position, created_at",
        bookcase_id,
        body.name
    )
    .fetch_one(&pool)
    .await?;

    Ok((StatusCode::CREATED, Json(shelf)))
}

#[derive(Deserialize, JsonSchema)]
struct UpdateShelf {
    name: String,
}

async fn update(
    CurrentUser(user_id): CurrentUser,
    State(pool): State<PgPool>,
    Path(ShelfPath { id }): Path<ShelfPath>,
    Json(body): Json<UpdateShelf>,
) -> Result<Json<Shelf>, AppError> {
    let shelf = sqlx::query_as!(
        Shelf,
        "UPDATE shelves s SET name = $1
         FROM bookcases b
         WHERE s.id = $2 AND s.bookcase_id = b.id AND b.user_id = $3
         RETURNING s.id, s.bookcase_id, s.name, s.position, s.created_at",
        body.name,
        id,
        user_id
    )
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::bad_request(anyhow::anyhow!("Shelf not found")))?;

    Ok(Json(shelf))
}

async fn delete(
    CurrentUser(user_id): CurrentUser,
    State(pool): State<PgPool>,
    Path(ShelfPath { id }): Path<ShelfPath>,
) -> Result<StatusCode, AppError> {
    sqlx::query!(
        "DELETE FROM shelves s USING bookcases b
         WHERE s.id = $1 AND s.bookcase_id = b.id AND b.user_id = $2",
        id,
        user_id
    )
    .execute(&pool)
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize, JsonSchema)]
struct ReorderShelves {
    ids: Vec<Uuid>,
}

async fn reorder(
    CurrentUser(user_id): CurrentUser,
    State(pool): State<PgPool>,
    Path(BookcasePath { bookcase_id }): Path<BookcasePath>,
    Json(body): Json<ReorderShelves>,
) -> Result<StatusCode, AppError> {
    let mut tx = pool.begin().await?;

    let existing_ids: Vec<Uuid> = sqlx::query_scalar!(
        "SELECT s.id FROM shelves s
         JOIN bookcases b ON b.id = s.bookcase_id
         WHERE s.bookcase_id = $1 AND b.user_id = $2",
        bookcase_id,
        user_id
    )
    .fetch_all(&mut *tx)
    .await?;

    let mut provided = body.ids.clone();
    let mut existing = existing_ids.clone();
    provided.sort();
    existing.sort();

    if provided != existing {
        return Err(AppError::bad_request(anyhow::anyhow!(
            "Provided IDs do not match the bookcase's shelves exactly"
        )));
    }

    for (position, id) in body.ids.iter().enumerate() {
        sqlx::query!(
            "UPDATE shelves SET position = $1 WHERE id = $2",
            position as i32,
            id
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
