//! CLI integration tests.
//!
//! Tests marked `#[ignore]` require a running backend on localhost:3742 with ArangoDB.
//! Run them via: `task test:cli` (which starts DB + backend automatically)
//! or: `cargo test -p crit-cli --test cli_test -- --include-ignored`
//!
//! Non-ignored tests only touch local context files and need no infrastructure.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

const BACKEND_URL: &str = "http://localhost:3742";

// ─── raw HTTP primitives ──────────────────────────────────────────────────────

fn api_get(token: &str, url: &str) -> reqwest::blocking::Response {
    reqwest::blocking::Client::new()
        .get(url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .unwrap_or_else(|e| panic!("GET {} failed: {}", url, e))
}

fn api_post(token: &str, url: &str, body: serde_json::Value) -> reqwest::blocking::Response {
    reqwest::blocking::Client::new()
        .post(url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&body)
        .send()
        .unwrap_or_else(|e| panic!("POST {} failed: {}", url, e))
}

fn api_delete(token: &str, url: &str) {
    let _ = reqwest::blocking::Client::new()
        .delete(url)
        .header("Authorization", format!("Bearer {}", token))
        .send();
}

// ─── auth helpers ─────────────────────────────────────────────────────────────

/// Register a test user via the API directly (bypass CLI).
fn register_user(username: &str, password: &str) {
    let resp = reqwest::blocking::Client::new()
        .post(format!("{}/api/v1/register", BACKEND_URL))
        .json(&serde_json::json!({ "user": username, "password": password }))
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
    let resp = reqwest::blocking::Client::new()
        .post(format!("{}/api/v1/login", BACKEND_URL))
        .json(&serde_json::json!({ "user": username, "password": password }))
        .send()
        .expect("failed to send login request");

    assert!(resp.status().is_success(), "login failed: {}", resp.status());

    let body: serde_json::Value = resp.json().expect("failed to parse login response");
    body["token"].as_str().expect("token not in response").to_string()
}

// ─── resource helpers ─────────────────────────────────────────────────────────

/// Create a test group via the API.
fn create_group(token: &str, group_id: &str, name: &str) {
    let resp = api_post(
        token,
        &format!("{}/api/v1/global/groups", BACKEND_URL),
        serde_json::json!({ "id": group_id, "name": name, "acl": {} }),
    );
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
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("clitest{}{:03}", ts % 1_000_000_000, seq)
}

// --- Tests requiring running backend (use `task test:cli`) ---

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

    // List groups — default table output shows NAME column header and group name
    cr1t_cmd(&home)
        .args(["groups", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("NAME").and(predicate::str::contains(group_name)));

    delete_group(&token, &group_id);
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

    delete_group(&token, &group_id);
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

    // List users — default table output shows NAME/ID column headers
    cr1t_cmd(&home)
        .args(["users", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("NAME").and(predicate::str::contains("ID")));
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

// --- Output format tests ---

#[test]
#[ignore]
fn test_groups_list_yaml_output() {
    let home = TempDir::new().unwrap();
    let user = unique_user();
    let pass = "testpassyaml1";
    let group_id = format!("g_yaml{}", &user[8..]);

    register_user(&user, pass);
    let token = login_user(&user, pass);
    create_group(&token, &group_id, "YAML Test Group");
    write_context(&home, &token);

    cr1t_cmd(&home)
        .args(["groups", "list", "-o", "yaml"])
        .assert()
        .success()
        .stdout(predicate::str::contains("id:").and(predicate::str::contains("name:")));

    delete_group(&token, &group_id);
}

#[test]
#[ignore]
fn test_groups_list_json_output() {
    let home = TempDir::new().unwrap();
    let user = unique_user();
    let pass = "testpassjson1";
    let group_id = format!("g_json{}", &user[8..]);

    register_user(&user, pass);
    let token = login_user(&user, pass);
    create_group(&token, &group_id, "JSON Test Group");
    write_context(&home, &token);

    cr1t_cmd(&home)
        .args(["groups", "list", "-o", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"items\"").and(predicate::str::contains("\"id\"")));

    delete_group(&token, &group_id);
}

#[test]
#[ignore]
fn test_get_groups_table() {
    let home = TempDir::new().unwrap();
    let user = unique_user();
    let pass = "testpassget1";
    let group_id = format!("g_get{}", &user[8..]);
    let group_name = "Get Table Group";

    register_user(&user, pass);
    let token = login_user(&user, pass);
    create_group(&token, &group_id, group_name);
    write_context(&home, &token);

    // `cr1t get groups` should show a table with NAME/ID headers
    cr1t_cmd(&home)
        .args(["get", "groups"])
        .assert()
        .success()
        .stdout(predicate::str::contains("NAME").and(predicate::str::contains(group_name)));

    delete_group(&token, &group_id);
}

#[test]
#[ignore]
fn test_get_single_resource_table() {
    let home = TempDir::new().unwrap();
    let user = unique_user();
    let pass = "testpassget2";
    let group_id = format!("g_single{}", &user[8..]);
    let group_name = "Single Row Group";

    register_user(&user, pass);
    let token = login_user(&user, pass);
    create_group(&token, &group_id, group_name);
    write_context(&home, &token);

    // `cr1t get groups <id>` should show a single-row table
    cr1t_cmd(&home)
        .args(["get", "groups", &group_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("NAME").and(predicate::str::contains(group_name)));

    delete_group(&token, &group_id);
}

#[test]
#[ignore]
fn test_describe_outputs_yaml_with_kind() {
    let home = TempDir::new().unwrap();
    let user = unique_user();
    let pass = "testpassdesc1";
    let group_id = format!("g_desc2{}", &user[8..]);

    register_user(&user, pass);
    let token = login_user(&user, pass);
    create_group(&token, &group_id, "Describe YAML Group");
    write_context(&home, &token);

    // `cr1t describe groups <id>` outputs full YAML including kind field
    cr1t_cmd(&home)
        .args(["describe", "groups", &group_id])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("kind: group")
                .and(predicate::str::contains(&group_id))
                .and(predicate::str::contains("acl:")),
        );

    delete_group(&token, &group_id);
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

/// Create a project via the API (requires ADM_CONFIG_EDITOR or USR_CREATE_PROJECTS).
fn create_project(token: &str, project_id: &str, name: &str) {
    let resp = api_post(
        token,
        &format!("{}/api/v1/global/projects", BACKEND_URL),
        serde_json::json!({ "id": project_id, "name": name }),
    );
    assert!(
        resp.status().as_u16() == 201 || resp.status().as_u16() == 409,
        "unexpected create project status: {}",
        resp.status()
    );
}

/// Delete a project via the API (best-effort cleanup).
fn delete_project(token: &str, project_id: &str) {
    api_delete(token, &format!("{}/api/v1/global/projects/{}", BACKEND_URL, project_id));
}

/// Create a ticket group via the API.
fn create_ticket_group(token: &str, project_id: &str, tg_id: &str, name: &str) {
    let resp = api_post(
        token,
        &format!("{}/api/v1/projects/{}/ticketgroups", BACKEND_URL, project_id),
        serde_json::json!({ "id": tg_id, "name": name, "ticket_types": [] }),
    );
    assert!(
        resp.status().as_u16() == 201 || resp.status().as_u16() == 409,
        "unexpected create ticket group status: {}",
        resp.status()
    );
}

/// Delete a ticket group via the API (best-effort cleanup).
fn delete_ticket_group(token: &str, project_id: &str, tg_id: &str) {
    api_delete(token, &format!("{}/api/v1/projects/{}/ticketgroups/{}", BACKEND_URL, project_id, tg_id));
}

/// Delete a group via the API (best-effort cleanup).
fn delete_group(token: &str, group_id: &str) {
    api_delete(token, &format!("{}/api/v1/global/groups/{}", BACKEND_URL, group_id));
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
    let resp = api_get(&token, &format!("{}/api/v1/global/groups/{}", BACKEND_URL, group_id));
    assert_eq!(resp.status().as_u16(), 200, "group should exist after apply");
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
    let existing: serde_json::Value = api_get(&token, &format!("{}/api/v1/global/groups/{}", BACKEND_URL, group_id))
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
    let updated: serde_json::Value = api_get(&token, &format!("{}/api/v1/global/groups/{}", BACKEND_URL, group_id))
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

    let resp = api_get(&token, &format!("{}/api/v1/global/groups/{}", BACKEND_URL, group_id));
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
    for gid in [&id_a, &id_b] {
        let resp = api_get(&token, &format!("{}/api/v1/global/groups/{}", BACKEND_URL, gid));
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

// --- Edit command (requires backend) ---

#[test]
#[ignore]
fn test_edit_group_basic() {
    let home = TempDir::new().unwrap();
    let user = unique_user();
    let pass = "testpass_edit";
    let group_id = format!(
        "g_edit{}",
        unique_user().chars().rev().take(4).collect::<String>()
    );
    let group_name = "Edit Test Group";
    let new_name = "Edited Group Name";

    register_user(&user, pass);
    let token = login_user(&user, pass);
    create_group(&token, &group_id, group_name);
    write_context(&home, &token);

    // Create a shell script that modifies the file in place
    let editor_sh = home.path().join("editor.sh");
    let editor_content = format!(
        "#!/bin/bash\nsed -i 's/name: {}/name: {}/' \"$1\"\n",
        group_name, new_name
    );
    std::fs::write(&editor_sh, editor_content).unwrap();

    // Make it executable on Unix
    #[cfg(unix)]
    {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o755);
        fs::set_permissions(&editor_sh, perms).unwrap();
    }

    cr1t_cmd(&home)
        .env("CR1T_EDITOR", editor_sh.to_string_lossy().to_string())
        .args(["edit", "group", &group_id])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("group/{} edited", group_id)));

    // Verify the change was applied
    cr1t_cmd(&home)
        .args(["groups", "describe", &group_id])
        .assert()
        .success()
        .stdout(predicate::str::contains(new_name));

    delete_group(&token, &group_id);
}

#[test]
#[ignore]
fn test_edit_group_no_changes() {
    let home = TempDir::new().unwrap();
    let user = unique_user();
    let pass = "testpass_noedit";
    let group_id = format!(
        "g_noedit{}",
        unique_user().chars().rev().take(4).collect::<String>()
    );
    let group_name = "No Changes Group";

    register_user(&user, pass);
    let token = login_user(&user, pass);
    create_group(&token, &group_id, group_name);
    write_context(&home, &token);

    // Use 'true' command which exits 0 without modifying the file
    cr1t_cmd(&home)
        .env("CR1T_EDITOR", "true")
        .args(["edit", "group", &group_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("no changes"));

    delete_group(&token, &group_id);
}

#[test]
#[ignore]
fn test_edit_resource_not_found() {
    let home = TempDir::new().unwrap();
    let user = unique_user();
    let pass = "testpass_nofound";

    register_user(&user, pass);
    let token = login_user(&user, pass);
    write_context(&home, &token);

    // Try to edit a non-existent group
    cr1t_cmd(&home)
        .env("CR1T_EDITOR", "true")
        .args(["edit", "group", "g_nonexistent_12345"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("404").or(predicate::str::contains("not found")));
}

// --- Debug commands (godmode required) ---

#[test]
#[ignore]
fn test_debug_events_requires_godmode() {
    let home = TempDir::new().unwrap();
    let user = unique_user();
    let pass = "testpass_nonroot";

    register_user(&user, pass);
    let token = login_user(&user, pass);
    write_context(&home, &token);

    // Non-godmode user should get 403 or 401
    cr1t_cmd(&home)
        .args(["debug", "events"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Forbidden").or(predicate::str::contains("Unauthorized")));
}

// --- TicketGroups commands ---

/// Non-ignored: missing project field on a scoped resource is caught before any HTTP call.
#[test]
fn test_apply_scoped_missing_project_fails() {
    let home = TempDir::new().unwrap();
    write_dummy_context(&home);

    cr1t_cmd(&home)
        .args(["apply"])
        .write_stdin("kind: ticketgroup\nid: BUG\nname: Bugs\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("project"));
}

#[test]
#[ignore]
fn test_ticketgroups_list_empty() {
    let home = TempDir::new().unwrap();
    // Use admin (godmode) to create a project — regular users need USR_CREATE_PROJECTS.
    let admin_token = login_user("root", "changeme");
    let project_id = format!("tgtest_{}", &unique_user()[8..]);

    create_project(&admin_token, &project_id, "TG Test Project");
    write_context(&home, &admin_token);

    cr1t_cmd(&home)
        .args(["ticket-groups", "list", "--project", &project_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("No ticketgroups found"));

    delete_project(&admin_token, &project_id);
}

#[test]
#[ignore]
fn test_ticketgroups_list_with_data() {
    let home = TempDir::new().unwrap();
    let admin_token = login_user("root", "changeme");
    let project_id = format!("tgdata_{}", &unique_user()[8..]);
    let tg_id = "BUG";
    let tg_name = "Bug Tracking";

    create_project(&admin_token, &project_id, "TG Data Project");
    create_ticket_group(&admin_token, &project_id, tg_id, tg_name);
    write_context(&home, &admin_token);

    cr1t_cmd(&home)
        .args(["ticket-groups", "list", "--project", &project_id])
        .assert()
        .success()
        .stdout(predicate::str::contains(tg_name));

    delete_ticket_group(&admin_token, &project_id, &format!("tg_{}", tg_id));
    delete_project(&admin_token, &project_id);
}

#[test]
#[ignore]
fn test_ticketgroups_list_yaml_output() {
    let home = TempDir::new().unwrap();
    let admin_token = login_user("root", "changeme");
    let project_id = format!("tgyaml_{}", &unique_user()[8..]);
    let tg_id = "TASK";

    create_project(&admin_token, &project_id, "TG Yaml Project");
    create_ticket_group(&admin_token, &project_id, tg_id, "Tasks");
    write_context(&home, &admin_token);

    cr1t_cmd(&home)
        .args(["ticket-groups", "list", "--project", &project_id, "-o", "yaml"])
        .assert()
        .success()
        .stdout(predicate::str::contains("name: Tasks"));

    delete_ticket_group(&admin_token, &project_id, &format!("tg_{}", tg_id));
    delete_project(&admin_token, &project_id);
}

#[test]
#[ignore]
fn test_ticketgroups_describe() {
    let home = TempDir::new().unwrap();
    let admin_token = login_user("root", "changeme");
    let project_id = format!("tgdesc_{}", &unique_user()[8..]);
    let tg_id = "FEAT";

    create_project(&admin_token, &project_id, "TG Describe Project");
    create_ticket_group(&admin_token, &project_id, tg_id, "Features");
    write_context(&home, &admin_token);

    cr1t_cmd(&home)
        .args(["ticket-groups", "describe", "--project", &project_id, &format!("tg_{}", tg_id)])
        .assert()
        .success()
        .stdout(predicate::str::contains("kind: ticketgroup"))
        .stdout(predicate::str::contains("name: Features"));

    delete_ticket_group(&admin_token, &project_id, &format!("tg_{}", tg_id));
    delete_project(&admin_token, &project_id);
}

#[test]
#[ignore]
fn test_ticketgroups_describe_not_found() {
    let home = TempDir::new().unwrap();
    let admin_token = login_user("root", "changeme");
    let project_id = format!("tg404_{}", &unique_user()[8..]);

    create_project(&admin_token, &project_id, "TG 404 Project");
    write_context(&home, &admin_token);

    cr1t_cmd(&home)
        .args(["ticket-groups", "describe", "--project", &project_id, "tg_NOPE"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("404").or(predicate::str::contains("not found")));

    delete_project(&admin_token, &project_id);
}

#[test]
#[ignore]
fn test_apply_creates_ticketgroup() {
    let home = TempDir::new().unwrap();
    let admin_token = login_user("root", "changeme");
    let project_id = format!("tgapply_{}", &unique_user()[8..]);
    let tg_id = "CR";

    create_project(&admin_token, &project_id, "TG Apply Project");
    write_context(&home, &admin_token);

    let yaml = format!(
        "kind: ticketgroup\nid: {}\nproject: {}\nname: Change Requests\nticket_types: []\n",
        tg_id, project_id
    );

    cr1t_cmd(&home)
        .args(["apply"])
        .write_stdin(yaml)
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("ticketgroup/{} applied", tg_id)));

    delete_ticket_group(&admin_token, &project_id, &format!("tg_{}", tg_id));
    delete_project(&admin_token, &project_id);
}
