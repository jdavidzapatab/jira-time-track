use axum_test::TestServer;
use jira_time_track::app;
use sqlx::mysql::MySqlPoolOptions;
use std::env;
use serde_json::json;
use uuid::Uuid;

async fn setup_test_server() -> TestServer {
    dotenvy::dotenv().ok();
    // Use a valid hex key for tests
    let test_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    unsafe { env::set_var("ENCRYPTION_KEY", test_key); }

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to the database");

    // Ensure migrations are run
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let app = app(pool).await;
    TestServer::new(app).unwrap()
}

#[tokio::test]
async fn test_auth_flow() {
    let server = setup_test_server().await;
    let email = format!("test-{}@example.com", Uuid::new_v4());
    let password = "password123";

    // 1. Register
    let response = server.post("/api/auth/register")
        .json(&json!({
            "email": email,
            "password": password,
            "password_confirmation": password
        }))
        .await;
    response.assert_status(axum::http::StatusCode::CREATED);

    // 2. Manually confirm the user in DB to allow login
    let database_url = env::var("DATABASE_URL").unwrap();
    let pool = MySqlPoolOptions::new().connect(&database_url).await.unwrap();
    sqlx::query("UPDATE users SET is_confirmed = TRUE WHERE email = ?")
        .bind(&email)
        .execute(&pool)
        .await
        .unwrap();

    // 3. Login
    let response = server.post("/api/auth/login")
        .json(&json!({
            "email": email,
            "password": password
        }))
        .await;
    response.assert_status(axum::http::StatusCode::OK);
    let token = response.json::<serde_json::Value>()["token"].as_str().unwrap().to_string();

    // 4. Create a Jira Server
    let response = server.post("/api/servers")
        .add_header(axum::http::header::AUTHORIZATION, format!("Bearer {}", token))
        .json(&json!({
            "name": "Test Server",
            "url": "https://test.atlassian.net",
            "username": "testuser",
            "password": "testpassword"
        }))
        .await;
    response.assert_status(axum::http::StatusCode::OK);
    let server_id = response.json::<serde_json::Value>()["id"].as_str().unwrap().to_string();

    // 5. List Servers
    let response = server.get("/api/servers")
        .add_header(axum::http::header::AUTHORIZATION, format!("Bearer {}", token))
        .await;
    response.assert_status(axum::http::StatusCode::OK);
    assert!(response.json::<Vec<serde_json::Value>>().len() > 0);

    // 6. Create a Ticket
    let response = server.post("/api/tickets")
        .add_header(axum::http::header::AUTHORIZATION, format!("Bearer {}", token))
        .json(&json!({
            "server_id": server_id,
            "ticket_number": "PROJ-1"
        }))
        .await;
    response.assert_status(axum::http::StatusCode::OK);
    let ticket_id = response.json::<serde_json::Value>()["id"].as_str().unwrap().to_string();

    // 7. Update Ticket
    let response = server.put(&format!("/api/tickets/{}", ticket_id))
        .add_header(axum::http::header::AUTHORIZATION, format!("Bearer {}", token))
        .json(&json!({
            "server_id": server_id,
            "ticket_number": "PROJ-1",
            "ticket_summary": "Test Summary",
            "time_spent_seconds": 3600
        }))
        .await;
    response.assert_status(axum::http::StatusCode::OK);

    // 8. Delete Server (should CASCADE delete tickets)
    let response = server.delete(&format!("/api/servers/{}", server_id))
        .add_header(axum::http::header::AUTHORIZATION, format!("Bearer {}", token))
        .await;
    response.assert_status(axum::http::StatusCode::NO_CONTENT);

    // 9. Verify ticket is gone
    let response = server.get("/api/tickets")
        .add_header(axum::http::header::AUTHORIZATION, format!("Bearer {}", token))
        .await;
    assert_eq!(response.json::<Vec<serde_json::Value>>().len(), 0);
}

#[tokio::test]
async fn test_confirmation_endpoint_methods() {
    let server = setup_test_server().await;
    
    // Test GET method (as clicked from email)
    let response = server.get("/api/auth/confirm?token=nonexistent").await;
    // Should be 400 Bad Request, NOT 405 Method Not Allowed
    assert_eq!(response.status_code(), axum::http::StatusCode::BAD_REQUEST);

    // Test POST method with JSON (as called from frontend)
    let response = server.post("/api/auth/confirm")
        .json(&json!({ "token": "nonexistent" }))
        .await;
    assert_eq!(response.status_code(), axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_invalid_registration() {
    let server = setup_test_server().await;
    
    let response = server.post("/api/auth/register")
        .json(&json!({
            "email": "invalid-email",
            "password": "short",
            "password_confirmation": "mismatch"
        }))
        .await;
    response.assert_status(axum::http::StatusCode::BAD_REQUEST);
}
