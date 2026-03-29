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
    let client_id = std::env::var("GITHUB_CLIENT_ID").expect("GITHUB_CLIENT_ID must be set");
    let client_secret =
        std::env::var("GITHUB_CLIENT_SECRET").expect("GITHUB_CLIENT_SECRET must be set");
    let redirect_url =
        std::env::var("GITHUB_REDIRECT_URL").expect("GITHUB_REDIRECT_URL must be set");

    BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        AuthUrl::new("https://github.com/login/oauth/authorize".to_string()).unwrap(),
        Some(
            TokenUrl::new("https://github.com/login/oauth/access_token".to_string()).unwrap(),
        ),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url).unwrap())
}

pub async fn authorize(jar: PrivateCookieJar) -> impl IntoResponse {
    let (auth_url, csrf_token) = client()
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("user:email".to_string()))
        .url();

    let jar = jar.add(Cookie::new("github_csrf", csrf_token.secret().clone()));
    (jar, Redirect::to(auth_url.as_str()))
}

#[derive(Deserialize)]
pub struct CallbackParams {
    code: String,
    state: String,
}

#[derive(Deserialize)]
struct GitHubUserInfo {
    id: i64,
    login: String,
    name: Option<String>,
    avatar_url: Option<String>,
}

#[derive(Deserialize)]
struct GitHubEmail {
    email: String,
    primary: bool,
    verified: bool,
}

pub async fn callback(
    Query(params): Query<CallbackParams>,
    State(pool): State<PgPool>,
    jar: PrivateCookieJar,
) -> Result<impl IntoResponse, AppError> {
    let csrf = jar
        .get("github_csrf")
        .map(|c| c.value().to_string())
        .ok_or_else(|| anyhow::anyhow!("Missing github_csrf cookie"))?;

    if csrf != params.state {
        return Err(anyhow::anyhow!("CSRF token mismatch").into());
    }

    let jar = jar.remove(Cookie::from("github_csrf"));

    let http = reqwest::Client::new();

    let token = client()
        .exchange_code(AuthorizationCode::new(params.code))
        .request_async(oauth2::reqwest::async_http_client)
        .await
        .map_err(|e| anyhow::anyhow!("Token exchange failed: {e}"))?;

    let user_info: GitHubUserInfo = http
        .get("https://api.github.com/user")
        .bearer_auth(token.access_token().secret())
        .header("User-Agent", "nokkonenshelf")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let emails: Vec<GitHubEmail> = http
        .get("https://api.github.com/user/emails")
        .bearer_auth(token.access_token().secret())
        .header("User-Agent", "nokkonenshelf")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let email = emails
        .into_iter()
        .find(|e| e.primary && e.verified)
        .map(|e| e.email)
        .ok_or_else(|| anyhow::anyhow!("No primary verified email on GitHub account"))?;

    let display_name = user_info.name.unwrap_or(user_info.login);

    let user_id = upsert_user(
        &pool,
        "github",
        &user_info.id.to_string(),
        &email,
        &display_name,
        user_info.avatar_url.as_deref(),
    )
    .await?;

    let frontend_url = std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());
    let jar = jar.add(Cookie::build(("user_id", user_id.to_string())).path("/").build());
    Ok((jar, Redirect::to(&frontend_url)))
}
