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
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_cr1t"));
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
    assert!(
        contents.contains("localhost-3742"),
        "context name should be derived from URL"
    );
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
    cr1t_cmd(&home).args(["groups", "list"]).assert().success();
}

#[test]
#[ignore]
fn test_groups_list_with_data() {
    let home = TempDir::new().unwrap();
    let user = unique_user();
    let pass = "testpass999";
    let group_id = format!(
        "g_test{}",
        unique_user().chars().rev().take(6).collect::<String>()
    );
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
    let group_id = format!(
        "g_desc{}",
        unique_user().chars().rev().take(4).collect::<String>()
    );
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

// --- Apply command tests ---

/// Write a context file with the given token so subsequent cr1t commands work.
fn write_context(home: &TempDir, token: &str) {
    let ctx_dir = home.path().join(".cr1tical");
    std::fs::create_dir_all(&ctx_dir).unwrap();
    let content = format!(
        "current: localhost-3742\ncontexts:\n- name: localhost-3742\n  url: {}\n  token: {}\n",
        BACKEND_URL, token
    );
    std::fs::write(ctx_dir.join("context.yaml"), content).unwrap();
}

/// Write a dummy context that points nowhere (for tests that check parse errors before HTTP).
fn write_dummy_context(home: &TempDir) {
    let ctx_dir = home.path().join(".cr1tical");
    std::fs::create_dir_all(&ctx_dir).unwrap();
    std::fs::write(
        ctx_dir.join("context.yaml"),
        "current: test\ncontexts:\n- name: test\n  url: http://localhost:1\n  token: dummy\n",
    )
    .unwrap();
}

/// Delete a group via the API (best-effort cleanup).
fn delete_group(token: &str, group_id: &str) {
    let client = reqwest::blocking::Client::new();
    let _ = client
        .delete(format!("{}/api/v1/global/groups/{}", BACKEND_URL, group_id))
        .header("Authorization", format!("Bearer {}", token))
        .send();
}

#[test]
#[ignore]
fn test_apply_creates_group_from_file() {
    let home = TempDir::new().unwrap();
    let user = unique_user();
    let pass = "applypass1";
    let group_id = format!("g_apply_{}", &user[8..]);

    register_user(&user, pass);
    let token = login_user(&user, pass);
    write_context(&home, &token);

    let yaml = format!("kind: group\nid: {}\nname: Apply Test\n", group_id);
    let yaml_path = home.path().join("group.yaml");
    std::fs::write(&yaml_path, &yaml).unwrap();

    cr1t_cmd(&home)
        .args(["apply", "-f", yaml_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "group/{} applied",
            group_id
        )));

    // Verify group exists with the correct name
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(format!("{}/api/v1/global/groups/{}", BACKEND_URL, group_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .unwrap();
    assert_eq!(
        resp.status().as_u16(),
        200,
        "group should exist after apply"
    );
    let body: serde_json::Value = resp.json().unwrap();
    assert_eq!(body["name"].as_str().unwrap(), "Apply Test");

    delete_group(&token, &group_id);
}

#[test]
#[ignore]
fn test_apply_updates_group_from_file() {
    let home = TempDir::new().unwrap();
    let user = unique_user();
    let pass = "applypass2";
    let group_id = format!("g_applyupd_{}", &user[8..]);

    register_user(&user, pass);
    let token = login_user(&user, pass);
    write_context(&home, &token);

    let yaml_path = home.path().join("group.yaml");
    let client = reqwest::blocking::Client::new();

    // First apply — create the group
    std::fs::write(
        &yaml_path,
        format!("kind: group\nid: {}\nname: Original\n", group_id),
    )
    .unwrap();
    cr1t_cmd(&home)
        .args(["apply", "-f", yaml_path.to_str().unwrap()])
        .assert()
        .success();

    // Fetch current ACL so the update preserves permissions
    let existing: serde_json::Value = client
        .get(format!("{}/api/v1/global/groups/{}", BACKEND_URL, group_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .unwrap()
        .json()
        .unwrap();
    let acl = &existing["acl"];

    // Second apply — update the name
    let update_yaml = format!(
        "kind: group\nid: {}\nname: Updated\nacl: {}\n",
        group_id,
        serde_json::to_string(acl).unwrap()
    );
    std::fs::write(&yaml_path, update_yaml).unwrap();
    cr1t_cmd(&home)
        .args(["apply", "-f", yaml_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "group/{} applied",
            group_id
        )));

    // Verify name changed
    let updated: serde_json::Value = client
        .get(format!("{}/api/v1/global/groups/{}", BACKEND_URL, group_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .unwrap()
        .json()
        .unwrap();
    assert_eq!(updated["name"].as_str().unwrap(), "Updated");

    delete_group(&token, &group_id);
}

#[test]
#[ignore]
fn test_apply_creates_group_from_stdin() {
    let home = TempDir::new().unwrap();
    let user = unique_user();
    let pass = "applypass3";
    let group_id = format!("g_stdin_{}", &user[8..]);

    register_user(&user, pass);
    let token = login_user(&user, pass);
    write_context(&home, &token);

    cr1t_cmd(&home)
        .args(["apply"])
        .write_stdin(format!(
            "kind: group\nid: {}\nname: Stdin Group\n",
            group_id
        ))
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "group/{} applied",
            group_id
        )));

    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(format!("{}/api/v1/global/groups/{}", BACKEND_URL, group_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200);

    delete_group(&token, &group_id);
}

#[test]
#[ignore]
fn test_apply_multi_document_file() {
    let home = TempDir::new().unwrap();
    let user = unique_user();
    let pass = "applypass4";
    let id_a = format!("g_multi_a_{}", &user[8..]);
    let id_b = format!("g_multi_b_{}", &user[8..]);

    register_user(&user, pass);
    let token = login_user(&user, pass);
    write_context(&home, &token);

    let yaml = format!(
        "kind: group\nid: {}\nname: Multi A\n---\nkind: group\nid: {}\nname: Multi B\n",
        id_a, id_b
    );
    let yaml_path = home.path().join("multi.yaml");
    std::fs::write(&yaml_path, &yaml).unwrap();

    cr1t_cmd(&home)
        .args(["apply", "-f", yaml_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("group/{} applied", id_a)))
        .stdout(predicate::str::contains(format!("group/{} applied", id_b)));

    // Verify both groups exist
    let client = reqwest::blocking::Client::new();
    for gid in [&id_a, &id_b] {
        let resp = client
            .get(format!("{}/api/v1/global/groups/{}", BACKEND_URL, gid))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .unwrap();
        assert_eq!(resp.status().as_u16(), 200, "group {} should exist", gid);
    }

    delete_group(&token, &id_a);
    delete_group(&token, &id_b);
}

// --- Apply: error cases (no backend needed) ---

#[test]
fn test_apply_no_context_fails() {
    let home = TempDir::new().unwrap();

    cr1t_cmd(&home)
        .args(["apply"])
        .write_stdin("kind: group\nid: g_x\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("context").or(predicate::str::contains("login")));
}

#[test]
fn test_apply_missing_kind_fails() {
    // Parse errors fire before any HTTP call, so no real backend is needed.
    let home = TempDir::new().unwrap();
    write_dummy_context(&home);

    cr1t_cmd(&home)
        .args(["apply"])
        .write_stdin("id: g_no_kind\nname: missing kind\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("kind"));
}

#[test]
fn test_apply_missing_id_fails() {
    let home = TempDir::new().unwrap();
    write_dummy_context(&home);

    cr1t_cmd(&home)
        .args(["apply"])
        .write_stdin("kind: group\nname: missing id\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("id"));
}

#[test]
fn test_apply_empty_stdin_fails() {
    let home = TempDir::new().unwrap();
    write_dummy_context(&home);

    cr1t_cmd(&home)
        .args(["apply"])
        .write_stdin("")
        .assert()
        .failure()
        .stderr(predicate::str::contains("no valid YAML documents"));
}

#[test]
fn test_apply_nonexistent_file_fails() {
    let home = TempDir::new().unwrap();
    write_dummy_context(&home);

    cr1t_cmd(&home)
        .args(["apply", "-f", "/tmp/cr1t_nonexistent_file_12345.yaml"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to read"));
}
