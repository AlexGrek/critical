#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::http::StatusCode;
    use axum_test::TestServer;
    use serial_test::serial;
    use serde_json::json;

    use crate::{create_app, create_mock_shared_state, schema::*, state::AppState};

    const ROOT_PASSWORD: &str = "changeme";

    /// Generate a unique username to avoid collisions across test runs.
    fn unique_user(prefix: &str) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        format!("{}_{}", prefix, nanos)
    }

    /// Seed the root user if it doesn't exist (mirrors main.rs startup logic).
    async fn ensure_root_user(state: &AppState) {
        if state.db.get_user_by_id("u_root").await.unwrap().is_none() {
            let password_hash = state.auth.hash_password(ROOT_PASSWORD).unwrap();
            let mut root_meta = crit_shared::util_models::ResourceMeta::default();
            root_meta.created_at = chrono::Utc::now();
            let root_user = crit_shared::data_models::User {
                id: "u_root".to_string(),
                password_hash,
                meta: root_meta,
                ..Default::default()
            };
            state.db.create_user(root_user, None).await.unwrap();
        }
    }

    /// Login as root and return the JWT token.
    async fn login_root(server: &TestServer) -> String {
        let resp = server
            .post("/api/login")
            .json(&LoginRequest {
                user: "root".to_string(),
                password: ROOT_PASSWORD.to_string(),
            })
            .await;
        resp.assert_status_ok();
        resp.json::<LoginResponse>().token
    }

    /// Register a user and return the JWT token.
    async fn register_and_login(server: &TestServer, username: &str) -> String {
        let password = "testpassword123";

        server
            .post("/api/register")
            .json(&RegisterRequest {
                user: username.to_string(),
                password: password.to_string(),
            })
            .await
            .assert_status(StatusCode::CREATED);

        let resp = server
            .post("/api/login")
            .json(&LoginRequest {
                user: username.to_string(),
                password: password.to_string(),
            })
            .await;
        resp.assert_status_ok();
        resp.json::<LoginResponse>().token
    }

    #[tokio::test]
    #[serial]
    async fn test_root_godmode_bypasses_user_creation_acl() {
        let state = create_mock_shared_state().await.unwrap();
        ensure_root_user(&state).await;

        // Grant godmode to root (normally done at startup, but mock state
        // doesn't run main() so we do it here)
        state
            .db
            .grant_permission(
                crit_shared::util_models::super_permissions::ADM_GODMODE,
                "u_root",
            )
            .await
            .unwrap();

        let server =
            TestServer::new(create_app(Arc::new(state))).expect("Failed to create TestServer");

        let root_token = login_root(&server).await;

        // Root (with godmode) can create a user via gitops API — normally
        // requires ADM_USER_MANAGER which root doesn't explicitly have.
        let new_user = unique_user("godcreated");
        let resp = server
            .post(&format!("/api/v1/global/users"))
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", root_token).parse::<axum::http::HeaderValue>().unwrap(),
            )
            .json(&json!({
                "id": &new_user,
                "password": "pass123",
                "personal": { "display_name": "God Created User" }
            }))
            .await;
        resp.assert_status(StatusCode::CREATED);

        // Verify the user was actually created
        let get_resp = server
            .get(&format!("/api/v1/global/users/u_{}", new_user))
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", root_token).parse::<axum::http::HeaderValue>().unwrap(),
            )
            .await;
        get_resp.assert_status_ok();
    }

    #[tokio::test]
    #[serial]
    async fn test_regular_user_cannot_create_users() {
        let state = create_mock_shared_state().await.unwrap();
        let server =
            TestServer::new(create_app(Arc::new(state))).expect("Failed to create TestServer");

        let regular_user = unique_user("regular");
        let regular_token = register_and_login(&server, &regular_user).await;

        // Regular user (no godmode, no ADM_USER_MANAGER) should be denied
        let target_user = unique_user("target");
        let resp = server
            .post(&format!("/api/v1/global/users"))
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", regular_token).parse::<axum::http::HeaderValue>().unwrap(),
            )
            .json(&json!({
                "id": &target_user,
                "password": "pass123",
                "personal": { "display_name": "Should Not Exist" }
            }))
            .await;
        // Should be 404 (ACL denial returns 404 to avoid leaking resource existence)
        resp.assert_status(StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    #[serial]
    async fn test_godmode_user_can_read_any_group() {
        let state = create_mock_shared_state().await.unwrap();
        ensure_root_user(&state).await;

        // Grant godmode to root
        state
            .db
            .grant_permission(
                crit_shared::util_models::super_permissions::ADM_GODMODE,
                "u_root",
            )
            .await
            .unwrap();

        let server =
            TestServer::new(create_app(Arc::new(state))).expect("Failed to create TestServer");

        let root_token = login_root(&server).await;
        let regular_user = unique_user("grpmaker");
        let regular_token = register_and_login(&server, &regular_user).await;

        // Regular user creates a group (they have USR_CREATE_GROUPS by default)
        let group_id = unique_user("secretgrp");
        let resp = server
            .post("/api/v1/global/groups")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", regular_token).parse::<axum::http::HeaderValue>().unwrap(),
            )
            .json(&json!({
                "id": &group_id,
                "name": "Secret Group"
            }))
            .await;
        resp.assert_status(StatusCode::CREATED);

        // Root (godmode) should be able to read it, even though root is not
        // in the group's ACL
        let get_resp = server
            .get(&format!("/api/v1/global/groups/g_{}", group_id))
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", root_token).parse::<axum::http::HeaderValue>().unwrap(),
            )
            .await;
        get_resp.assert_status_ok();
    }

    #[tokio::test]
    #[serial]
    async fn test_godmode_user_can_delete_any_group() {
        let state = create_mock_shared_state().await.unwrap();
        ensure_root_user(&state).await;

        // Grant godmode to root
        state
            .db
            .grant_permission(
                crit_shared::util_models::super_permissions::ADM_GODMODE,
                "u_root",
            )
            .await
            .unwrap();

        let server =
            TestServer::new(create_app(Arc::new(state))).expect("Failed to create TestServer");

        let root_token = login_root(&server).await;
        let regular_user = unique_user("grpowner");
        let regular_token = register_and_login(&server, &regular_user).await;

        // Regular user creates a group
        let group_id = unique_user("delgrp");
        server
            .post("/api/v1/global/groups")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", regular_token).parse::<axum::http::HeaderValue>().unwrap(),
            )
            .json(&json!({
                "id": &group_id,
                "name": "Group to Delete"
            }))
            .await
            .assert_status(StatusCode::CREATED);

        // Root (godmode) can delete it
        let del_resp = server
            .delete(&format!("/api/v1/global/groups/g_{}", group_id))
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", root_token).parse::<axum::http::HeaderValue>().unwrap(),
            )
            .await;
        del_resp.assert_status(StatusCode::NO_CONTENT);

        // Verify it's gone (soft-deleted)
        let get_resp = server
            .get(&format!("/api/v1/global/groups/g_{}", group_id))
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", root_token).parse::<axum::http::HeaderValue>().unwrap(),
            )
            .await;
        get_resp.assert_status(StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    #[serial]
    async fn test_granting_godmode_to_regular_user() {
        let state = create_mock_shared_state().await.unwrap();

        let server =
            TestServer::new(create_app(Arc::new(state.clone()))).expect("Failed to create TestServer");

        let user = unique_user("promoted");
        let token = register_and_login(&server, &user).await;

        // Before godmode: cannot create users
        let target = unique_user("shouldfail");
        let resp = server
            .post("/api/v1/global/users")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", token).parse::<axum::http::HeaderValue>().unwrap(),
            )
            .json(&json!({
                "id": &target,
                "password": "pass123"
            }))
            .await;
        resp.assert_status(StatusCode::NOT_FOUND);

        // Grant godmode to the user
        state
            .db
            .grant_permission(
                crit_shared::util_models::super_permissions::ADM_GODMODE,
                &format!("u_{}", user),
            )
            .await
            .unwrap();

        // Invalidate cache so the change takes effect immediately
        state
            .cache
            .invalidate(
                crate::godmode::SPECIAL_ACCESS_CACHE,
                &crate::godmode::godmode_cache_key(&format!("u_{}", user)),
            )
            .await;

        // After godmode: can create users
        let target2 = unique_user("shouldpass");
        let resp = server
            .post("/api/v1/global/users")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", token).parse::<axum::http::HeaderValue>().unwrap(),
            )
            .json(&json!({
                "id": &target2,
                "password": "pass123"
            }))
            .await;
        resp.assert_status(StatusCode::CREATED);
    }

    #[tokio::test]
    #[serial]
    async fn test_godmode_list_returns_all_groups() {
        let state = create_mock_shared_state().await.unwrap();
        ensure_root_user(&state).await;

        // Grant godmode to root
        state
            .db
            .grant_permission(
                crit_shared::util_models::super_permissions::ADM_GODMODE,
                "u_root",
            )
            .await
            .unwrap();

        let server =
            TestServer::new(create_app(Arc::new(state))).expect("Failed to create TestServer");

        let root_token = login_root(&server).await;
        let user = unique_user("listmaker");
        let user_token = register_and_login(&server, &user).await;

        // User creates a group (only they have ACL on it)
        let group_id = unique_user("listgrp");
        server
            .post("/api/v1/global/groups")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", user_token).parse::<axum::http::HeaderValue>().unwrap(),
            )
            .json(&json!({
                "id": &group_id,
                "name": "List Test Group"
            }))
            .await
            .assert_status(StatusCode::CREATED);

        // Root (godmode) listing groups should include this group
        let list_resp = server
            .get("/api/v1/global/groups")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", root_token).parse::<axum::http::HeaderValue>().unwrap(),
            )
            .await;
        list_resp.assert_status_ok();

        let body: serde_json::Value = list_resp.json();
        let items = body["items"].as_array().expect("items should be array");
        let found = items
            .iter()
            .any(|item| item["id"].as_str() == Some(&format!("g_{}", group_id)));
        assert!(
            found,
            "Godmode user should see all groups in list, missing g_{}",
            group_id
        );
    }
}
