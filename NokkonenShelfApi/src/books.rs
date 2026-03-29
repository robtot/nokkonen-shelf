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
        .api_route("/api/shelves/{shelf_id}/books", get(list).post(create))
        .api_route("/api/shelves/{shelf_id}/books/reorder", put(reorder))
        .api_route("/api/bookcases/{bookcase_id}/books/reorder", put(reorder_bookcase))
        .api_route("/api/books/{id}", patch(update).delete(delete))
}

#[derive(Deserialize, JsonSchema)]
struct ShelfPath {
    shelf_id: Uuid,
}

#[derive(Deserialize, JsonSchema)]
struct BookPath {
    id: Uuid,
}

#[derive(Serialize, JsonSchema)]
struct Book {
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

async fn list(
    CurrentUser(user_id): CurrentUser,
    State(pool): State<PgPool>,
    Path(ShelfPath { shelf_id }): Path<ShelfPath>,
) -> Result<Json<Vec<Book>>, AppError> {
    let books = sqlx::query_as!(
        Book,
        "SELECT bk.id, bk.shelf_id, bk.title, bk.author, bk.color, bk.position,
                bk.open_library_url, bk.cover_url, bk.created_at
         FROM books bk
         JOIN shelves s ON s.id = bk.shelf_id
         JOIN bookcases bc ON bc.id = s.bookcase_id
         WHERE bk.shelf_id = $1 AND bc.user_id = $2
         ORDER BY bk.position",
        shelf_id,
        user_id
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(books))
}

#[derive(Deserialize, JsonSchema)]
struct CreateBook {
    title: String,
    author: String,
    color: Option<String>,
    open_library_url: Option<String>,
}

async fn create(
    CurrentUser(user_id): CurrentUser,
    State(pool): State<PgPool>,
    Path(ShelfPath { shelf_id }): Path<ShelfPath>,
    Json(body): Json<CreateBook>,
) -> Result<(StatusCode, Json<Book>), AppError> {
    let owned = sqlx::query_scalar!(
        "SELECT EXISTS(
             SELECT 1 FROM shelves s
             JOIN bookcases bc ON bc.id = s.bookcase_id
             WHERE s.id = $1 AND bc.user_id = $2
         )",
        shelf_id,
        user_id
    )
    .fetch_one(&pool)
    .await?
    .unwrap_or(false);

    if !owned {
        return Err(AppError::bad_request(anyhow::anyhow!("Shelf not found")));
    }

    let book = sqlx::query_as!(
        Book,
        "INSERT INTO books (shelf_id, title, author, color, open_library_url, position)
         VALUES ($1, $2, $3, $4, $5,
                 COALESCE((SELECT MAX(position) + 1 FROM books WHERE shelf_id = $1), 0))
         RETURNING id, shelf_id, title, author, color, position,
                   open_library_url, cover_url, created_at",
        shelf_id,
        body.title,
        body.author,
        body.color.unwrap_or_else(|| "#8B4513".to_string()),
        body.open_library_url,
    )
    .fetch_one(&pool)
    .await?;

    Ok((StatusCode::CREATED, Json(book)))
}

#[derive(Deserialize, JsonSchema)]
struct UpdateBook {
    title: Option<String>,
    author: Option<String>,
    color: Option<String>,
    open_library_url: Option<String>,
    cover_url: Option<String>,
    shelf_id: Option<Uuid>,
}

async fn update(
    CurrentUser(user_id): CurrentUser,
    State(pool): State<PgPool>,
    Path(BookPath { id }): Path<BookPath>,
    Json(body): Json<UpdateBook>,
) -> Result<Json<Book>, AppError> {
    if let Some(new_shelf_id) = body.shelf_id {
        let mut tx = pool.begin().await?;

        // Verify user owns the book
        let owned = sqlx::query_scalar!(
            "SELECT EXISTS(
                 SELECT 1 FROM books bk
                 JOIN shelves s ON s.id = bk.shelf_id
                 JOIN bookcases bc ON bc.id = s.bookcase_id
                 WHERE bk.id = $1 AND bc.user_id = $2
             )",
            id,
            user_id
        )
        .fetch_one(&mut *tx)
        .await?
        .unwrap_or(false);

        if !owned {
            return Err(AppError::bad_request(anyhow::anyhow!("Book not found")));
        }

        // Verify user owns the target shelf
        let shelf_owned = sqlx::query_scalar!(
            "SELECT EXISTS(
                 SELECT 1 FROM shelves s
                 JOIN bookcases bc ON bc.id = s.bookcase_id
                 WHERE s.id = $1 AND bc.user_id = $2
             )",
            new_shelf_id,
            user_id
        )
        .fetch_one(&mut *tx)
        .await?
        .unwrap_or(false);

        if !shelf_owned {
            return Err(AppError::bad_request(anyhow::anyhow!("Target shelf not found")));
        }

        let new_position: i32 = sqlx::query_scalar!(
            "SELECT COALESCE(MAX(position) + 1, 0) FROM books WHERE shelf_id = $1",
            new_shelf_id
        )
        .fetch_one(&mut *tx)
        .await?
        .unwrap_or(0);

        let book = sqlx::query_as!(
            Book,
            "UPDATE books SET
                 shelf_id        = $1,
                 position        = $2,
                 title           = COALESCE($3, title),
                 author          = COALESCE($4, author),
                 color           = COALESCE($5, color),
                 open_library_url = COALESCE($6, open_library_url),
                 cover_url       = COALESCE($7, cover_url)
             WHERE id = $8
             RETURNING id, shelf_id, title, author, color, position,
                       open_library_url, cover_url, created_at",
            new_shelf_id,
            new_position,
            body.title,
            body.author,
            body.color,
            body.open_library_url,
            body.cover_url,
            id,
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::bad_request(anyhow::anyhow!("Book not found")))?;

        tx.commit().await?;
        return Ok(Json(book));
    }

    let book = sqlx::query_as!(
        Book,
        "UPDATE books bk SET
             title           = COALESCE($1, bk.title),
             author          = COALESCE($2, bk.author),
             color           = COALESCE($3, bk.color),
             open_library_url = COALESCE($4, bk.open_library_url),
             cover_url       = COALESCE($5, bk.cover_url)
         FROM shelves s
         JOIN bookcases bc ON bc.id = s.bookcase_id
         WHERE bk.id = $6 AND bk.shelf_id = s.id AND bc.user_id = $7
         RETURNING bk.id, bk.shelf_id, bk.title, bk.author, bk.color, bk.position,
                   bk.open_library_url, bk.cover_url, bk.created_at",
        body.title,
        body.author,
        body.color,
        body.open_library_url,
        body.cover_url,
        id,
        user_id
    )
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::bad_request(anyhow::anyhow!("Book not found")))?;

    Ok(Json(book))
}

async fn delete(
    CurrentUser(user_id): CurrentUser,
    State(pool): State<PgPool>,
    Path(BookPath { id }): Path<BookPath>,
) -> Result<StatusCode, AppError> {
    sqlx::query!(
        "DELETE FROM books bk USING shelves s, bookcases bc
         WHERE bk.id = $1 AND bk.shelf_id = s.id AND s.bookcase_id = bc.id AND bc.user_id = $2",
        id,
        user_id
    )
    .execute(&pool)
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize, JsonSchema)]
struct BookcaseReorderPath {
    bookcase_id: Uuid,
}

#[derive(Deserialize, JsonSchema)]
struct ShelfBookIds {
    id: Uuid,
    book_ids: Vec<Uuid>,
}

#[derive(Deserialize, JsonSchema)]
struct ReorderBookcaseBooks {
    shelves: Vec<ShelfBookIds>,
}

async fn reorder_bookcase(
    CurrentUser(user_id): CurrentUser,
    State(pool): State<PgPool>,
    Path(BookcaseReorderPath { bookcase_id }): Path<BookcaseReorderPath>,
    Json(body): Json<ReorderBookcaseBooks>,
) -> Result<StatusCode, AppError> {
    let mut tx = pool.begin().await?;

    // Fetch all shelf IDs belonging to this bookcase and user
    let existing_shelf_ids: Vec<Uuid> = sqlx::query_scalar!(
        "SELECT s.id FROM shelves s
         JOIN bookcases bc ON bc.id = s.bookcase_id
         WHERE s.bookcase_id = $1 AND bc.user_id = $2",
        bookcase_id,
        user_id
    )
    .fetch_all(&mut *tx)
    .await?;

    let mut provided_shelf_ids: Vec<Uuid> = body.shelves.iter().map(|s| s.id).collect();
    let mut expected_shelf_ids = existing_shelf_ids.clone();
    provided_shelf_ids.sort();
    expected_shelf_ids.sort();

    if provided_shelf_ids != expected_shelf_ids {
        return Err(AppError::bad_request(anyhow::anyhow!(
            "Provided shelf IDs do not match the bookcase's shelves"
        )));
    }

    // Fetch all book IDs currently in the bookcase
    let existing_book_ids: Vec<Uuid> = sqlx::query_scalar!(
        "SELECT bk.id FROM books bk
         JOIN shelves s ON s.id = bk.shelf_id
         WHERE s.bookcase_id = $1",
        bookcase_id
    )
    .fetch_all(&mut *tx)
    .await?;

    let mut provided_book_ids: Vec<Uuid> = body.shelves.iter().flat_map(|s| s.book_ids.iter().copied()).collect();
    let mut expected_book_ids = existing_book_ids.clone();
    provided_book_ids.sort();
    expected_book_ids.sort();

    if provided_book_ids != expected_book_ids {
        return Err(AppError::bad_request(anyhow::anyhow!(
            "Provided book IDs do not match the bookcase's books"
        )));
    }

    for shelf in &body.shelves {
        for (position, book_id) in shelf.book_ids.iter().enumerate() {
            sqlx::query!(
                "UPDATE books SET shelf_id = $1, position = $2 WHERE id = $3",
                shelf.id,
                position as i32,
                book_id
            )
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize, JsonSchema)]
struct ReorderBooks {
    ids: Vec<Uuid>,
}

async fn reorder(
    CurrentUser(user_id): CurrentUser,
    State(pool): State<PgPool>,
    Path(ShelfPath { shelf_id }): Path<ShelfPath>,
    Json(body): Json<ReorderBooks>,
) -> Result<StatusCode, AppError> {
    let mut tx = pool.begin().await?;

    let existing_ids: Vec<Uuid> = sqlx::query_scalar!(
        "SELECT bk.id FROM books bk
         JOIN shelves s ON s.id = bk.shelf_id
         JOIN bookcases bc ON bc.id = s.bookcase_id
         WHERE bk.shelf_id = $1 AND bc.user_id = $2",
        shelf_id,
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
            "Provided IDs do not match the shelf's books exactly"
        )));
    }

    for (position, id) in body.ids.iter().enumerate() {
        sqlx::query!(
            "UPDATE books SET position = $1 WHERE id = $2",
            position as i32,
            id
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
