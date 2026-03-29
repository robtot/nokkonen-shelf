use axum::{
    body::Body,
    http::{Request, Response},
};
use axum_extra::extract::cookie::Key;
use cookie::{Cookie, CookieJar};
use nokkonenshelfapi::{create_app, AppState};
use serde_json::Value;
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

const TEST_COOKIE_SECRET: &[u8] =
    b"test_secret_key_must_be_at_least_64_bytes_long_for_tests_pad_pad";

pub struct TestApp {
    pub pool: PgPool,
    pub user_id: Uuid,
    #[allow(dead_code)]
    cookie_key: Key,
    pub app: axum::Router,
}

impl TestApp {
    pub async fn new(pool: PgPool) -> Self {
        let user_id = sqlx::query_scalar!(
            "INSERT INTO users (username, email) VALUES ($1, $2) RETURNING id",
            format!("testuser_{}", Uuid::new_v4()),
            format!("test_{}@example.com", Uuid::new_v4()),
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert test user");

        let cookie_key = Key::from(TEST_COOKIE_SECRET);

        let state = AppState {
            pool: pool.clone(),
            cookie_key: cookie_key.clone(),
        };

        let app = create_app(state);

        Self {
            pool,
            user_id,
            cookie_key,
            app,
        }
    }

    pub fn session_cookie_header(&self) -> String {
        // Use the `cookie` crate directly to create a privately-encrypted cookie.
        let key = cookie::Key::from(TEST_COOKIE_SECRET);
        let mut jar = CookieJar::new();
        jar.private_mut(&key)
            .add(Cookie::new("user_id", self.user_id.to_string()));
        let encrypted_value = jar.get("user_id").unwrap().value().to_string();
        format!("user_id={}", encrypted_value)
    }

    pub async fn get(&self, uri: &str) -> Response<Body> {
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .header("Cookie", self.session_cookie_header())
            .body(Body::empty())
            .unwrap();

        self.app.clone().oneshot(req).await.unwrap()
    }

    pub async fn post(&self, uri: &str, body: Value) -> Response<Body> {
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header("Cookie", self.session_cookie_header())
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        self.app.clone().oneshot(req).await.unwrap()
    }

    pub async fn patch_req(&self, uri: &str, body: Value) -> Response<Body> {
        let req = Request::builder()
            .method("PATCH")
            .uri(uri)
            .header("Cookie", self.session_cookie_header())
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        self.app.clone().oneshot(req).await.unwrap()
    }

    pub async fn put_req(&self, uri: &str, body: Value) -> Response<Body> {
        let req = Request::builder()
            .method("PUT")
            .uri(uri)
            .header("Cookie", self.session_cookie_header())
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        self.app.clone().oneshot(req).await.unwrap()
    }

    pub async fn delete_req(&self, uri: &str) -> Response<Body> {
        let req = Request::builder()
            .method("DELETE")
            .uri(uri)
            .header("Cookie", self.session_cookie_header())
            .body(Body::empty())
            .unwrap();

        self.app.clone().oneshot(req).await.unwrap()
    }
}
