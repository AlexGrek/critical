# Big Model Refactoring Plan

## Overview

Replace the current `#[derive(Brief)]` derive macro with a powerful `#[crit_resource(...)]` **attribute macro** that wraps structs and injects standard fields. Simultaneously rework the data model to support soft deletion, graph-native principals, change history, events, hash-based conflict detection, and server-injected state.

Only **Users** and **Groups** will remain active. Projects will be commented out. Everything else (Tasks, Sprints, Features, Pipelines, etc.) will be removed entirely.

---

## Phase 1: Cleanup — Remove unused models and infrastructure

### 1a. Remove models from `shared/src/data_models.rs`
- **Delete entirely**: `Task`, `TaskState`, `Priority`, `Sprint`, `SprintState`, `Feature`, `Pipeline`, `PipelineRun`, `RunState`, `Artifact`, `Deployment`, `Release`, `ReleaseState`, `Page`, `Policy`, `ResourceRevision`
- **Comment out** (with `// TODO: re-enable after namespace rework`): `Project`
- **Keep as-is for now** (will be refactored in Phase 3): `User`, `Group`, `GroupMembership`, `GlobalPermission`, `PersonalInfo`

### 1b. Remove unused util models from `shared/src/util_models.rs`
- **Delete**: `LifecycleState`, `RelationKind`, `Relation`
- **Trim super_permissions**: Remove `ADM_PROJECT_MANAGER`, `USR_CREATE_PROJECTS`, `USR_CREATE_PIPELINES`, `ADM_POLICY_EDITOR`, `ADM_RELEASE_MANAGER`. Keep `ADM_USER_MANAGER`, `ADM_CONFIG_EDITOR`, `USR_CREATE_GROUPS`.

### 1c. Remove controllers
- **Delete files**: `project_controller.rs`, `task_controller.rs`
- **Update** `controllers/mod.rs`: Remove Project/Task controller fields, imports, and `for_kind()` match arms

### 1d. Remove DB collections from `backend/src/db/arangodb/mod.rs`
- Remove from `ArangoDb` struct fields: `projects`, `relations`, `tasks`, `sprints`, `features`, `pipelines`, `pipeline_runs`, `artifacts`, `deployments`, `releases`, `pages`, `policies`, `revisions`
- Remove corresponding `create_collection`/`create_edge_collection` calls from `connect_basic`, `connect_anon`, `connect_jwt`
- Remove from `begin_transaction` write collections list
- Remove unused super_permissions from `seed_permissions()`
- Keep: `users`, `groups`, `memberships`, `permissions`

### 1e. Remove tests referencing deleted models
- Remove Python itests: `task_test.py`, project-specific tests in `gitops_permissions_test.py`
- Update `gitops_permissions_test.py` to remove project/task tests (keep user/group permission tests)
- Remove any Rust tests that reference Task, Project, Sprint, etc.

---

## Phase 2: New shared types in `shared/src/util_models.rs`

### 2a. Add `DeletionInfo` (replaces `LifecycleState`)
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeletionInfo {
    pub deleted_at: DateTime<Utc>,
    pub deleted_by: PrincipalId,
}
```

### 2b. Add `RuntimeState` — server-injected runtime data
```rust
/// Server-injected runtime data, NOT part of desired-state or history.
/// Used for computed/dynamic fields like last_login, member_count, etc.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct RuntimeState {
    #[serde(flatten)]
    pub fields: HashMap<String, Value>,
}
```

Note: `ResourceState` is now used for server-managed audit timestamps (`created_at`, `created_by`, `updated_at`, `updated_by`) — injected into every resource by the `#[crit_resource]` macro.

### 2c. Add `HistoryEntry` — stored in `resource_history` collection
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistoryEntry {
    #[serde(rename = "_key")]
    pub id: String, // "{collection}_{resource_key}_{revision}"
    pub resource_kind: String,
    pub resource_key: String,
    pub revision: u64,
    pub snapshot: Value, // full desired-state at this point
    pub changed_by: PrincipalId,
    pub changed_at: DateTime<Utc>,
}
```

### 2d. Add `ResourceEvent` — stored in `resource_events` collection
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResourceEvent {
    #[serde(rename = "_key")]
    pub id: String, // UUID v7
    pub resource_kind: String,
    pub resource_key: String,
    pub event_type: String, // e.g., "sign_in"
    pub timestamp: DateTime<Utc>,
    pub actor: Option<PrincipalId>,
    pub details: Option<Value>,
}
```

### 2e. Add `FullResource` — the "describe" envelope
```rust
/// Envelope for full resource representation with optional extras.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FullResource {
    /// The resource itself (desired state + audit state), flattened
    #[serde(flatten)]
    pub resource: Value,
    /// Server-injected runtime data (member_count, last_login, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_state: Option<RuntimeState>,
    /// Change history (oldest first)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<Vec<HistoryEntry>>,
    /// Runtime events
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events: Option<Vec<ResourceEvent>>,
}
```

---

## Phase 3: The `#[crit_resource(...)]` attribute macro

### 3a. Macro design

Replaces the old `#[derive(Brief)]`. The user writes:

```rust
#[crit_resource(collection = "users", prefix = "u_")]
pub struct User {
    pub password_hash: String,
    #[brief]
    pub personal: PersonalInfo,
}
```

The macro **injects** these standard fields into the struct (at the top, before user fields):

```rust
pub struct User {
    // ---- injected by macro ----
    #[serde(rename = "_key")]
    pub id: PrincipalId,
    #[serde(default)]
    pub labels: Labels,
    #[serde(default)]
    pub annotations: Labels,
    #[serde(default)]
    pub state: ResourceState,
    #[serde(default)]
    pub acl: AccessControlStore,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deletion: Option<DeletionInfo>,
    #[serde(default)]
    pub hash_code: String,
    // ---- user-defined fields below ----
    pub password_hash: String,
    #[brief]
    pub personal: PersonalInfo,
}
```

The macro also **generates**:
1. `{Name}Brief` struct (from `#[brief]` fields, including injected ones like `id`, `labels`, and `annotations`)
2. `to_brief(&self) -> {Name}Brief`
3. `brief_field_names() -> &'static [&'static str]`
4. `compute_hash(&self) -> String` — FNV-1a over JSON of desired-state fields (everything except `hash_code` itself, `deletion`, `state`)
5. `collection_name() -> &'static str` — returns the collection name
6. `id_prefix() -> &'static str` — returns the prefix

### 3b. Macro parameters
- `collection = "..."` (required) — ArangoDB collection name
- `prefix = "..."` (required) — ID prefix (e.g., `"u_"`, `"g_"`, `"sa_"`, `"pa_"`)
- `no_acl` (optional flag) — skip injecting the `acl` field (for resources that don't have per-resource ACL, like users)

### 3c. Implementation in `shared/derive/src/lib.rs`
- Add `syn` `full` feature (already present)
- Parse item struct, inject fields, re-emit with derives `(Debug, Serialize, Deserialize, Clone)`
- Generate companion code (Brief struct, hash, static methods)
- Use `const_fnv1a_hash` or inline FNV-1a implementation (no external dep needed for such a simple algorithm)

### 3d. Hash computation
- Serialize the struct to `serde_json::Value`, remove `hash_code`, `deletion`, and any non-desired-state fields
- Run FNV-1a on the canonical JSON string
- Store as 16-char hex string (64-bit)
- Provide a `with_computed_hash(&mut self)` method that sets `self.hash_code`

---

## Phase 4: Refactor User and Group models

### 4a. User model
```rust
#[crit_resource(collection = "users", prefix = "u_", no_acl)]
pub struct User {
    pub password_hash: String,
    #[brief]
    pub personal: PersonalInfo,
}
```

Users don't have per-resource ACL (access is controlled by super-permissions). The `no_acl` flag skips `acl` field injection.

### 4b. Group model
```rust
#[crit_resource(collection = "groups", prefix = "g_")]
pub struct Group {
    #[brief]
    pub name: String,
    pub description: Option<String>,
}
```

Groups get `acl` injected automatically.

### 4c. GroupMembership — stays manual (edge collection, special serde)
No macro. Keep manual definition but ensure it references `PrincipalId` correctly.

### 4d. GlobalPermission — stays manual (simple key-value, no lifecycle)

---

## Phase 5: New principal types (stub collections)

### 5a. ServiceAccount model
```rust
#[crit_resource(collection = "service_accounts", prefix = "sa_")]
pub struct ServiceAccount {
    #[brief]
    pub name: String,
    pub description: Option<String>,
    /// Hashed API token
    pub token_hash: String,
}
```

### 5b. PipelineAccount model
```rust
#[crit_resource(collection = "pipeline_accounts", prefix = "pa_")]
pub struct PipelineAccount {
    #[brief]
    pub name: String,
    pub description: Option<String>,
    /// Scoped to a specific pipeline/project
    pub scope: Option<String>,
    pub token_hash: String,
}
```

### 5c. Update `add_principal_to_group` to resolve prefix → collection
```rust
fn collection_for_principal(principal_id: &str) -> &'static str {
    if principal_id.starts_with("g_") { "groups" }
    else if principal_id.starts_with("sa_") { "service_accounts" }
    else if principal_id.starts_with("pa_") { "pipeline_accounts" }
    else { "users" } // u_ prefix or fallback
}
```

---

## Phase 6: Database changes

### 6a. New collections in `ArangoDb`
- Add: `service_accounts`, `pipeline_accounts`, `resource_history`, `resource_events`
- Add to struct, `connect_basic`, `connect_anon`, `connect_jwt`, `begin_transaction`

### 6b. ArangoDB named graph for principal traversal
Create a named graph `principal_graph` with edge collection `memberships`:
- From collections: `users`, `groups`, `service_accounts`, `pipeline_accounts`
- To collection: `groups`
This enables `FOR v IN 1..10 OUTBOUND ... memberships` across all principal types.

Add graph creation in `connect_basic` (idempotent):
```rust
// Create named graph (ignore if exists)
let _ = self.db.create_graph("principal_graph", /* edge definitions */);
```

### 6c. Soft deletion filter in queries
- `generic_list`: add `FILTER doc.deletion == null` by default. Add optional `include_deleted` parameter.
- `generic_get`: add `FILTER doc.deletion == null` by default. Add optional `include_deleted` parameter.
- New method `generic_soft_delete`: sets `deletion` field instead of removing doc, plus disconnects from graphs.
- New method `generic_restore`: clears `deletion` field, re-links valid edges.

### 6d. History write helper
```rust
pub async fn write_history_entry(&self, kind: &str, key: &str, snapshot: Value, user_id: &str) -> Result<()>
```
Called by controllers after create/update. Stores snapshot in `resource_history`.

### 6e. Event write helper
```rust
pub async fn write_event(&self, kind: &str, key: &str, event_type: &str, actor: Option<&str>, details: Option<Value>) -> Result<()>
```
Used for login events, etc.

### 6f. Soft deletion graph disconnection
When soft-deleting a resource:
1. Set `deletion` field on the document
2. Remove all edges where this document is `_from` or `_to` in `memberships`
3. Store removed edges in a `_deletion_edges` field on the deletion info (so restore can re-link)

When restoring:
1. Clear `deletion` field
2. For each edge in `_deletion_edges`, check if the other end still exists and is not deleted
3. Re-create valid edges, skip deleted targets
4. Clear `_deletion_edges`

---

## Phase 7: Controller updates

### 7a. Update `UserController` and `GroupController`
- Use `User::collection_name()`, `User::id_prefix()` from macro-generated code
- Update `prepare_create` to use new field layout (meta is injected, acl for groups)
- Add history writing in `after_create` and `after_update`
- Add login event writing in login handler

### 7b. Update `KindController` trait
- Add `fn collection_name(&self) -> &str` method (with default returning kind name)
- `after_delete` becomes soft-delete-aware (disconnects graphs, writes history)

### 7c. Update gitops route handlers
- `generic_list` and `generic_get` filter out deleted by default
- Support `?deleted=true` query param to include deleted resources

---

## Phase 8: Login event recording

In `backend/src/api/auth.rs` (or wherever login is handled):
- After successful JWT generation, call `db.write_event("users", user_key, "sign_in", Some(user_key), None)`

---

## Phase 9: Documentation updates

### 9a. Update `DATABASE.md`
- Remove all deleted collection entries
- Add `service_accounts`, `pipeline_accounts`, `resource_history`, `resource_events`
- Document `principal_graph` named graph
- Update User/Group schemas to reflect new fields (deletion, hash_code)
- Note Projects are planned/commented-out, no prefix

### 9b. Update `WHITEPAPER.md`
- Add principal types section (users, groups, service_accounts, pipeline_accounts)
- Document soft deletion model
- Document change history and events
- Update relation examples to reflect current state
- Note projects-as-namespaces is planned, no prefix

### 9c. Update `CLAUDE.md`
- Update model/field references
- Document `#[crit_resource]` macro usage
- Update collection list
- Add service_accounts/pipeline_accounts to ID conventions

---

## Execution order (dependency-aware)

1. **Phase 1** (cleanup) — remove dead code so we have a clean slate
2. **Phase 2** (new shared types) — foundation types needed by everything else
3. **Phase 3** (attribute macro) — the core deliverable, needed before model refactoring
4. **Phase 4** (refactor User/Group) — apply the macro to existing models
5. **Phase 5** (new principal stubs) — models only, minimal DB usage
6. **Phase 6** (database) — new collections, queries, soft delete, history/events
7. **Phase 7** (controllers) — wire everything together
8. **Phase 8** (login events) — small addition to auth handler
9. **Phase 9** (docs) — update all documentation last

**Tests will be updated in a separate pass** as the user requested focusing on refactoring first.

---

## Files touched (summary)

| File | Action |
|------|--------|
| `shared/derive/src/lib.rs` | Major rewrite: attribute macro |
| `shared/derive/Cargo.toml` | No changes expected |
| `shared/src/lib.rs` | Update re-export |
| `shared/src/data_models.rs` | Gut and rebuild with macro |
| `shared/src/util_models.rs` | Add DeletionInfo, HistoryEntry, ResourceEvent, FullResource; remove LifecycleState, RelationKind, Relation |
| `backend/src/db/arangodb/mod.rs` | Remove collections, add new ones, soft-delete queries, history/event helpers |
| `backend/src/controllers/mod.rs` | Remove Project/Task controllers |
| `backend/src/controllers/project_controller.rs` | Delete |
| `backend/src/controllers/task_controller.rs` | Delete |
| `backend/src/controllers/user_controller.rs` | Update for new model shape |
| `backend/src/controllers/group_controller.rs` | Update for new model shape |
| `backend/src/controllers/gitops_controller.rs` | Update KindController trait, soft-delete support |
| `backend/itests/tests/task_test.py` | Delete |
| `backend/itests/tests/gitops_permissions_test.py` | Remove project/task tests |
| `DATABASE.md` | Major rewrite |
| `WHITEPAPER.md` | Update |
| `CLAUDE.md` | Update |
