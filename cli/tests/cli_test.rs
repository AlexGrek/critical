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
