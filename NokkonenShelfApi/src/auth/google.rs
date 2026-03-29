use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::cookie::{Cookie, PrivateCookieJar};
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use serde::Deserialize;
use sqlx::PgPool;
use crate::auth::upsert_user;
use crate::error::AppError;

fn client() -> BasicClient {
    let client_id = std::env::var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID must be set");
    let client_secret =
        std::env::var("GOOGLE_CLIENT_SECRET").expect("GOOGLE_CLIENT_SECRET must be set");
    let redirect_url =
        std::env::var("GOOGLE_REDIRECT_URL").expect("GOOGLE_REDIRECT_URL must be set");

    BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string()).unwrap(),
        Some(TokenUrl::new("https://oauth2.googleapis.com/token".to_string()).unwrap()),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url).unwrap())
}

pub async fn authorize(jar: PrivateCookieJar) -> impl IntoResponse {
    let (auth_url, csrf_token) = client()
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();

    let jar = jar.add(Cookie::new("google_csrf", csrf_token.secret().clone()));
    (jar, Redirect::to(auth_url.as_str()))
}

#[derive(Deserialize)]
pub struct CallbackParams {
    code: String,
    state: String,
}

#[derive(Deserialize)]
struct GoogleUserInfo {
    sub: String,
    email: String,
    name: String,
    picture: Option<String>,
}

pub async fn callback(
    Query(params): Query<CallbackParams>,
    State(pool): State<PgPool>,
    jar: PrivateCookieJar,
) -> Result<impl IntoResponse, AppError> {
    let csrf = jar
        .get("google_csrf")
        .map(|c| c.value().to_string())
        .ok_or_else(|| anyhow::anyhow!("Missing google_csrf cookie"))?;

    if csrf != params.state {
        return Err(anyhow::anyhow!("CSRF token mismatch").into());
    }

    let jar = jar.remove(Cookie::from("google_csrf"));

    let token = client()
        .exchange_code(AuthorizationCode::new(params.code))
        .request_async(oauth2::reqwest::async_http_client)
        .await
        .map_err(|e| anyhow::anyhow!("Token exchange failed: {e}"))?;

    let user_info: GoogleUserInfo = reqwest::Client::new()
        .get("https://www.googleapis.com/oauth2/v3/userinfo")
        .bearer_auth(token.access_token().secret())
        .send()
        .await?
        .json()
        .await?;

    let user_id = upsert_user(
        &pool,
        "google",
        &user_info.sub,
        &user_info.email,
        &user_info.name,
        user_info.picture.as_deref(),
    )
    .await?;

    let frontend_url = std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());
    let jar = jar.add(Cookie::build(("user_id", user_id.to_string())).path("/").build());
    Ok((jar, Redirect::to(&frontend_url)))
}
