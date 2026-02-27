---
name: rust-backend
description: >
  Expert knowledge of this project's Rust backend architecture. Use when writing,
  reviewing, or debugging backend Rust code — controllers, DB layer, models,
  middleware, routing, or new resource kinds. Provides architectural constraints
  and patterns specific to this codebase so you don't have to re-read CLAUDE.md.
user-invocable: true
---

You are working on the **Critical (crit-cli)** Rust backend. Apply the following
architectural knowledge to every piece of code you write or review.

---

## Cargo Workspace

Three crates:
- `shared/` (`crit-shared`) — domain models + ACL utilities, used by backend and CLI
- `backend/` (`axum-api`) — Axum 0.8 + Tokio web server
- `cli/` (`crit-cli`) — binary `cr1t`

Build: `cargo build --bin axum-api` / `cargo build --bin cr1t`

---

## Domain Models (`shared/src/data_models.rs`)

All entities use the **`#[crit_derive::crit_resource]`** proc macro
(`shared/derive/src/lib.rs`). It automatically injects:
- `id: String` (serde → `_key` for ArangoDB)
- `labels: Labels` (queryable key-value pairs, user-managed desired state)
- `annotations: Labels` (freeform strings, user-managed desired state)
- `state: ResourceState` (server-managed audit: created_at/by, updated_at/by)
- `acl: AccessControlStore` (unless `no_acl` is specified)
- `deletion: Option<DeletionInfo>` (for soft-delete)
- `hash_code: String`

And generates:
- `{Name}Brief` struct with fields tagged `#[brief]`
- `to_brief()`, `compute_hash()`, `collection_name()`, `id_prefix()`

**ID prefixes**: `u_` (users), `g_` (groups), `sa_` (service_accounts), `pa_` (pipeline_accounts)

**Soft-delete**: all list/get queries filter `doc.deletion == null`;
DELETE uses `generic_soft_delete` — never physically remove documents.

**Adding a model field**: use `Option<T>` or `#[serde(default)]` for backwards
compatibility — old documents must still deserialize without errors.

Edge collection (`memberships`) is defined manually (no macro); uses `_from`/`_to`/`_key`.

---

## Database Layer (`backend/src/db/arangodb/`)

- `ArangoDb` struct holds one `Collection<ReqwestClient>` handle per collection.
- **Adding a new collection**: add a field to `ArangoDb`, add `create_collection`
  calls in both `connect_basic` and `connect_anon` in `init.rs`.
- No migration system — ArangoDB is schemaless; Rust structs are the schema.
- `arangors` crate — use `db.aql_query(AqlQuery::...)` for complex queries.
- UPSERT operations have a retry helper to handle write-write conflicts.
- **Always update `DATABASE.md`** when changing schema.

---

## Routing (`backend/src/`)

```
/health              — unauthenticated health check
/register, /login    — auth (no JWT required)
/v1/*                — JWT-protected (middleware in src/middleware/auth.rs)
/v1/ws               — WebSocket
/swagger-ui          — OpenAPI (utoipa)
```

All routes are nested under `/api` in the OpenAPI router.

---

## KindController Pattern (CRITICAL — always follow this)

The generic gitops API at `/v1/global/{kind}/...` dispatches to kind-specific
controllers via the `KindController` trait (`backend/src/controllers/gitops_controller.rs`).

**Trait methods** (implement all, use defaults where appropriate):
```rust
async fn can_read(&self, user_id: &str, doc: Option<&Value>) -> Result<bool, AppError>
async fn can_write(&self, user_id: &str, doc: Option<&Value>) -> Result<bool, AppError>
async fn can_create(&self, user_id: &str, body: &Value) -> Result<bool, AppError>  // default: can_write(None)
fn to_internal(&self, body: Value, auth: &Auth) -> Result<Value, AppError>
fn to_external(&self, doc: Value) -> Value
fn to_list_external(&self, doc: Value) -> Value  // default: to_external
fn list_projection_fields(&self) -> Option<&'static [&'static str]>  // None = all fields
fn prepare_create(&self, body: &mut Value, user_id: &str)
async fn after_create(&self, key: &str, user_id: &str, db: &ArangoDb) -> Result<(), AppError>
async fn after_delete(&self, key: &str, db: &ArangoDb) -> Result<(), AppError>
async fn after_update(&self, key: &str, db: &ArangoDb) -> Result<(), AppError>
fn is_scoped(&self) -> bool  // true for project-scoped resources
fn resource_kind_name(&self) -> &str  // e.g. "tasks" for scoped resources
fn super_permission(&self) -> Option<&str>
```

**Adding a new resource kind — checklist:**
1. Create `backend/src/controllers/{kind}_controller.rs`
2. `struct {Kind}Controller { db: Arc<ArangoDb> }` + `impl {Kind}Controller { pub fn new(db: Arc<ArangoDb>) -> Self }`
3. `#[async_trait] impl KindController for {Kind}Controller { ... }`
4. Add field `pub {kind}: {Kind}Controller` to `Controller` in `controllers/mod.rs`
5. Add `{kind}: {Kind}Controller::new(db.clone())` in `Controller::new()`
6. Add match arm `"{kinds}" => &self.{kind}` in `Controller::for_kind()`
7. **No changes needed in route handlers** — dispatch is automatic

**NEVER** add kind-specific `match kind { ... }` logic in route handlers.
All kind-specific behavior lives in controllers.

---

## Shared Helpers (use these, don't duplicate)

From `gitops_controller.rs`:
```rust
standard_to_internal(body: Value) -> Value       // renames id → _key
standard_to_external(doc: Value) -> Value        // renames _key → id, strips _id/_rev
rename_id_to_key(body: &mut Value)
rename_key_to_id(doc: &mut Value)
parse_acl(doc: &Value) -> Result<AccessControlStore, ()>
filter_to_brief(value: Value, fields: &[&str]) -> Value
```

---

## ACL / Permissions (`shared/src/util_models.rs`)

- `Permissions` — bitflag struct: `READ`, `MODIFY`, `DELETE`, `LIST`, `NOTIFY`, etc.
- `AccessControlList { permissions, principals: Vec<String>, scope: Option<String> }`
- `AccessControlStore { list: Vec<AccessControlList>, last_mod_date }`
- `acl.check_permission(principals, required)` — checks if any principal has the bits
- `check_hybrid_acl(doc, principals, required, project_acl)` — resource ACL with
  fallback to project-scoped ACL (implemented on `KindController`)
- ACL denial → return **404** (not 403) to avoid leaking resource existence

Super-permissions (in `permissions` collection) short-circuit normal ACL checks.

---

## AppState (`backend/src/state.rs`)

```rust
pub struct AppState {
    pub config: Config,
    pub auth: Auth,
    pub db: Arc<ArangoDb>,
    pub controllers: Controller,
    pub github: Option<GitHubService>,
    pub offloadmq: Option<OffloadMqService>,
}
```

Shared as `Arc<AppState>` — extract with `State(state): State<Arc<AppState>>` in handlers.

---

## Error Handling

Use `AppError` (`backend/src/error.rs`) — implements `IntoResponse` for Axum.
Common variants: `AppError::NotFound`, `AppError::Unauthorized`, `AppError::BadRequest`,
`AppError::Internal`.

---

## Debugging: Direct ArangoDB Queries

When debugging data issues, query ArangoDB directly via its HTTP API.
Connection details are in the root `.env` file.

```bash
# Read .env for connection details (DB_CONNECTION_STRING, DB_NAME, DB_USER, DB_PASSWORD)
# Default dev values: root:devpassword on localhost:8529, database "devdb"

# Fetch a single document by _key
curl -u root:devpassword \
  'http://localhost:8529/_db/devdb/_api/document/{collection}/{_key}'

# Run an AQL query
curl -u root:devpassword \
  'http://localhost:8529/_db/devdb/_api/cursor' \
  -d '{"query": "FOR doc IN users LIMIT 5 RETURN doc"}'

# List all collections
curl -u root:devpassword \
  'http://localhost:8529/_db/devdb/_api/collection'
```

Use this to verify what's actually stored in the database when the API response
doesn't match expectations — the issue may be in document creation, AQL projection
(`KEEP`), or the `to_external`/`to_list_external` transformation pipeline.

---

## Testing

- Backend integration tests: `backend/src/test/` — use `axum-test` in-memory server
- `create_mock_shared_state()` in `main.rs` connects to ArangoDB from `.env`
- **Always write Python integration tests** for new endpoints in `backend/itests/tests/`
  (PDM-managed, run with `cd backend/itests && pdm run pytest tests/ -v`)
- Python test pattern: random user fixture (scope=module), JWT auth, clean up after

---

## Logging

Use `log::debug!`, `log::info!`, `log::warn!`, `log::error!`.
ACL checks must include a `log::debug!` with controller name and decision.

---

## Self-Review Before Finishing

- [ ] `cargo build` passes with no warnings
- [ ] New kind follows KindController pattern end-to-end (no inline `match kind`)
- [ ] Used `standard_to_internal/external` helpers
- [ ] ACL denials return 404
- [ ] `DATABASE.md` updated if schema changed
- [ ] Python itest added for new endpoints
