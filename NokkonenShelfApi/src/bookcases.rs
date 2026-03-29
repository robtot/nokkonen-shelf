use aide::axum::{routing::{get, put}, ApiRouter};
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
        .api_route("/api/bookcases", get(list).post(create))
        .api_route("/api/bookcases/reorder", put(reorder))
        .api_route("/api/bookcases/{id}", get(get_detail).patch(update).delete(delete))
}

#[derive(Deserialize, JsonSchema)]
struct BookcasePath {
    id: Uuid,
}

#[derive(Serialize, JsonSchema)]
struct Bookcase {
    id: Uuid,
    name: String,
    position: i32,
    created_at: DateTime<Utc>,
}

#[derive(Serialize, JsonSchema)]
struct BookDetail {
    id: Uuid,
    shelf_id: Uuid,
    title: String,
    author: String,
    color: String,
    position: i32,
    open_library_url: Option<String>,
    cover_url: Option<String>,
    created_at: DateTime<Utc>,
}

#[derive(Serialize, JsonSchema)]
struct ShelfDetail {
    id: Uuid,
    bookcase_id: Uuid,
    name: String,
    position: i32,
    created_at: DateTime<Utc>,
    books: Vec<BookDetail>,
}

#[derive(Serialize, JsonSchema)]
struct BookcaseDetail {
    id: Uuid,
    name: String,
    position: i32,
    created_at: DateTime<Utc>,
    shelves: Vec<ShelfDetail>,
}

async fn get_detail(
    CurrentUser(user_id): CurrentUser,
    State(pool): State<PgPool>,
    Path(BookcasePath { id }): Path<BookcasePath>,
) -> Result<Json<BookcaseDetail>, AppError> {
    let bookcase = sqlx::query_as!(
        Bookcase,
        "SELECT id, name, position, created_at FROM bookcases
         WHERE id = $1 AND user_id = $2",
        id,
        user_id
    )
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::bad_request(anyhow::anyhow!("Bookcase not found")))?;

    struct ShelfRow {
        id: Uuid,
        bookcase_id: Uuid,
        name: String,
        position: i32,
        created_at: DateTime<Utc>,
    }

    let shelf_rows = sqlx::query_as!(
        ShelfRow,
        "SELECT id, bookcase_id, name, position, created_at
         FROM shelves WHERE bookcase_id = $1 ORDER BY position",
        id
    )
    .fetch_all(&pool)
    .await?;

    let shelf_ids: Vec<Uuid> = shelf_rows.iter().map(|s| s.id).collect();

    let books = sqlx::query_as!(
        BookDetail,
        "SELECT id, shelf_id, title, author, color, position,
                open_library_url, cover_url, created_at
         FROM books WHERE shelf_id = ANY($1) ORDER BY shelf_id, position",
        &shelf_ids
    )
    .fetch_all(&pool)
    .await?;

    let shelves_with_books: Vec<ShelfDetail> = shelf_rows
        .into_iter()
        .map(|s| {
            let shelf_books = books
                .iter()
                .filter(|b| b.shelf_id == s.id)
                .map(|b| BookDetail {
                    id: b.id,
                    shelf_id: b.shelf_id,
                    title: b.title.clone(),
                    author: b.author.clone(),
                    color: b.color.clone(),
                    position: b.position,
                    open_library_url: b.open_library_url.clone(),
                    cover_url: b.cover_url.clone(),
                    created_at: b.created_at,
                })
                .collect();
            ShelfDetail {
                id: s.id,
                bookcase_id: s.bookcase_id,
                name: s.name,
                position: s.position,
                created_at: s.created_at,
                books: shelf_books,
            }
        })
        .collect();

    Ok(Json(BookcaseDetail {
        id: bookcase.id,
        name: bookcase.name,
        position: bookcase.position,
        created_at: bookcase.created_at,
        shelves: shelves_with_books,
    }))
}

async fn list(
    CurrentUser(user_id): CurrentUser,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<Bookcase>>, AppError> {
    let bookcases = sqlx::query_as!(
        Bookcase,
        "SELECT id, name, position, created_at FROM bookcases
         WHERE user_id = $1 ORDER BY position",
        user_id
    )
    .fetch_all(&pool)
    .await?;

    if !bookcases.is_empty() {
        return Ok(Json(bookcases));
    }

    let bookcase = sqlx::query_as!(
        Bookcase,
        "INSERT INTO bookcases (user_id, name, position) VALUES ($1, 'My Bookcase', 0)
         RETURNING id, name, position, created_at",
        user_id
    )
    .fetch_one(&pool)
    .await?;

    Ok(Json(vec![bookcase]))
}

#[derive(Deserialize, JsonSchema)]
struct CreateBookcase {
    name: String,
}

async fn create(
    CurrentUser(user_id): CurrentUser,
    State(pool): State<PgPool>,
    Json(body): Json<CreateBookcase>,
) -> Result<(StatusCode, Json<Bookcase>), AppError> {
    let bookcase = sqlx::query_as!(
        Bookcase,
        "INSERT INTO bookcases (user_id, name, position)
         VALUES ($1, $2, COALESCE((SELECT MAX(position) + 1 FROM bookcases WHERE user_id = $1), 0))
         RETURNING id, name, position, created_at",
        user_id,
        body.name
    )
    .fetch_one(&pool)
    .await?;

    Ok((StatusCode::CREATED, Json(bookcase)))
}

#[derive(Deserialize, JsonSchema)]
struct UpdateBookcase {
    name: String,
}

async fn update(
    CurrentUser(user_id): CurrentUser,
    State(pool): State<PgPool>,
    Path(BookcasePath { id }): Path<BookcasePath>,
    Json(body): Json<UpdateBookcase>,
) -> Result<Json<Bookcase>, AppError> {
    let bookcase = sqlx::query_as!(
        Bookcase,
        "UPDATE bookcases SET name = $1
         WHERE id = $2 AND user_id = $3
         RETURNING id, name, position, created_at",
        body.name,
        id,
        user_id
    )
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::from(anyhow::anyhow!("Bookcase not found")))?;

    Ok(Json(bookcase))
}

async fn delete(
    CurrentUser(user_id): CurrentUser,
    State(pool): State<PgPool>,
    Path(BookcasePath { id }): Path<BookcasePath>,
) -> Result<StatusCode, AppError> {
    sqlx::query!(
        "DELETE FROM bookcases WHERE id = $1 AND user_id = $2",
        id,
        user_id
    )
    .execute(&pool)
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize, JsonSchema)]
struct ReorderBookcases {
    ids: Vec<Uuid>,
}

async fn reorder(
    CurrentUser(user_id): CurrentUser,
    State(pool): State<PgPool>,
    Json(body): Json<ReorderBookcases>,
) -> Result<StatusCode, AppError> {
    let mut tx = pool.begin().await?;

    let existing_ids: Vec<Uuid> = sqlx::query_scalar!(
        "SELECT id FROM bookcases WHERE user_id = $1",
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
            "Provided IDs do not match the user's bookcases exactly"
        )));
    }

    for (position, id) in body.ids.iter().enumerate() {
        sqlx::query!(
            "UPDATE bookcases SET position = $1 WHERE id = $2 AND user_id = $3",
            position as i32,
            id,
            user_id
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
