//! Mock implementations for external services and platform components.

use httptest::{matchers::*, responders::*, Expectation, Server};
use serde_json::json;

/// Create a mock Kratos server for testing authentication flows.
pub async fn mock_kratos_server() -> Server {
    Server::run()
}

/// Mock a successful Kratos session check.
pub fn mock_kratos_session_valid(server: &Server, identity_id: &str) {
    let identity_id = identity_id.to_string();
    server.expect(
        Expectation::matching(request::method_path("GET", "/sessions/whoami"))
            .respond_with(status_code(200).body(
                json!({
                    "id": "session-123",
                    "identity": {
                        "id": identity_id,
                        "traits": {
                            "email": "test@example.com"
                        }
                    },
                    "active": true
                })
                .to_string(),
            )),
    );
}

/// Mock a failed Kratos session check (401 Unauthorized).
pub fn mock_kratos_session_invalid(server: &Server) {
    server.expect(
        Expectation::matching(request::method_path("GET", "/sessions/whoami"))
            .respond_with(status_code(401).body(
                json!({
                    "error": {
                        "code": 401,
                        "status": "Unauthorized",
                        "message": "No valid session found"
                    }
                })
                .to_string(),
            )),
    );
}

/// Create a mock Matrix homeserver for testing messaging.
pub async fn mock_matrix_server() -> Server {
    Server::run()
}

/// Mock successful Matrix room creation.
pub fn mock_matrix_create_room(server: &Server, room_id: &str) {
    let room_id = room_id.to_string();
    server.expect(
        Expectation::matching(request::method_path("POST", "/_matrix/client/v3/createRoom"))
            .respond_with(status_code(200).body(json!({"room_id": room_id}).to_string())),
    );
}

/// Create a mock Meilisearch server for testing search.
pub async fn mock_meilisearch_server() -> Server {
    Server::run()
}

/// Mock successful Meilisearch indexing.
pub fn mock_meilisearch_index_documents(server: &Server, index_name: &str) {
    server.expect(
        Expectation::matching(request::method_path(
            "POST",
            &format!("/indexes/{index_name}/documents"),
        ))
        .respond_with(status_code(202).body(
            json!({
                "taskUid": 123,
                "indexUid": index_name,
                "status": "enqueued"
            })
            .to_string(),
        )),
    );
}

/// Create a mock Gorush server for testing push notifications.
pub async fn mock_gorush_server() -> Server {
    Server::run()
}

/// Mock successful push notification sending.
pub fn mock_gorush_send_notification(server: &Server) {
    server.expect(
        Expectation::matching(request::method_path("POST", "/api/push"))
            .respond_with(status_code(200).body(
                json!({
                    "success": "ok",
                    "counts": 1
                })
                .to_string(),
            )),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_kratos_server_starts() {
        let server = mock_kratos_server().await;
        assert!(!server.url("/").to_string().is_empty());
    }

    #[tokio::test]
    async fn mock_kratos_session_responds() {
        let server = mock_kratos_server().await;
        mock_kratos_session_valid(&server, "test-identity-123");

        let client = reqwest::Client::new();
        let response = client
            .get(server.url("/sessions/whoami"))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
        let body: serde_json::Value = response.json().await.unwrap();
        assert_eq!(body["identity"]["id"], "test-identity-123");
    }
}
