mod handlers;
mod middleware;
mod models;
mod utils;
mod repositories;
mod services;

use axum::{
    middleware::from_fn_with_state,
    routing::{get, post, put, delete},
    Router,
};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;


pub async fn app(pool: sqlx::MySqlPool) -> Router {
    let api_routes = Router::new()
        .nest(
            "/auth",
            Router::new()
                .route("/register", post(handlers::auth::register))
                .route("/confirm", get(handlers::auth::confirm).post(handlers::auth::confirm))
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

    Router::new()
        .nest("/api", api_routes)
        .fallback_service(ServeDir::new("dist").fallback(ServeDir::new("dist/index.html")))
        .layer(CorsLayer::permissive())
}
