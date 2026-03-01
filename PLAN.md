# Refactoring Plan — Remaining TODOs

## 1. Hash-Based Optimistic Concurrency Control (OCC) - DONE

**Goal**: Prevent lost updates by computing and validating `hash_code` on every write.

**Effort**: Large — touches backend handlers, controllers, CLI, and frontend.

### Files to Modify

| File | Change |
|------|--------|
| `backend/src/api/v1/gitops.rs` | Extract `hash_code` from request body in `update_object` and `upsert_object`; compare against stored doc's hash; return 409 on mismatch |
| `backend/src/controllers/gitops_controller.rs` | Add `compute_hash_on_write()` default method or modify `to_internal()` contract to set `hash_code` after transformation |
| `shared/derive/src/lib.rs` | Remove the 4 TODO comments from `with_computed_hash()` once integrated |
| `backend/src/db/arangodb/mod.rs` | `generic_update` and `generic_upsert` should accept and store the computed hash |
| `frontend/` (API layer) | Send back `hash_code` received from GET when submitting PUT/POST updates; handle 409 response (show "resource was modified" dialog) |
| `cli/src/commands/apply.rs` | Send `hash_code` from fetched resource on apply; handle 409 with user-facing error |

### Exact Changes

**`gitops.rs` — `update_object` (around line 250)**:
```rust
// After fetching the existing document:
let existing = state.db.generic_get(&kind, &id).await?;
let existing = existing.ok_or_else(|| AppError::not_found(...))?;

// NEW: Extract client hash from request body (optional field)
let client_hash = body.get("hash_code").and_then(|v| v.as_str()).map(String::from);

// NEW: Compare hashes if client provided one
if let Some(ref ch) = client_hash {
    let server_hash = existing.get("hash_code").and_then(|v| v.as_str()).unwrap_or("");
    if !server_hash.is_empty() && ch != server_hash {
        return Err(AppError::conflict(format!(
            "{}/{} was modified since last read (expected hash {}, server has {})",
            kind, id, ch, server_hash
        )));
    }
}
```

**`gitops.rs` — `upsert_object` update branch (around line 206)**:
Same hash comparison logic as above, gated on `is_update && body.contains_key("hash_code")`.

**`to_internal` pipeline — compute hash before DB write**:
After `ctrl.to_internal(body, &state.auth)?` returns the transformed doc, compute the hash on the JSON value:
```rust
let doc = ctrl.to_internal(body, &state.auth)?;
// Compute hash on the desired-state fields
let mut doc = doc; // make mutable
if let Some(obj) = doc.as_object_mut() {
    // Remove non-desired-state fields before hashing
    let mut hash_val = doc.clone();
    if let Some(hobj) = hash_val.as_object_mut() {
        for key in ["hash_code", "deletion", "state", "_id", "_rev"] {
            hobj.remove(key);
        }
    }
    let canonical = serde_json::to_string(&hash_val).unwrap_or_default();
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in canonical.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    obj.insert("hash_code".to_string(), json!(format!("{:016x}", hash)));
}
```

> **Note**: The hash computation logic duplicates `compute_hash()` from the proc macro but operates on raw `Value` (controllers work with `serde_json::Value`, not typed structs). Extract a shared `fn compute_value_hash(val: &Value) -> String` into `crit-shared` to avoid duplication.

### Implementation Order
1. Add `compute_value_hash()` to `crit-shared`
2. Wire hash computation into handlers (after `to_internal`, before DB write)
3. Add hash validation in `update_object` and `upsert_object`
4. Verify hash is included in `to_external` responses (it already should be since it's a regular field)
5. Update frontend to send `hash_code` back on edits
6. Update CLI `apply` command to send `hash_code`
7. Add integration tests for 409 scenarios

---

## 2. Resource History Integration - DONE

**Goal**: Wire up `write_history_entry` into create/update flows; add `?with_history=true` query param to GET.

**Effort**: Medium — backend only, isolated changes.

### Files to Modify

| File | Change |
|------|--------|
| `backend/src/api/v1/gitops.rs` | Call `db.write_history_entry()` after successful create/update/upsert; add `with_history` query param to `get_object` |
| `backend/src/db/arangodb/mod.rs` | Add `get_latest_history_entry()` method for fetching most recent history entry for a resource |

### Exact Changes

**`gitops.rs` — after successful `create_object` (line ~158)**:
```rust
// After after_create succeeds, write initial history entry
let snapshot = state.db.generic_get(&kind, &final_id).await?;
if let Some(snap) = snapshot {
    if let Err(e) = state.db.write_history_entry(&kind, &final_id, snap, &user_id).await {
        log::error!("[HANDLER] create_object: write_history_entry failed: kind={}, id={}, error={}", kind, final_id, e);
        // Non-fatal: history is supplementary, don't fail the create
    }
}
```

Same pattern for `update_object` (after line ~274) and `upsert_object` (after line ~230).

**`mod.rs` — new `get_latest_history_entry` method**:
```rust
pub async fn get_latest_history_entry(&self, kind: &str, key: &str) -> Result<Option<Value>> {
    let query = r#"
        FOR h IN resource_history
            FILTER h.resource_kind == @kind AND h.resource_key == @key
            SORT h.revision DESC
            LIMIT 1
            RETURN h
    "#;
    let vars = std::collections::HashMap::from([
        ("kind", Value::String(kind.to_string())),
        ("key", Value::String(key.to_string())),
    ]);
    let mut result: Vec<Value> = self.aql(query, vars).await?;
    Ok(result.pop())
}
```

**`gitops.rs` — `get_object` with history param**:
```rust
pub async fn get_object(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((kind, id)): Path<(String, String)>,
    Query(params): Query<HashMap<String, String>>,  // NEW
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    // ... existing ACL logic ...
    let mut result = ctrl.to_external(d);

    // NEW: optionally attach latest history entry
    if params.get("with_history").map(|v| v == "true").unwrap_or(false) {
        if let Ok(Some(history)) = state.db.get_latest_history_entry(&kind, &id).await {
            result.as_object_mut().map(|obj| obj.insert("_history".to_string(), history));
        }
    }

    Ok(Json(result))
}
```

### Integration Tests (`backend/itests/tests/`)
- Create a resource, verify `resource_history` has revision 1
- Update the resource, verify revision 2 exists
- GET with `?with_history=true`, verify `_history` field present with correct revision
- GET without param, verify no `_history` field

---

## 3. Principal Caching (5s TTL)

**Goal**: Cache `get_user_principals()` results to avoid repeated graph traversals on every ACL check.

**Effort**: Medium — backend only, uses existing `CacheStore` infrastructure.

### Files to Modify

| File | Change |
|------|--------|
| `backend/src/cache.rs` | Add `PRINCIPALS_CACHE` constant and register it in `create_default_cache()` |
| `backend/src/db/arangodb/mod.rs` | `get_user_principals` needs access to `CacheStore` — either pass it as param or restructure |
| `backend/src/state.rs` | May need to move cache-aware principal lookup to `AppState` |

### Design Decision

`get_user_principals` lives on `ArangoDb` which doesn't hold a reference to `CacheStore`. Two options:

**Option A — Cache at call site (AppState helper)**: Add `get_cached_principals()` on `AppState` that wraps `db.get_user_principals()` with cache logic. Callers use `state.get_cached_principals()` instead of `state.db.get_user_principals()`.

**Recommended: Option A** — minimal changes, follows the existing `has_godmode()` pattern.

### Exact Changes

**`cache.rs`** — add constant and registration:
```rust
pub const PRINCIPALS_CACHE: &str = "principals";
pub const PRINCIPALS_TTL: Duration = Duration::from_secs(5);

// In create_default_cache():
store.register_cache(PRINCIPALS_CACHE, PRINCIPALS_TTL).await;
```

**`state.rs`** — add helper method:
```rust
impl AppState {
    pub async fn get_cached_principals(&self, user_id: &str) -> Result<Vec<String>, AppError> {
        // Check cache
        if let Some(cached) = self.cache.get(cache::PRINCIPALS_CACHE, user_id).await {
            if let Some(arr) = cached.as_array() {
                let principals: Vec<String> = arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
                return Ok(principals);
            }
        }

        // Cache miss — query DB
        let principals = self.db.get_user_principals(user_id).await
            .map_err(AppError::Internal)?;

        // Store in cache
        self.cache.set(
            cache::PRINCIPALS_CACHE,
            user_id.to_string(),
            json!(principals),
        ).await;

        Ok(principals)
    }
}
```

**Callers** — grep for `get_user_principals` and replace `state.db.get_user_principals(&user_id)` with `state.get_cached_principals(&user_id)`. Expected call sites:
- `backend/src/api/v1/gitops.rs` (list handlers)
- `backend/src/api/v1/scoped_gitops.rs`
- Any controller that resolves principals

### Documentation
Add a note to `CLAUDE.md` under Architecture:
> Group membership changes may take up to 5 seconds to propagate to permission checks. There is no cache invalidation — the system relies on TTL expiry. This is acceptable because group membership changes are infrequent.

---

## 4. Permission Batch Operations & Debug Endpoint

**Goal**: Add batch grant/revoke helpers and an admin debug endpoint for permission inspection.

**Effort**: Medium — backend only.

### Files to Modify

| File | Change |
|------|--------|
| `backend/src/db/arangodb/mod.rs` | Add `grant_permissions_batch()` and `revoke_permissions_batch()` methods |
| `backend/src/api/v1/mod.rs` | Add debug route registration |
| `backend/src/api/v1/debug.rs` (new) | Permission inspection endpoint |

### Exact Changes

**`mod.rs`** — batch grant:
```rust
/// Grant multiple permissions to a principal atomically.
/// Silently succeeds if the principal already has any of the permissions.
pub async fn grant_permissions_batch(&self, permissions: &[&str], principal: &str) -> Result<()> {
    let query = r#"
        FOR perm_key IN @permissions
            UPSERT { _key: perm_key }
            INSERT { _key: perm_key, principals: [@principal] }
            UPDATE { principals: UNION_DISTINCT(OLD.principals, [@principal]) }
            IN permissions
    "#;
    let vars = std::collections::HashMap::from([
        ("permissions", json!(permissions)),
        ("principal", Value::String(principal.to_string())),
    ]);
    upsert_with_retry(|| {
        let vars = vars.clone();
        async move { self.aql::<Value>(query, vars).await.map(|_| ()) }
    }).await
}
```

**`mod.rs`** — batch revoke:
```rust
/// Revoke multiple permissions from a principal atomically.
pub async fn revoke_permissions_batch(&self, permissions: &[&str], principal: &str) -> Result<()> {
    let query = r#"
        FOR perm_key IN @permissions
            LET perm = DOCUMENT("permissions", perm_key)
            FILTER perm != null
            UPDATE perm WITH {
                principals: REMOVE_VALUE(perm.principals, @principal)
            } IN permissions
    "#;
    let vars = std::collections::HashMap::from([
        ("permissions", json!(permissions)),
        ("principal", Value::String(principal.to_string())),
    ]);
    self.aql::<Value>(query, vars).await?;
    Ok(())
}
```

**`debug.rs`** (new file) — permission inspection endpoint:
```rust
/// GET /v1/debug/access?user={user_id}&resource={kind}/{id}&permission={perm_bits}
///
/// Godmode-only endpoint. Returns a JSON report:
/// - user's resolved principals (direct + transitive groups)
/// - user's super-permissions
/// - resource's ACL entries
/// - effective permission bits for this user on this resource
/// - whether access is granted and why (direct ACL match, group match, super-permission bypass)
```

This endpoint is gated on `godmode_middleware` (requires `ADM_GODMODE` super-permission), not a separate management token. It never modifies state.

### Integration Tests
- Test batch grant: grant 3 permissions, verify all present
- Test batch revoke: revoke 2 of 3, verify only 1 remains
- Test debug endpoint: create user + resource with specific ACL, verify report accuracy

---

## 5. Scoped ACL Refactor - DONE, not tested

**Goal**: Simplify `generic_list_scoped` by removing the `scope` field from ACL matching. Each resource checks its own ACL first, then falls back to the parent project's full ACL without scope filtering.

**Effort**: Large — core authorization model change, high risk.

### Files to Modify

| File | Change |
|------|--------|
| `backend/src/db/arangodb/mod.rs` | Simplify `generic_list_scoped` AQL — remove `resource_kind` param and scope filter |
| `backend/src/db/arangodb/mod.rs` | Simplify `generic_get_scoped` similarly |
| `backend/src/controllers/gitops_controller.rs` | Remove `resource_kind_name()` from `KindController` trait (or deprecate) |
| `backend/src/api/v1/scoped_gitops.rs` | Stop passing `resource_kind` to DB methods |
| `shared/src/util_models.rs` | Keep `scope: Option<String>` on `AccessControlList` for backwards compat but ignore it in matching |
| `CLAUDE.md` | Update ACL documentation |

### Exact Changes

**`generic_list_scoped` AQL — simplified**:
```sql
-- BEFORE (lines 997-1003):
LET effective_acl = LENGTH(doc.acl.list || []) > 0
    ? (doc.acl.list || [])
    : (
        FOR entry IN project_acl
            FILTER entry.scope == null OR entry.scope == "*" OR entry.scope == @resource_kind
            RETURN entry
    )

-- AFTER:
LET effective_acl = LENGTH(doc.acl.list || []) > 0
    ? (doc.acl.list || [])
    : project_acl
```

This removes the scope filtering entirely. If a document has its own ACL, use it. Otherwise, use the project's full ACL — all entries apply regardless of scope.

**Function signature change**:
```rust
// BEFORE:
pub async fn generic_list_scoped(&self, collection, project_id, principals, required_perm,
    resource_kind, super_bypass, fields, limit, cursor)

// AFTER:
pub async fn generic_list_scoped(&self, collection, project_id, principals, required_perm,
    super_bypass, fields, limit, cursor)
// resource_kind parameter removed
```

**`check_hybrid_acl` in `gitops_controller.rs`**:
```rust
// BEFORE (lines 114-136): filters project ACL by scope matching
// AFTER: use all project ACL entries when resource has no own ACL
fn check_hybrid_acl(doc, principals, required, project_acl) {
    let resource_acl = parse_acl(doc).map(|a| a.list).unwrap_or_default();
    let effective = if resource_acl.is_empty() { project_acl } else { &resource_acl };
    effective.check_permission(principals, required)
}
```

### Risk Mitigation
- **Keep `scope` field on the struct** — don't remove it from `AccessControlList`, just stop using it in queries. Old documents with `scope` set still deserialize fine.
- **Feature flag**: Consider adding a `LEGACY_SCOPED_ACL=true` env var that preserves old behavior during rollout.
- **Write comprehensive tests before refactoring**: Create test fixtures with scoped ACLs and verify behavior doesn't regress for the "no scope" / "wildcard scope" cases (which are the majority).

### Migration
No data migration needed — `scope` stays on documents, it's just ignored. New documents can omit it (it's `Option<String>`). Over time, stop setting scope on new ACL entries.

---

## 6. Scoped ACL Integration Tests

**Goal**: Verify the scoped ACL refactor (item 5) behaves correctly end-to-end via Python integration tests.

**Effort**: Medium — new test file only, no backend changes.

**Status**: `DefaultKindController.is_scoped()` was changed to `true` so that ad-hoc kinds (e.g. "tasks") route through the scoped endpoints without a dedicated controller. The backend changes from item 5 are complete.

### File to Create

`backend/itests/tests/scoped_gitops_test.py`

### Test Scenarios

| # | Scenario | Expected |
|---|----------|----------|
| 1 | Root creates a project with ROOT ACL for itself | 201 |
| 2 | Root creates a task under the project | 201 |
| 3 | Root lists tasks — sees the task | items non-empty |
| 4 | Root gets a single task | 200 |
| 5 | Root updates the task | 200 |
| 6 | Root deletes the task | 204 |
| 7 | Non-scoped kind (`users`) via scoped endpoint | 400 |
| 8 | Non-existent project → list/get/create → 404 | 404 |
| 9 | User with project READ permission can list and get | 200 |
| 10 | User with NO project ACL cannot list/get | 404 |
| 11 | User with NO project ACL cannot create | 404 |
| 12 | User with CREATE permission on project can create | 201 |
| 13 | **Scope field ignored (regression)**: project ACL entry has `scope="other"` — user can still list tasks | items non-empty |
| 14 | **Scope field ignored (regression)**: project ACL entry has `scope="other"` — user can still GET a task | 200 |
| 15 | Task with its own ACL — user in resource ACL but NOT in project ACL can GET | 200 |
| 16 | Task with its own ACL — user NOT in resource ACL cannot GET (even if in project ACL for other perms) | 404 |
| 17 | Cross-project isolation: task in project A is not visible when listing project B | items is empty |
| 18 | Root (godmode) bypasses all ACLs | 200/201 always |

### Key Fixtures

```python
@pytest.fixture(scope="module")
def admin_token():
    """Login as root (ADM_GODMODE, all bypasses)."""

@pytest.fixture(scope="module")
def project_owner():
    """Random user with USR_CREATE_PROJECTS super-perm granted by root."""

@pytest.fixture(scope="module")
def reader_user():
    """Random user — will be given READ-only on the test project."""

@pytest.fixture(scope="module")
def outsider_user():
    """Random user — never given any project ACL."""

@pytest.fixture(scope="module")
def test_project(admin_token, project_owner):
    """Create a project with ROOT ACL for project_owner; yield project_id; delete after."""

@pytest.fixture(scope="module")
def reader_project(admin_token, project_owner, reader_user):
    """Separate project where reader_user has READ permission (scope field set to unrelated value)."""
```

### Regression Test Detail (scenarios 13–14)

**Important: Principal Caching (5s TTL)**

The critical scenario: a project ACL entry carries `scope="other"` (a different resource kind). Under the old code this entry would be filtered out when listing "tasks", so the user would see an empty list. Under the new code the scope field is ignored, so the user sees their task.

```python
def test_scope_field_ignored_on_list(reader_user, reader_project, ...):
    # Project ACL: [{ permissions: READ, principals: [reader_id], scope: "other" }]
    # Task has no own ACL, so project ACL is used.
    # Old: scope="other" != "tasks" → filtered out → 0 items
    # New: scope ignored → 1 item
    resp = requests.get(f"{URL_PROJECTS}/{reader_project}/tasks", headers=...)
    assert resp.status_code == 200
    assert len(resp.json()["items"]) == 1
```

### Cleanup Requirements
- Every created resource (project, tasks) must be deleted in fixture teardown
- Use `yield` in all resource fixtures
- Inline DELETE for ad-hoc resources created within individual tests
