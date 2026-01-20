mod handlers;
mod middleware;
mod models;
mod utils;

use axum::{
    middleware::from_fn_with_state,
    routing::{get, post, put, delete},
    Router,
};
use dotenvy::dotenv;
use sqlx::mysql::MySqlPoolOptions;
use std::env;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInit};

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            env::var("RUST_LOG").unwrap_or_else(|_| "jira_time_track=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to the database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let api_routes = Router::new()
        .nest(
            "/auth",
            Router::new()
                .route("/register", post(handlers::auth::register))
                .route("/confirm", post(handlers::auth::confirm))
                .route("/login", post(handlers::auth::login))
                .route("/change-password-request", post(handlers::auth::request_password_change)),
        )
        .nest(
            "/servers",
            Router::new()
                .route("/", get(handlers::jira_servers::list_servers).post(handlers::jira_servers::create_server))
                .route("/:id", delete(handlers::jira_servers::delete_server))
                .route("/:id/test", post(handlers::jira_servers::test_credentials))
                .layer(from_fn_with_state(pool.clone(), middleware::auth::auth)),
        )
        .nest(
            "/tickets",
            Router::new()
                .route("/", get(handlers::jira_tickets::list_tickets).post(handlers::jira_tickets::create_ticket))
                .route("/:id", put(handlers::jira_tickets::update_ticket).delete(handlers::jira_tickets::delete_ticket))
                .route("/:id/summary", get(handlers::jira_tickets::get_ticket_summary))
                .route("/:id/worklog", post(handlers::jira_tickets::submit_worklog))
                .layer(from_fn_with_state(pool.clone(), middleware::auth::auth)),
        )
        .with_state(pool);

    let app = Router::new()
        .nest("/api", api_routes)
        .fallback_service(ServeDir::new("dist").fallback(ServeDir::new("dist/index.html")))
        .layer(CorsLayer::permissive());

    let port = env::var("APP_PORT").unwrap_or_else(|_| "3000".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
    
    tracing::debug!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
