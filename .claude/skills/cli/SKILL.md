---
name: cli
description: >
  Expert knowledge of the cr1t CLI codebase (cli/ crate). Use when writing,
  reviewing, or debugging CLI commands, context management, apply logic, API
  client code, or CLI integration tests. Enforces the correct patterns and
  always runs tests after changes.
user-invocable: true
---

You are working on the **cr1t CLI** (`cli/` crate, binary name `cr1t`).
Apply the following architectural knowledge to every piece of code you write or review.
**Always run CLI tests after making changes** — see the Testing section.

> **API reference**: [`docs/api.md`](../../../docs/api.md) contains all backend routes, request/response
> shapes, resource models, and permission bits. Check it before reading any backend Rust files.

---

## Overview

`cr1t` is a gitops-style CLI tool (similar to `kubectl`) for managing Critical
resources. It authenticates via a JWT stored in a local context file and sends
requests to the backend API.

Build: `cargo build --bin cr1t`

---

## File Structure

```
cli/
├── src/
│   ├── main.rs          — clap entrypoint; top-level Commands/Subcommands enum; routes to commands/
│   ├── api.rs           — async HTTP client functions (reqwest); one fn per API call
│   ├── context.rs       — context file load/save; ContextFile, ContextEntry structs
│   └── commands/
│       ├── mod.rs       — re-exports command modules
│       ├── login.rs     — `cr1t login`, `cr1t context list/use`
│       ├── gitops.rs    — `cr1t groups/users list/describe/get`; interactive list selection
│       ├── apply.rs     — `cr1t apply -f FILE` / stdin; YAML parsing + API dispatch
│       ├── edit.rs      — `cr1t edit <kind> <id>`; OS editor launch, hash-conflict handling
│       └── debug.rs     — `cr1t debug events`; event log display
└── tests/
    └── cli_test.rs      — integration tests (assert_cmd)
```

---

## Adding a New Top-Level Command

1. **Add to `main.rs`**:
   - Add a variant to `Commands` enum with `#[command(subcommand)]` or direct args
   - Add subcommand enum(s) if needed (e.g. `FooAction`)
   - Add a match arm in `main()` routing to `commands::foo::run(...).await`

2. **Create `cli/src/commands/foo.rs`**:
   - One `pub async fn run(...) -> anyhow::Result<()>` entry point per subcommand
   - Use `context::require_current()` to get the active server context
   - Call `api::` functions for HTTP — never construct `reqwest::Client` in commands
   - Print results to stdout; print status/progress to stderr

3. **Add API functions in `api.rs`**:
   - Use `fetch_authenticated(url, token)` for GET requests
   - Use `post_authenticated(url, token, body)` for POST/PUT
   - URL format: `{base_url}/api/v1/global/{kind}` or `{base_url}/api/v1/global/{kind}/{id}`
   - All functions return `anyhow::Result<Value>` or `anyhow::Result<T>`

4. **Register in `commands/mod.rs`**:
   - Add `pub mod foo;`

5. **Write integration tests** — see Testing section below.

---

## Adding a New Subcommand to an Existing Group

Example: adding `cr1t projects list`:

1. Add `Projects { #[command(subcommand)] action: ProjectsAction }` to `Commands`
2. Add `enum ProjectsAction { List, Describe { id: String } }`
3. Add match arm: `Commands::Projects { action } => match action { ... }`
4. Implement in `commands/gitops.rs` (or a new `commands/projects.rs`)
5. Add `api::list_projects()` / `api::get_project()` in `api.rs`

---

## `apply` Command Pattern

The `apply` command (`commands/apply.rs`) is the generic resource creation/update path:

- Reads YAML from a file (`-f FILE`) or stdin
- Supports multi-document YAML (`---` separator)
- Each document must have `kind` and `id` fields
- `kind` is stripped from the body before sending (not a DB field)
- `kind` → plural API kind: `"group"` → `"groups"` via `to_api_kind()`
- Sends `POST /api/v1/global/{kind}/{id}` (backend upserts)
- Prints `{kind}/{id} applied` to stdout on success

**To support a new kind via `apply`**, no code changes are needed in `apply.rs` —
just ensure the backend has a `KindController` registered for it.

---

## UX Modes: Interactive, View, and Edit

The CLI supports three complementary UX modes for interacting with resources. When adding new list/get commands or supporting new resource kinds, use this guide to decide which modes to enable.

### Interactive Mode (`-i` / `--interactive`)

Enables fuzzy-searchable selection from a list of resources.

**When to add:**
- Any list command that returns multiple items (e.g., `cr1t get groups`, `cr1t get users`)
- Resources where users frequently need to drill down into a specific item
- Mutable or read-heavy resources (not write-only operations)

**Pattern:**
```rust
// In main.rs Commands::Get variant:
#[arg(short = 'i', long = "interactive")]
interactive: bool,

// In commands/gitops.rs list_resources():
if interactive {
    return list_interactive(&api_kind).await;
}
```

**Already implemented for:**
- `cr1t get [kind] -i` — works with any resource kind via generic `list_interactive()`
- Reuses `dialoguer::FuzzySelect` for type-to-filter + arrow key navigation

**When NOT to add:**
- Single-item operations (e.g., `cr1t describe [kind] [id]`)
- Write-only operations with no natural list view
- Audit-log-only resources where browsing adds no value

### View Modes (`-o table|yaml|json`)

Enables output format selection for structured data.

**When to add:**
- Any command that outputs structured data (lists, single resources, events, etc.)
- Supports YAML for easy piping to `cr1t apply`, JSON for scripting

**Pattern:**
```rust
// In main.rs Commands variant:
#[arg(short = 'o', long = "output", value_name = "FORMAT", default_value = "table")]
output: OutputFormat,

// In command implementation:
match output {
    OutputFormat::Table => { /* print table */ }
    OutputFormat::Yaml => { /* print YAML */ }
    OutputFormat::Json => { /* print JSON */ }
}
```

**Already implemented for:**
- `cr1t get [kind]` / `cr1t get [kind] [id]`
- `cr1t groups list` / `cr1t users list`
- `cr1t describe [kind] [id]`
- `cr1t debug events`

**When NOT to add:**
- Login/auth commands (fixed output)
- Context switching (no structured data)
- Write operations with fixed success/error messages

### Edit Mode (`cr1t edit [kind] [id]`)

Opens resources in `$CR1T_EDITOR` / `$VISUAL` / `$EDITOR` / `vi` for interactive editing.

**Implementation:**
- Generic in `commands/edit.rs` — **no code changes needed for new resource kinds**
- Editor resolution order: `CR1T_EDITOR` (highest priority) → `$VISUAL` → `$EDITOR` → `vi`
- Workflow: fetch → strip server fields → open editor → save changes via upsert POST
- Conflict detection: re-fetches `hash_code` before saving; 409 Conflict handled gracefully
- Server-managed fields stripped from editor view: `hash_code`, `state`, `deletion`

**Works automatically for:**
- Any resource kind where the backend `KindController` supports upsert
- Just ensure the resource is in the database schema — no CLI changes needed

**When to add for new kinds:**
- Mutable resources with complex YAML structure (groups, users, projects, tickets, service_accounts, pipeline_accounts)
- Test via: `cr1t edit [kind] [id]`

**When NOT to add:**
- Membership edges (create/delete only; no YAML body to edit)
- Read-only audit logs (events)

### Decision Matrix: Which Modes to Support

| Resource Kind | `-i` Interactive | `-o` View Modes | `cr1t edit` | Notes |
|---|---|---|---|---|
| groups | ✅ | ✅ | ✅ | Fully supported |
| users | ✅ | ✅ | ✅ | Fully supported |
| projects | ✅ | ✅ | ✅ | Fully supported |
| tickets | ✅ | ✅ | ✅ | Fully supported |
| permissions | ✅ | ✅ | ✅ | Editable ACLs (YAML or form mode) |
| service_accounts | ✅ | ✅ | ✅ | Fully supported |
| pipeline_accounts | ✅ | ✅ | ✅ | Fully supported |
| memberships | ✅ | ✅ | ❌ | Edges; create/delete only, no YAML body |
| events | ✅ | ✅ | ❌ | Read-only audit log; no editing |

---

## Context System (`context.rs`)

```rust
struct ContextFile {
    current: Option<String>,       // name of the active context
    contexts: Vec<ContextEntry>,
}

struct ContextEntry {
    name: String,   // e.g. "localhost-3742" (derived from URL)
    url: String,    // e.g. "http://localhost:3742"
    token: String,  // JWT
}
```

- Stored at `~/.cr1tical/context.yaml`
- `context::load()` / `context::save()` — use `HOME` env var (overridable in tests)
- `context::require_current()` — returns active entry or bails with "Run `cr1t login` first."
- `ContextFile::upsert()` — adds or updates an entry by name (idempotent login)

**In tests**: always set `HOME` via `cmd.env("HOME", home.path())` to isolate context files.

---

## API Client (`api.rs`)

Pattern for all API functions:
```rust
pub async fn list_things(base_url: &str, token: &str) -> Result<Value> {
    let url = format!("{}/api/v1/global/things", base_url.trim_end_matches('/'));
    fetch_authenticated(&url, token).await
}

pub async fn get_thing(base_url: &str, token: &str, id: &str) -> Result<Value> {
    let url = format!("{}/api/v1/global/things/{}", base_url.trim_end_matches('/'), id);
    fetch_authenticated(&url, token).await
}
```

Error handling: both `fetch_authenticated` and `post_authenticated` deserialize
the backend's `{ "error": { "message": "...", "status": 422 } }` format and
propagate it as a human-readable `anyhow::Error`.

---

## Testing

### ALWAYS RUN TESTS AFTER MAKING CLI CHANGES

> **WARNING**: Compilation of the CLI (`cargo build --bin cr1t`) and backend (`cargo build --bin axum-api`) is **slow** — expect 30–120 seconds on first build or after significant changes. Do **NOT** modify source files while a build is in progress; doing so invalidates the build and may cause it to fail or produce a stale binary.

**Fast (no infrastructure needed) — run first:**
```bash
cargo test -p crit-cli
```
This runs all non-`#[ignore]` tests: unit tests in `context.rs` and `apply.rs`,
plus CLI tests that don't need a backend.

**Full integration tests (needs DB + backend):**
```bash
task test:cli
```
Starts ephemeral ArangoDB + backend, runs all tests including `#[ignore]` ones,
then tears everything down.

> **IMPORTANT**: If the fast tests pass and you need to verify that the CLI works
> correctly (e.g. after making changes, or when the user asks you to run tests),
> **always follow up with `task test:cli`** to run the full suite including
> integration tests. Fast tests passing alone is not sufficient — integration
> tests catch real API, auth, and end-to-end failures.

Or manually (backend already running):
```bash
cargo test -p crit-cli --test cli_test -- --include-ignored
```

> **CRITICAL**: If you reset or restart the dev database (`task db:reset`, `task run:fresh`, etc.), you **must restart the backend** before running tests. The backend initializes all ArangoDB collections on startup — if ArangoDB wasn't running when the backend started, no collections exist and every API call returns 500. Symptoms: `register_user` fails with 500, all `#[ignore]` tests fail at line 28.

---

### Test Patterns in `cli/tests/cli_test.rs`

**Always use `assert_cmd::Command` — never shell out directly.**

```rust
fn cr1t_cmd(home: &TempDir) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_cr1t"));
    cmd.env("HOME", home.path());  // isolates context file
    cmd
}
```

**Tests that need a live backend are `#[ignore]`**:
```rust
#[test]
#[ignore]
fn test_foo_requires_backend() { ... }
```

**Tests that don't need a backend** (parse errors, context-only operations):
```rust
#[test]
fn test_apply_missing_kind_fails() {
    let home = TempDir::new().unwrap();
    write_dummy_context(&home);  // valid context pointing to localhost:1 (unreachable)
    cr1t_cmd(&home)
        .args(["apply"])
        .write_stdin("id: g_x\nname: missing kind\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("kind"));
}
```

**Creating a seeded context file for backend tests** (avoids interactive TTY):
```rust
fn write_context(home: &TempDir, token: &str) {
    let ctx_dir = home.path().join(".cr1tical");
    std::fs::create_dir_all(&ctx_dir).unwrap();
    let content = format!(
        "current: localhost-3742\ncontexts:\n- name: localhost-3742\n  url: {}\n  token: {}\n",
        BACKEND_URL, token
    );
    std::fs::write(ctx_dir.join("context.yaml"), content).unwrap();
}
```

**Standard test scaffolding for backend tests**:
```rust
#[test]
#[ignore]
fn test_my_command() {
    let home = TempDir::new().unwrap();
    let user = unique_user();     // random name to avoid conflicts
    let pass = "testpassXXX";

    register_user(&user, pass);   // POST /api/register
    let token = login_user(&user, pass);
    write_context(&home, &token);

    cr1t_cmd(&home)
        .args(["my-command", "arg"])
        .assert()
        .success()
        .stdout(predicate::str::contains("expected output"));
}
```

**Cleanup**: always delete resources created during backend tests:
```rust
delete_group(&token, &group_id);  // best-effort, use let _ = ... if it can fail
```

### Unit tests in source files

`context.rs` and `apply.rs` both have `#[cfg(test)]` modules with unit tests.
Add unit tests there for pure logic (parsing, context manipulation) — they run
without infrastructure as part of `cargo test -p crit-cli`.

---

## Self-Review Before Finishing

- [ ] `cargo test -p crit-cli` passes (fast, no infra needed — run this first)
- [ ] `task test:cli` passes (full integration suite — always run if fast tests pass and CLI correctness must be verified)
- [ ] New commands have integration tests in `cli/tests/cli_test.rs` with `#[ignore]`
- [ ] `write_context()` / `write_dummy_context()` used for context setup in tests
- [ ] `unique_user()` used to generate random usernames (avoid test conflicts)
- [ ] Resources created in tests are cleaned up (delete at end)
- [ ] `cargo build --bin cr1t` passes with no warnings
- [ ] New API calls in `api.rs` follow `fetch_authenticated` / `post_authenticated` pattern
- [ ] Commands read context via `context::require_current()`, never hardcode URLs
- [ ] Output: results → stdout, status/errors → stderr

### UX Modes Checklist (for new commands/kinds)

- [ ] **Interactive mode**: If adding a new list command for a mutable, multi-item resource, consider `-i` flag using `list_interactive(&api_kind)` from `gitops.rs`
- [ ] **View modes**: All list/get commands should support `-o table|yaml|json` (already standard for existing commands)
- [ ] **Edit mode**: New mutable resource kinds work automatically with `cr1t edit [kind] [id]` if backend `KindController` supports upsert — no CLI code changes needed; test with `cr1t edit [kind] [id]`
- [ ] **Editor override**: If modifying editor logic, respect `CR1T_EDITOR` > `$VISUAL` > `$EDITOR` > `vi` resolution order; for `cr1t edit`, this is already implemented
