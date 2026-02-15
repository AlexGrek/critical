#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::http::StatusCode;

    use axum_test::TestServer;
    use serial_test::serial;
    use serde_json::json;

    use crate::{create_app, create_mock_shared_state, schema::*, validation::limit_min_length};

    /// Generate a unique username to avoid collisions across test runs against a persistent DB.
    fn unique_user(prefix: &str) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos();
        format!("{}_{}", prefix, nanos)
    }

    #[tokio::test]
    #[serial]
    async fn test_health_check() {
        let state = create_mock_shared_state().await.unwrap();
        let server =
            TestServer::new(create_app(Arc::new(state))).expect("Failed to create TestServer");

        let response = server.get("/health").await;

        response.assert_status_ok();
        response.assert_header("Content-Type", "application/json");
        response.assert_json_contains(&json!({
            "status": "healthy",
        }));
    }

    #[tokio::test]
    #[serial]
    async fn test_user_registration_and_login() {
        let state = create_mock_shared_state().await.unwrap();
        let server =
            TestServer::new(create_app(Arc::new(state))).expect("Failed to create TestServer");

        let email = unique_user("reglogin");
        let password = "securepassword123";

        let register_request = RegisterRequest {
            user: email.to_string(),
            password: password.to_string(),
        };

        let register_response = server.post("/api/register").json(&register_request).await;
        register_response.assert_status(StatusCode::CREATED);

        let login_request = LoginRequest {
            user: email.to_string(),
            password: password.to_string(),
        };

        let login_response = server.post("/api/login").json(&login_request).await;
        login_response.assert_status_ok();

        let body: LoginResponse = login_response.json::<LoginResponse>();
        assert!(limit_min_length(15)(&body.token).is_ok());
    }

    #[tokio::test]
    #[serial]
    async fn test_invalid_login_credentials() {
        let state = create_mock_shared_state().await.unwrap();
        let server =
            TestServer::new(create_app(Arc::new(state))).expect("Failed to create TestServer");

        let user = unique_user("invalidcreds");

        server
            .post("/api/register")
            .json(&RegisterRequest {
                user: user.clone(),
                password: "correct_password".to_string(),
            })
            .await
            .assert_status_success();

        let login_request = LoginRequest {
            user: user.clone(),
            password: "wrong_password".to_string(),
        };

        let login_response = server.post("/api/login").json(&login_request).await;
        login_response.assert_status(StatusCode::UNAUTHORIZED);
    }
}
