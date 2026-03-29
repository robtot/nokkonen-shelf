mod common;

use axum::body::to_bytes;
use axum::http::StatusCode;
use common::TestApp;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test(migrations = "./migrations")]
async fn list_bookcases_empty(pool: PgPool) {
    let app = TestApp::new(pool).await;

    let response = app.get("/api/bookcases").await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json, json!([]));
}

#[sqlx::test(migrations = "./migrations")]
async fn create_bookcase(pool: PgPool) {
    let app = TestApp::new(pool).await;

    let response = app.post("/api/bookcases", json!({"name": "My Bookcase"})).await;
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["name"], "My Bookcase");
    assert!(json["id"].is_string());
    assert_eq!(json["position"], 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn list_bookcases_returns_created(pool: PgPool) {
    let app = TestApp::new(pool).await;

    app.post("/api/bookcases", json!({"name": "First Shelf"})).await;

    let response = app.get("/api/bookcases").await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let arr = json.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["name"], "First Shelf");
}

#[sqlx::test(migrations = "./migrations")]
async fn update_bookcase(pool: PgPool) {
    let app = TestApp::new(pool).await;

    let create_resp = app.post("/api/bookcases", json!({"name": "Old Name"})).await;
    let create_body = to_bytes(create_resp.into_body(), usize::MAX).await.unwrap();
    let created: Value = serde_json::from_slice(&create_body).unwrap();
    let id = created["id"].as_str().unwrap();

    let patch_resp = app
        .patch_req(&format!("/api/bookcases/{}", id), json!({"name": "New Name"}))
        .await;
    assert_eq!(patch_resp.status(), StatusCode::OK);

    let patch_body = to_bytes(patch_resp.into_body(), usize::MAX).await.unwrap();
    let patched: Value = serde_json::from_slice(&patch_body).unwrap();
    assert_eq!(patched["name"], "New Name");
    assert_eq!(patched["id"], id);
}

#[sqlx::test(migrations = "./migrations")]
async fn delete_bookcase(pool: PgPool) {
    let app = TestApp::new(pool).await;

    let create_resp = app.post("/api/bookcases", json!({"name": "To Delete"})).await;
    let create_body = to_bytes(create_resp.into_body(), usize::MAX).await.unwrap();
    let created: Value = serde_json::from_slice(&create_body).unwrap();
    let id = created["id"].as_str().unwrap();

    let delete_resp = app.delete_req(&format!("/api/bookcases/{}", id)).await;
    assert_eq!(delete_resp.status(), StatusCode::NO_CONTENT);

    let list_resp = app.get("/api/bookcases").await;
    let list_body = to_bytes(list_resp.into_body(), usize::MAX).await.unwrap();
    let list: Value = serde_json::from_slice(&list_body).unwrap();
    assert_eq!(list, json!([]));
}

#[sqlx::test(migrations = "./migrations")]
async fn reorder_bookcases(pool: PgPool) {
    let app = TestApp::new(pool).await;

    let resp_a = app.post("/api/bookcases", json!({"name": "A"})).await;
    let body_a = to_bytes(resp_a.into_body(), usize::MAX).await.unwrap();
    let bc_a: Value = serde_json::from_slice(&body_a).unwrap();
    let id_a = bc_a["id"].as_str().unwrap().to_string();

    let resp_b = app.post("/api/bookcases", json!({"name": "B"})).await;
    let body_b = to_bytes(resp_b.into_body(), usize::MAX).await.unwrap();
    let bc_b: Value = serde_json::from_slice(&body_b).unwrap();
    let id_b = bc_b["id"].as_str().unwrap().to_string();

    // Swap order: B first, then A
    let reorder_resp = app
        .put_req("/api/bookcases/reorder", json!({"ids": [id_b, id_a]}))
        .await;
    assert_eq!(reorder_resp.status(), StatusCode::NO_CONTENT);

    let list_resp = app.get("/api/bookcases").await;
    let list_body = to_bytes(list_resp.into_body(), usize::MAX).await.unwrap();
    let list: Value = serde_json::from_slice(&list_body).unwrap();
    let arr = list.as_array().unwrap();
    assert_eq!(arr[0]["id"], id_b);
    assert_eq!(arr[1]["id"], id_a);
}

#[sqlx::test(migrations = "./migrations")]
async fn reorder_fails_with_wrong_ids(pool: PgPool) {
    let app = TestApp::new(pool).await;

    app.post("/api/bookcases", json!({"name": "A"})).await;

    let wrong_id = Uuid::new_v4().to_string();
    let resp = app
        .put_req("/api/bookcases/reorder", json!({"ids": [wrong_id]}))
        .await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "./migrations")]
async fn get_bookcase_detail(pool: PgPool) {
    let app = TestApp::new(pool).await;

    // Create a bookcase
    let create_resp = app.post("/api/bookcases", json!({"name": "Detail Bookcase"})).await;
    let create_body = to_bytes(create_resp.into_body(), usize::MAX).await.unwrap();
    let bookcase: Value = serde_json::from_slice(&create_body).unwrap();
    let bookcase_id: Uuid = bookcase["id"].as_str().unwrap().parse().unwrap();

    // Insert a shelf directly via sqlx
    let shelf_id: Uuid = sqlx::query_scalar!(
        "INSERT INTO shelves (bookcase_id, name, position) VALUES ($1, $2, 0) RETURNING id",
        bookcase_id,
        "Test Shelf"
    )
    .fetch_one(&app.pool)
    .await
    .unwrap();

    // Insert a book directly via sqlx
    sqlx::query!(
        "INSERT INTO books (shelf_id, title, author, color, position)
         VALUES ($1, $2, $3, $4, 0)",
        shelf_id,
        "Test Book",
        "Test Author",
        "#8B4513"
    )
    .execute(&app.pool)
    .await
    .unwrap();

    let resp = app.get(&format!("/api/bookcases/{}", bookcase_id)).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let detail: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(detail["name"], "Detail Bookcase");
    let shelves = detail["shelves"].as_array().unwrap();
    assert_eq!(shelves.len(), 1);
    assert_eq!(shelves[0]["name"], "Test Shelf");

    let books = shelves[0]["books"].as_array().unwrap();
    assert_eq!(books.len(), 1);
    assert_eq!(books[0]["title"], "Test Book");
    assert_eq!(books[0]["author"], "Test Author");
}

#[sqlx::test(migrations = "./migrations")]
async fn unauthenticated_request(pool: PgPool) {
    let cookie_key = axum_extra::extract::cookie::Key::from(
        b"test_secret_key_must_be_at_least_64_bytes_long_for_tests_pad_pad" as &[u8],
    );
    let state = nokkonenshelfapi::AppState {
        pool: pool.clone(),
        cookie_key,
    };
    let app = nokkonenshelfapi::create_app(state);

    use tower::ServiceExt;
    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/api/bookcases")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
