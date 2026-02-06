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
    unsafe {
        env::set_var("ENCRYPTION_KEY", test_key);
        env::set_var("APP_BASE_URL", "http://localhost:3000");
        env::set_var("SMTP_HOST", "localhost");
        env::set_var("SMTP_PORT", "1025");
        env::set_var("SMTP_FROM", "noreply@test.com");
        env::set_var("SMTP_STUB", "true");
    }

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

    // 7.1 Reorder Tickets
    let response = server.post("/api/tickets/reorder")
        .add_header(axum::http::header::AUTHORIZATION, format!("Bearer {}", token))
        .json(&json!({
            "ticket_ids": [ticket_id]
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
async fn test_frontend_fallback() {
    let server = setup_test_server().await;
    
    // Request a frontend route
    let response = server.get("/change-password").await;
    response.assert_status(axum::http::StatusCode::OK);
    // It should return the index.html content (we can check for a common tag if we want)
    let body = response.text();
    assert!(body.contains("<div id=\"app\"></div>"));
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

#[tokio::test]
async fn test_password_reset_flow() {
    let server = setup_test_server().await;
    let email = format!("reset-{}@example.com", Uuid::new_v4());
    let password = "oldpassword";
    let new_password = "newpassword123";

    // 1. Register and confirm
    server.post("/api/auth/register")
        .json(&json!({
            "email": email,
            "password": password,
            "password_confirmation": password
        }))
        .await;
    
    let database_url = env::var("DATABASE_URL").unwrap();
    let pool = MySqlPoolOptions::new().connect(&database_url).await.unwrap();
    sqlx::query("UPDATE users SET is_confirmed = TRUE WHERE email = ?")
        .bind(&email)
        .execute(&pool)
        .await
        .unwrap();

    // 2. Request password change
    server.post("/api/auth/change-password-request")
        .json(&json!({ "email": email }))
        .await
        .assert_status(axum::http::StatusCode::OK);

    // 3. Get token from DB
    let row: (Option<String>,) = sqlx::query_as("SELECT confirmation_token FROM users WHERE email = ?")
        .bind(&email)
        .fetch_one(&pool)
        .await
        .unwrap();
    let token = row.0.expect("Token should be present");

    // 4. Execute password change
    server.post("/api/auth/change-password")
        .json(&json!({
            "token": token,
            "password": new_password,
            "password_confirmation": new_password
        }))
        .await
        .assert_status(axum::http::StatusCode::OK);

    // 5. Verify login with NEW password
    server.post("/api/auth/login")
        .json(&json!({
            "email": email,
            "password": new_password
        }))
        .await
        .assert_status(axum::http::StatusCode::OK);

    // 6. Verify login with OLD password fails
    server.post("/api/auth/login")
        .json(&json!({
            "email": email,
            "password": password
        }))
        .await
        .assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_worklog_validation() {
    let server = setup_test_server().await;
    let email = format!("worklog-{}@example.com", Uuid::new_v4());
    let password = "password123";

    // 1. Register and login
    server.post("/api/auth/register")
        .json(&json!({
            "email": email,
            "password": password,
            "password_confirmation": password
        }))
        .await;
    
    let database_url = env::var("DATABASE_URL").unwrap();
    let pool = MySqlPoolOptions::new().connect(&database_url).await.unwrap();
    sqlx::query("UPDATE users SET is_confirmed = TRUE WHERE email = ?")
        .bind(&email)
        .execute(&pool)
        .await
        .unwrap();

    let response = server.post("/api/auth/login")
        .json(&json!({
            "email": email,
            "password": password
        }))
        .await;
    let token = response.json::<serde_json::Value>()["token"].as_str().unwrap().to_string();

    // 2. Create a ticket
    let response = server.post("/api/tickets")
        .add_header(axum::http::header::AUTHORIZATION, format!("Bearer {}", token))
        .json(&json!({ "ticket_number": "TEST-1" }))
        .await;
    let ticket_id = response.json::<serde_json::Value>()["id"].as_str().unwrap().to_string();

    // 3. Attempt to submit worklog with EMPTY description
    let response = server.post(&format!("/api/tickets/{}/worklog", ticket_id))
        .add_header(axum::http::header::AUTHORIZATION, format!("Bearer {}", token))
        .json(&json!({
            "time_spent_formatted": "1h",
            "description": ""
        }))
        .await;
    
    // Should fail with 400 Bad Request
    response.assert_status(axum::http::StatusCode::BAD_REQUEST);
    assert!(response.text().contains("Validation error"));
}

#[tokio::test]
async fn test_version_endpoint() {
    let server = setup_test_server().await;
    let response = server.get("/api/version").await;
    response.assert_status(axum::http::StatusCode::OK);
    let json = response.json::<serde_json::Value>();
    assert_eq!(json["version"], "1.0.0");
    assert!(json["revision"].as_str().is_some());
}
