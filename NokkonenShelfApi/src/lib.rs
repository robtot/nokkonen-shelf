pub mod auth;
pub mod books;
pub mod bookcases;
pub mod error;
pub mod shelves;

use aide::{
    axum::{routing::get as api_get, ApiRouter},
    openapi::OpenApi,
};
use axum::{routing::get, Extension, Json, Router};
use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub cookie_key: Key,
}

impl FromRef<AppState> for PgPool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.cookie_key.clone()
    }
}

pub fn create_app(state: AppState) -> Router {
    aide::generate::extract_schemas(true);

    let mut api = OpenApi::default();

    ApiRouter::new()
        .api_route("/health", api_get(health_check))
        .merge(auth::router())
        .merge(bookcases::router())
        .merge(shelves::router())
        .merge(books::router())
        .route("/api-doc/openapi.json", get(serve_openapi))
        .finish_api(&mut api)
        .layer(Extension(Arc::new(api)))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health_check() -> &'static str {
    "OK"
}

async fn serve_openapi(Extension(api): Extension<Arc<OpenApi>>) -> Json<OpenApi> {
    Json(api.as_ref().clone())
}
