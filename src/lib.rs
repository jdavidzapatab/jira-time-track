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
use tower_http::trace::TraceLayer;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use std::sync::Arc;
use crate::services::mail::MailService;
use tracing::info_span;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::MySqlPool,
    pub mail_service: Arc<MailService>,
}

pub async fn app(pool: sqlx::MySqlPool) -> Router {
    let mail_service = Arc::new(MailService::new());
    let state = AppState {
        pool,
        mail_service,
    };

    let api_routes = Router::new()
        .route("/version", get(handlers::meta::get_version))
        .nest(
            "/auth",
            Router::new()
                .route("/register", post(handlers::auth::register))
                .route("/confirm", get(handlers::auth::confirm).post(handlers::auth::confirm))
                .route("/login", post(handlers::auth::login))
                .route("/change-password-request", post(handlers::auth::request_password_change))
                .route("/change-password", post(handlers::auth::change_password)),
        )
        .nest(
            "/servers",
            Router::new()
                .route("/", get(handlers::jira_servers::list_servers).post(handlers::jira_servers::create_server))
                .route("/test-new", post(handlers::jira_servers::test_new_server_credentials))
                .route("/:id", delete(handlers::jira_servers::delete_server))
                .route("/:id/test", post(handlers::jira_servers::test_credentials))
                .layer(from_fn_with_state(state.clone(), middleware::auth::auth)),
        )
        .nest(
            "/tickets",
            Router::new()
                .route("/", get(handlers::jira_tickets::list_tickets).post(handlers::jira_tickets::create_ticket))
                .route("/reorder", post(handlers::jira_tickets::reorder_tickets))
                .route("/:id", put(handlers::jira_tickets::update_ticket).delete(handlers::jira_tickets::delete_ticket))
                .route("/:id/summary", get(handlers::jira_tickets::get_ticket_summary))
                .route("/:id/worklog", post(handlers::jira_tickets::submit_worklog))
                .layer(from_fn_with_state(state.clone(), middleware::auth::auth)),
        )
        .with_state(state);

    Router::new()
        .nest("/api", api_routes)
        .fallback_service(ServeDir::new("dist").fallback(ServeFile::new("dist/index.html")))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &axum::http::Request<_>| {
                    let path = request.uri().path();
                    info_span!(
                        "http_request",
                        method = ?request.method(),
                        path = %path,
                    )
                })
        )
        .layer(CorsLayer::permissive())
}
