pub mod github;
pub mod google;

use axum::{extract::{FromRequestParts, State}, http::{request::Parts, StatusCode}, routing::get, Json, Router};
use axum_extra::extract::cookie::{Key, PrivateCookieJar, Cookie};
use axum::extract::FromRef;
use schemars::JsonSchema;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::AppState;
use crate::error::AppError;

pub struct CurrentUser(pub Uuid);

impl aide::OperationInput for CurrentUser {}

impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
    Key: FromRef<S>,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jar = PrivateCookieJar::<Key>::from_request_parts(parts, state)
            .await
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        let user_id = jar
            .get("user_id")
            .and_then(|c| Uuid::parse_str(c.value()).ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        Ok(CurrentUser(user_id))
    }
}

#[derive(Serialize, JsonSchema)]
pub struct UserInfo {
    pub username: String,
    pub email: String,
    pub avatar_url: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/me", get(me))
        .route("/auth/google", get(google::authorize))
        .route("/auth/google/callback", get(google::callback))
        .route("/auth/github", get(github::authorize))
        .route("/auth/github/callback", get(github::callback))
        .route("/auth/logout", axum::routing::post(logout))
}

async fn me(
    CurrentUser(user_id): CurrentUser,
    State(pool): State<PgPool>,
) -> Result<Json<UserInfo>, AppError> {
    let user = sqlx::query_as!(
        UserInfo,
        "SELECT username, email, avatar_url FROM users WHERE id = $1",
        user_id
    )
    .fetch_one(&pool)
    .await?;

    Ok(Json(user))
}

async fn logout(jar: PrivateCookieJar) -> impl axum::response::IntoResponse {
    let jar = jar.remove(Cookie::build("user_id").path("/").build());
    (jar, StatusCode::NO_CONTENT)
}

/// Find or create a user from an OAuth login. Returns the user's UUID.
pub async fn upsert_user(
    pool: &PgPool,
    provider: &str,
    provider_user_id: &str,
    email: &str,
    username: &str,
    avatar_url: Option<&str>,
) -> Result<Uuid, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let existing = sqlx::query_scalar!(
        "SELECT user_id FROM oauth_accounts WHERE provider = $1 AND provider_user_id = $2",
        provider,
        provider_user_id
    )
    .fetch_optional(&mut *tx)
    .await?;

    let user_id = if let Some(id) = existing {
        sqlx::query!(
            "UPDATE users SET avatar_url = $1 WHERE id = $2",
            avatar_url,
            id
        )
        .execute(&mut *tx)
        .await?;
        id
    } else {
        let base = username.to_lowercase().replace(' ', "_");
        let unique_username = format!("{}_{}", base, &Uuid::new_v4().to_string()[..6]);

        let user_id = sqlx::query_scalar!(
            "INSERT INTO users (username, email, avatar_url) VALUES ($1, $2, $3)
             ON CONFLICT (email) DO UPDATE SET avatar_url = EXCLUDED.avatar_url
             RETURNING id",
            unique_username,
            email,
            avatar_url
        )
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query!(
            "INSERT INTO oauth_accounts (user_id, provider, provider_user_id, provider_email)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (provider, provider_user_id) DO NOTHING",
            user_id,
            provider,
            provider_user_id,
            email
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "INSERT INTO bookcases (user_id, name, position) VALUES ($1, 'My Bookcase', 0)",
            user_id
        )
        .execute(&mut *tx)
        .await?;

        user_id
    };

    tx.commit().await?;
    Ok(user_id)
}
