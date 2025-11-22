#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::http::StatusCode;

    use axum_test::TestServer;
    use serde_json::json;

    use crate::{create_app, create_mock_shared_state, schema::*, validation::limit_min_length};

    #[tokio::test]
    async fn test_health_check() {
        // 1. Create a TestServer from the application router.
        // This creates an in-memory server, avoiding actual networking setup.
        let state = create_mock_shared_state().unwrap();
        let server =
            TestServer::new(create_app(Arc::new(state))).expect("Failed to create TestServer");

        // 2. Send a GET request to the "/health" endpoint.
        let response = server.get("/health").await;

        // 3. Assert the HTTP status code is 200 OK.
        response.assert_status_ok();

        // 4. Assert the Content-Type header is JSON.
        response.assert_header("Content-Type", "application/json");

        // 5. Assert the JSON body structure and content.
        // We expect the 'status' field to be 'healthy', ignoring the 'timestamp'.
        response.assert_json_contains(&json!({
            "status": "healthy",
            // The assert_json_matches method allows you to check for a subset
            // of the JSON fields, which is perfect for ignoring the dynamic timestamp.
        }));
    }

    #[tokio::test]
    async fn test_user_registration_and_login() {
        // GIVEN: A fresh application instance with an empty state
        let state = create_mock_shared_state().unwrap();
        let server =
            TestServer::new(create_app(Arc::new(state))).expect("Failed to create TestServer");

        let email = "user112";
        let password = "securepassword123";

        // --- STEP 1: Register the User ---

        let register_request = RegisterRequest {
            user: email.to_string(),
            password: password.to_string(),
        };

        let register_response = server.post("/api/register").json(&register_request).await;

        // THEN: Registration should succeed with a 201 Created status
        register_response.assert_status(StatusCode::CREATED);

        // --- STEP 2: Log in with the Registered User ---

        let login_request = LoginRequest {
            user: email.to_string(),
            password: password.to_string(),
        };

        let login_response = server.post("/api/login").json(&login_request).await;

        // THEN: Login should succeed with a 200 OK status
        login_response.assert_status_ok();

        // Deserialize the response into the LoginResponse struct
        let body: LoginResponse = login_response.json::<LoginResponse>();
        assert!(limit_min_length(15)(&body.token).is_ok());
    }

    // --- Optional: Test for Invalid Login ---

    #[tokio::test]
    async fn test_invalid_login_credentials() {
        // GIVEN: A server with a registered user
        let state = create_mock_shared_state().unwrap();
        let server =
            TestServer::new(create_app(Arc::new(state))).expect("Failed to create TestServer");

        // Register a user first (setup)
        server
            .post("/api/register")
            .json(&RegisterRequest {
                user: "validusername".to_string(),
                password: "correct_password".to_string(),
            })
            .await
            .assert_status_success();

        // WHEN: Attempting to log in with the wrong password
        let login_request = LoginRequest {
            user: "validusername".to_string(),
            password: "wrong_password".to_string(),
        };

        let login_response = server.post("/api/login").json(&login_request).await;

        // THEN: The status should be 401 Unauthorized
        login_response.assert_status(StatusCode::UNAUTHORIZED);
    }
}
