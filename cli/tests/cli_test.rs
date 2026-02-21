//! CLI integration tests.
//!
//! Tests marked `#[ignore]` require a running backend on localhost:3742 with ArangoDB.
//! Run them via: `make test-cli` (which starts DB + backend automatically)
//! or: `cargo test -p crit-cli --test cli_test -- --include-ignored`
//!
//! Non-ignored tests only touch local context files and need no infrastructure.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

const BACKEND_URL: &str = "http://localhost:3742";

/// Register a test user via the API directly (bypass CLI).
fn register_user(username: &str, password: &str) {
    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(format!("{}/api/register", BACKEND_URL))
        .json(&serde_json::json!({
            "user": username,
            "password": password,
        }))
        .send()
        .expect("failed to send register request");

    // 201 = created, 409 = already exists (both acceptable)
    assert!(
        resp.status().as_u16() == 201 || resp.status().as_u16() == 409,
        "unexpected register status: {}",
        resp.status()
    );
}

/// Login a user via the API and return JWT token.
fn login_user(username: &str, password: &str) -> String {
    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(format!("{}/api/login", BACKEND_URL))
        .json(&serde_json::json!({
            "user": username,
            "password": password,
        }))
        .send()
        .expect("failed to send login request");

    assert!(
        resp.status().is_success(),
        "login failed with status: {}",
        resp.status()
    );

    let body: serde_json::Value = resp.json().expect("failed to parse login response");
    body.get("token")
        .and_then(|v| v.as_str())
        .expect("token not in response")
        .to_string()
}

/// Create a test group via the API.
fn create_group(token: &str, group_id: &str, name: &str) {
    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(format!("{}/api/v1/global/groups", BACKEND_URL))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "id": group_id,
            "name": name,
            "acl": {}
        }))
        .send()
        .expect("failed to send create group request");

    // 201 = created, 409 = already exists
    assert!(
        resp.status().as_u16() == 201 || resp.status().as_u16() == 409,
        "unexpected create group status: {}",
        resp.status()
    );
}

fn cr1t_cmd(home: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("cr1t").expect("cr1t binary not found");
    cmd.env("HOME", home.path());
    cmd
}

fn unique_user() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("clitest{}", ts % 1_000_000_000)
}

// --- Tests requiring running backend (use `make test-cli`) ---

#[test]
#[ignore]
fn test_login_success() {
    let home = TempDir::new().unwrap();
    let user = unique_user();
    let pass = "testpass123";

    register_user(&user, pass);

    let mut cmd = cr1t_cmd(&home);
    cmd.args(["login", "--url", BACKEND_URL, "--user", &user])
        .write_stdin(format!("{}\n", pass));

    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Logged in successfully"));

    // Verify context file was created
    let ctx_path = home.path().join(".cr1tical").join("context.yaml");
    assert!(ctx_path.exists(), "context.yaml should exist after login");

    let contents = std::fs::read_to_string(&ctx_path).unwrap();
    assert!(contents.contains("localhost-3742"), "context name should be derived from URL");
}

#[test]
#[ignore]
fn test_login_invalid_credentials() {
    let home = TempDir::new().unwrap();

    let mut cmd = cr1t_cmd(&home);
    cmd.args(["login", "--url", BACKEND_URL, "--user", "nonexistent"])
        .write_stdin("wrongpass\n");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Unauthorized"));
}

#[test]
#[ignore]
fn test_context_list_after_login() {
    let home = TempDir::new().unwrap();
    let user = unique_user();
    let pass = "testpass456";

    register_user(&user, pass);

    // Login first
    cr1t_cmd(&home)
        .args(["login", "--url", BACKEND_URL, "--user", &user])
        .write_stdin(format!("{}\n", pass))
        .assert()
        .success();

    // List contexts
    cr1t_cmd(&home)
        .args(["context", "list"])
        .assert()
        .success()
        .stderr(predicate::str::contains("*"))
        .stderr(predicate::str::contains("localhost-3742"));
}

// --- Tests that need no infrastructure ---

#[test]
fn test_context_use_nonexistent() {
    let home = TempDir::new().unwrap();

    cr1t_cmd(&home)
        .args(["context", "use", "doesnotexist"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_context_list_empty() {
    let home = TempDir::new().unwrap();

    cr1t_cmd(&home)
        .args(["context", "list"])
        .assert()
        .success()
        .stderr(predicate::str::contains("No contexts configured"));
}

// --- Groups and Users commands (require backend) ---

#[test]
#[ignore]
fn test_groups_list() {
    let home = TempDir::new().unwrap();
    let user = unique_user();
    let pass = "testpass789";

    register_user(&user, pass);
    let token = login_user(&user, pass);

    // Create context file manually (avoid interactive login with TTY issues)
    let ctx_dir = home.path().join(".cr1tical");
    std::fs::create_dir_all(&ctx_dir).unwrap();
    let ctx_content = format!(
        "current: localhost-3742\ncontexts:\n- name: localhost-3742\n  url: {}\n  token: {}\n",
        BACKEND_URL, token
    );
    std::fs::write(ctx_dir.join("context.yaml"), ctx_content).unwrap();

    // List groups (should succeed even if empty)
    cr1t_cmd(&home)
        .args(["groups", "list"])
        .assert()
        .success();
}

#[test]
#[ignore]
fn test_groups_list_with_data() {
    let home = TempDir::new().unwrap();
    let user = unique_user();
    let pass = "testpass999";
    let group_id = format!("g_test{}", unique_user().chars().rev().take(6).collect::<String>());
    let group_name = "Test Group";

    register_user(&user, pass);
    let token = login_user(&user, pass);
    create_group(&token, &group_id, group_name);

    // Create context file manually
    let ctx_dir = home.path().join(".cr1tical");
    std::fs::create_dir_all(&ctx_dir).unwrap();
    let ctx_content = format!(
        "current: localhost-3742\ncontexts:\n- name: localhost-3742\n  url: {}\n  token: {}\n",
        BACKEND_URL, token
    );
    std::fs::write(ctx_dir.join("context.yaml"), ctx_content).unwrap();

    // List groups
    cr1t_cmd(&home)
        .args(["groups", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Groups:").or(predicate::str::contains(group_name)));
}

#[test]
#[ignore]
fn test_groups_describe() {
    let home = TempDir::new().unwrap();
    let user = unique_user();
    let pass = "testpassabc";
    let group_id = format!("g_desc{}", unique_user().chars().rev().take(4).collect::<String>());
    let group_name = "Describe Test Group";

    register_user(&user, pass);
    let token = login_user(&user, pass);
    create_group(&token, &group_id, group_name);

    // Create context file manually
    let ctx_dir = home.path().join(".cr1tical");
    std::fs::create_dir_all(&ctx_dir).unwrap();
    let ctx_content = format!(
        "current: localhost-3742\ncontexts:\n- name: localhost-3742\n  url: {}\n  token: {}\n",
        BACKEND_URL, token
    );
    std::fs::write(ctx_dir.join("context.yaml"), ctx_content).unwrap();

    // Describe group
    cr1t_cmd(&home)
        .args(["groups", "describe", &group_id])
        .assert()
        .success()
        .stdout(predicate::str::contains(&group_id).or(predicate::str::contains(group_name)));
}

#[test]
#[ignore]
fn test_groups_describe_not_found() {
    let home = TempDir::new().unwrap();
    let user = unique_user();
    let pass = "testpassdef";

    register_user(&user, pass);
    let token = login_user(&user, pass);

    // Create context file manually
    let ctx_dir = home.path().join(".cr1tical");
    std::fs::create_dir_all(&ctx_dir).unwrap();
    let ctx_content = format!(
        "current: localhost-3742\ncontexts:\n- name: localhost-3742\n  url: {}\n  token: {}\n",
        BACKEND_URL, token
    );
    std::fs::write(ctx_dir.join("context.yaml"), ctx_content).unwrap();

    // Try to describe non-existent group
    cr1t_cmd(&home)
        .args(["groups", "describe", "g_nonexistent"])
        .assert()
        .failure();
}

#[test]
#[ignore]
fn test_users_list() {
    let home = TempDir::new().unwrap();
    let user = unique_user();
    let pass = "testpassghi";

    register_user(&user, pass);
    let token = login_user(&user, pass);

    // Create context file manually
    let ctx_dir = home.path().join(".cr1tical");
    std::fs::create_dir_all(&ctx_dir).unwrap();
    let ctx_content = format!(
        "current: localhost-3742\ncontexts:\n- name: localhost-3742\n  url: {}\n  token: {}\n",
        BACKEND_URL, token
    );
    std::fs::write(ctx_dir.join("context.yaml"), ctx_content).unwrap();

    // List users (should at least contain the logged-in user)
    cr1t_cmd(&home)
        .args(["users", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Users:").or(predicate::str::contains(&user)));
}

#[test]
#[ignore]
fn test_users_describe() {
    let home = TempDir::new().unwrap();
    let user = unique_user();
    let pass = "testpassjkl";

    register_user(&user, pass);
    let token = login_user(&user, pass);

    // Create context file manually
    let ctx_dir = home.path().join(".cr1tical");
    std::fs::create_dir_all(&ctx_dir).unwrap();
    let ctx_content = format!(
        "current: localhost-3742\ncontexts:\n- name: localhost-3742\n  url: {}\n  token: {}\n",
        BACKEND_URL, token
    );
    std::fs::write(ctx_dir.join("context.yaml"), ctx_content).unwrap();

    // Describe the logged-in user
    let user_id = format!("u_{}", user);
    cr1t_cmd(&home)
        .args(["users", "describe", &user_id])
        .assert()
        .success()
        .stdout(predicate::str::contains(&user_id));
}

#[test]
#[ignore]
fn test_users_describe_not_found() {
    let home = TempDir::new().unwrap();
    let user = unique_user();
    let pass = "testpassmno";

    register_user(&user, pass);
    let token = login_user(&user, pass);

    // Create context file manually
    let ctx_dir = home.path().join(".cr1tical");
    std::fs::create_dir_all(&ctx_dir).unwrap();
    let ctx_content = format!(
        "current: localhost-3742\ncontexts:\n- name: localhost-3742\n  url: {}\n  token: {}\n",
        BACKEND_URL, token
    );
    std::fs::write(ctx_dir.join("context.yaml"), ctx_content).unwrap();

    // Try to describe non-existent user
    cr1t_cmd(&home)
        .args(["users", "describe", "u_nonexistent"])
        .assert()
        .failure();
}
