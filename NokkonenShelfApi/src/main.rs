use nokkonenshelfapi::{create_app, AppState};
use axum_extra::extract::cookie::Key;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("nokkonenshelfapi=debug".parse().unwrap()),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let cookie_secret = std::env::var("COOKIE_SECRET").expect("COOKIE_SECRET must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let state = AppState {
        pool,
        cookie_key: Key::from(cookie_secret.as_bytes()),
    };

    let app = create_app(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
