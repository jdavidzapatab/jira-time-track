use reqwest::Client;
use crate::utils::encryption::decrypt;
use crate::models::JiraServer;
use serde_json::Value;

pub async fn test_connection(server: &JiraServer) -> Result<(), String> {
    let password = decrypt(&server.encrypted_password);
    let client = Client::new();
    let resp = client
        .get(format!("{}/rest/api/2/myself", server.url))
        .basic_auth(&server.username, Some(password))
        .send()
        .await
        .map_err(|e| format!("Failed to connect to Jira: {}", e))?;

    if resp.status().is_success() {
        Ok(())
    } else {
        Err("Failed to authenticate with Jira".to_string())
    }
}

pub async fn get_ticket_summary(server: &JiraServer, ticket_number: &str) -> Result<String, String> {
    let password = decrypt(&server.encrypted_password);
    let client = Client::new();
    let url = format!("{}/rest/api/2/issue/{}", server.url, ticket_number);
    let resp = client
        .get(&url)
        .basic_auth(&server.username, Some(&password))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch ticket from Jira: {}", e))?;

    if !resp.status().is_success() {
        return Err("Jira fetch error".to_string());
    }

    let data: Value = resp.json().await.map_err(|e| e.to_string())?;
    let summary = data["fields"]["summary"].as_str().unwrap_or("No summary found");
    Ok(summary.to_string())
}

pub async fn submit_worklog(
    server: &JiraServer,
    ticket_number: &str,
    time_spent_formatted: &str,
    description: &str,
) -> Result<(), String> {
    let password = decrypt(&server.encrypted_password);
    let client = Client::new();

    // Add Worklog
    let worklog_url = format!("{}/rest/api/2/issue/{}/worklog", server.url, ticket_number);
    let worklog_resp = client
        .post(&worklog_url)
        .basic_auth(&server.username, Some(&password))
        .json(&serde_json::json!({
            "comment": description,
            "timeSpent": time_spent_formatted
        }))
        .send()
        .await
        .map_err(|e| format!("Failed to send worklog to Jira: {}", e))?;

    if !worklog_resp.status().is_success() {
        let err_text = worklog_resp.text().await.unwrap_or_default();
        return Err(format!("Jira worklog error: {}", err_text));
    }

    // Add Comment
    let comment_url = format!("{}/rest/api/2/issue/{}/comment", server.url, ticket_number);
    let _comment_resp = client
        .post(&comment_url)
        .basic_auth(&server.username, Some(&password))
        .json(&serde_json::json!({
            "body": format!("Time spent: {}\n\n{}", time_spent_formatted, description)
        }))
        .send()
        .await;

    Ok(())
}
