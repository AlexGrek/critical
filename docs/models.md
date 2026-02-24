# Models — How Resources Work

This document explains Critical's data model, the `#[crit_resource]` macro, and how to work with resources from the API and Rust code.

## The Resource Contract

Every entity in Critical is a **resource**. All resources share a standard set of fields injected by the `#[crit_resource]` attribute macro:

| Field | Type | Description |
|-------|------|-------------|
| `id` | `String` | ArangoDB `_key` — e.g. `u_alice`, `g_engineering` |
| `meta` | `ResourceMeta` | Created/updated timestamps, labels, annotations |
| `acl` | `AccessControlStore` | Per-document ACL _(omitted with `no_acl`)_ |
| `deletion` | `Option<DeletionInfo>` | `null` = active, present = soft-deleted |
| `hash_code` | `String` | FNV-1a hash of desired state (conflict detection) |

Kind-specific fields come after these injected fields.

---

## Defining a Resource

```rust
// In shared/src/data_models.rs

#[crit_derive::crit_resource(collection = "groups", prefix = "g_")]
pub struct Group {
    #[brief]
    pub name: String,
    pub description: Option<String>,
}
```

**Macro parameters:**

| Parameter | Required | Description |
|-----------|----------|-------------|
| `collection = "..."` | yes | ArangoDB collection name |
| `prefix = "..."` | yes | ID prefix, e.g. `"g_"` |
| `no_acl` | no | Skip injecting the `acl` field |

**`#[brief]` attribute on fields:** marks the field to be included in the list (brief) response. `id` and `meta` are always included in briefs. Fields without `#[brief]` are only in the full (describe) response.

**What the macro generates:**

- `GroupBrief` struct — only `#[brief]` fields
- `fn to_brief(&self) -> GroupBrief`
- `fn brief_field_names() -> &'static [&'static str]` — AQL `KEEP()` list for efficient projections
- `fn compute_hash(&self) -> String` — FNV-1a over desired-state JSON
- `fn collection_name() -> &'static str` — `"groups"`
- `fn id_prefix() -> &'static str` — `"g_"`

---

## Principal Types

All four principal types are equal-level nodes in the membership graph.

### `users` — Human accounts

```rust
#[crit_derive::crit_resource(collection = "users", prefix = "u_", no_acl)]
pub struct User {
    pub password_hash: String,  // bcrypt, stripped from all API responses
    #[brief]
    pub personal: PersonalInfo,
}

pub struct PersonalInfo {
    pub name: String,
    pub gender: String,
    pub job_title: String,
    pub manager: Option<String>,  // user ID of manager
}
```

Users use `no_acl` — access to user resources is controlled by super-permissions only, not per-document ACL.

**Brief fields:** `id`, `meta`, `personal`

---

### `groups` — Principal aggregators

```rust
#[crit_derive::crit_resource(collection = "groups", prefix = "g_")]
pub struct Group {
    #[brief]
    pub name: String,
    pub description: Option<String>,
}
```

Groups have a per-document ACL. Members of a group are stored in the `memberships` edge collection, not as a field on the group.

**Brief fields:** `id`, `meta`, `name`

---

### `service_accounts` — Non-human API principals

```rust
#[crit_derive::crit_resource(collection = "service_accounts", prefix = "sa_")]
pub struct ServiceAccount {
    #[brief]
    pub name: String,
    pub description: Option<String>,
    pub token_hash: String,  // bcrypt hash of API token
}
```

**Brief fields:** `id`, `meta`, `name`

---

### `pipeline_accounts` — CI/CD-scoped non-human principals

```rust
#[crit_derive::crit_resource(collection = "pipeline_accounts", prefix = "pa_")]
pub struct PipelineAccount {
    #[brief]
    pub name: String,
    pub description: Option<String>,
    pub scope: Option<String>,  // e.g., a project or pipeline name
    pub token_hash: String,
}
```

**Brief fields:** `id`, `meta`, `name`

---

## Projects

Projects are global resources that act as namespaces for all work items (tasks, pipelines, wikis, deployments, etc.). They have no ID prefix — plain identifiers serve directly as the namespace key.

```rust
#[crit_derive::crit_resource(collection = "projects", prefix = "")]
pub struct Project {
    #[brief]
    pub name: String,
    pub description: Option<String>,
    /// Source code repositories linked to this project.
    pub repositories: Vec<RepoLink>,  // omitted from JSON when empty
    /// Feature modules enabled for this project (controls visible UI tabs).
    pub enabled_services: Vec<ProjectService>,  // omitted from JSON when empty
}

pub struct RepoLink {
    pub url: String,
    pub provider: RepoProvider,   // default: Git
    pub name: Option<String>,
    pub default_branch: Option<String>,
}

pub enum RepoProvider {
    Git, Github, Gitlab, Bitbucket, Svn, Mercurial, Custom
}

pub enum ProjectService {
    Integrations,  // webhooks, GitHub Apps, third-party
    Pipelines,     // built-in CI/CD engine
    Deployments,   // state-controlled deployment management
    Secrets,       // vault alternative
    Wikis,         // git-backed docs (Confluence alternative)
    Apps,          // custom internal tools
    Tasks,         // issue tracking / kanban (JIRA alternative)
    Talks,         // team discussion boards
    Releases,      // version tagging + changelogs
    Environments,  // dev/staging/prod config management
    Insights,      // analytics and burndown charts
}
```

**Brief fields:** `id`, `meta`, `name`

### Project — full document

```json
{
  "id": "api-v2",
  "meta": {
    "labels": { "team": "platform" },
    "annotations": {},
    "created_at": "2026-02-24T10:00:00Z",
    "created_by": "u_alice",
    "updated_at": "2026-02-24T10:00:00Z",
    "updated_by": "u_alice"
  },
  "acl": {
    "list": [
      { "permissions": 127, "principals": ["u_alice"] },
      { "permissions": 7,   "principals": ["g_engineering"] }
    ],
    "last_mod_date": "2026-02-24T10:00:00Z"
  },
  "deletion": null,
  "hash_code": "b2c3d4e5f6789012",
  "name": "API v2",
  "description": "Next generation API project",
  "repositories": [
    {
      "url": "https://github.com/acme/api-v2",
      "provider": "github",
      "name": "Main repo",
      "default_branch": "main"
    }
  ],
  "enabled_services": ["tasks", "pipelines", "wikis", "deployments"]
}
```

---

## Full Resource Shape (JSON)

### Group — full document

```json
{
  "id": "g_engineering",
  "meta": {
    "labels": { "team": "platform" },
    "annotations": {},
    "created_at": "2026-02-23T10:00:00Z",
    "created_by": "u_alice",
    "updated_at": "2026-02-23T10:00:00Z",
    "updated_by": "u_alice"
  },
  "acl": {
    "list": [
      { "permissions": 127, "principals": ["u_alice"] },
      { "permissions": 7,   "principals": ["g_leads"] }
    ],
    "last_mod_date": "2026-02-23T10:00:00Z"
  },
  "deletion": null,
  "hash_code": "a1b2c3d4e5f67890",
  "name": "engineering",
  "description": "Main engineering team"
}
```

### Group — brief (list view)

```json
{
  "id": "g_engineering",
  "meta": { "created_at": "...", "created_by": "u_alice", ... },
  "name": "engineering"
}
```

### User — full document (API response, password_hash stripped)

```json
{
  "id": "u_alice",
  "meta": {
    "labels": {},
    "annotations": { "registered_at": "2026-02-23T10:00:00Z" },
    "created_at": "2026-02-23T10:00:00Z",
    "created_by": null,
    "updated_at": "2026-02-23T10:00:00Z",
    "updated_by": null
  },
  "deletion": null,
  "hash_code": "f1e2d3c4b5a69870",
  "personal": {
    "name": "Alice Example",
    "gender": "",
    "job_title": "Engineer",
    "manager": null
  }
}
```

---

## Soft Deletion

Resources are never hard-deleted. Deleting a resource sets its `deletion` field:

```json
{
  "deletion": {
    "deleted_at": "2026-02-23T12:00:00Z",
    "deleted_by": "u_admin",
    "disconnected_edges": [
      {
        "collection": "memberships",
        "key": "u_alice::g_engineering",
        "from": "users/u_alice",
        "to": "groups/g_engineering"
      }
    ]
  }
}
```

- `disconnected_edges` captures membership edges at the time of deletion (for future restore support).
- All list and get queries filter `doc.deletion == null` — soft-deleted documents are invisible by default.
- Cascading: deleting a user removes them from all groups. If a group becomes empty, it is also soft-deleted (recursively).

---

## Metadata (`ResourceMeta`)

Every resource carries `meta`:

```json
{
  "labels": { "env": "prod", "team": "platform" },
  "annotations": { "jira-link": "CRIT-123" },
  "created_at": "2026-02-23T10:00:00Z",
  "created_by": "u_alice",
  "updated_at": "2026-02-23T11:00:00Z",
  "updated_by": "u_bob"
}
```

- **Labels**: queryable key-value pairs (future: `-l key=value` selector support).
- **Annotations**: non-queryable freeform strings (links, notes, etc.).
- `created_by` / `updated_by` are principal IDs (set automatically by the backend).

---

## Access Control (`AccessControlStore`)

Resources with `acl` (groups, service accounts, pipeline accounts) embed per-document permissions:

```json
{
  "acl": {
    "list": [
      { "permissions": 127, "principals": ["u_alice"] },
      { "permissions": 7,   "principals": ["g_leads", "sa_ci"] }
    ],
    "last_mod_date": "2026-02-23T10:00:00Z"
  }
}
```

Permissions are bitflags stored as integers:

| Name | Value | Includes |
|------|-------|---------|
| FETCH | 1 | Read single document |
| LIST | 2 | Appear in listings |
| NOTIFY | 4 | Real-time events |
| CREATE | 8 | Create child resources |
| MODIFY | 16 | Update or delete |
| READ | 7 | FETCH + LIST + NOTIFY |
| WRITE | 31 | READ + CREATE + MODIFY |
| ROOT | 127 | All bits |

Group membership resolves transitively: if `u_alice` is a member of `g_leads`, and `g_leads` appears in an ACL entry, Alice gets those permissions (up to 10 levels of nesting).

---

## Membership Edge

Group membership is stored in the `memberships` edge collection, not as a field on a resource:

```json
{
  "_key": "u_alice::g_engineering",
  "_from": "users/u_alice",
  "_to": "groups/g_engineering",
  "principal": "u_alice",
  "group": "g_engineering"
}
```

Any principal can be a member of a group — users, groups (nesting), service accounts, and pipeline accounts all use the same edge collection.

---

## Change History

Every create and update writes an immutable snapshot to `resource_history`:

```json
{
  "_key": "groups_g_engineering_000003",
  "resource_kind": "groups",
  "resource_key": "g_engineering",
  "revision": 3,
  "snapshot": { "...full desired-state at this revision..." },
  "changed_by": "u_alice",
  "changed_at": "2026-02-23T11:00:00Z"
}
```

Revisions are 1-based and monotonically increasing per resource. History survives resource deletion.

---

## Runtime Events

Non-desired-state occurrences are stored in `resource_events`:

```json
{
  "_key": "ev_sign_in_1740312000123456789",
  "resource_kind": "users",
  "resource_key": "u_alice",
  "event_type": "sign_in",
  "timestamp": "2026-02-23T10:00:00Z",
  "actor": "u_alice",
  "details": null
}
```

Currently recorded events:

| Event | Kind | Trigger |
|-------|------|---------|
| `sign_in` | `users` | Successful password authentication |

Events survive resource deletion.

---

## API Usage Examples

### Create a group

```http
POST /api/v1/global/groups
Content-Type: application/json
Authorization: Bearer <token>

{
  "id": "my-team",
  "name": "My Team",
  "description": "Optional description"
}
```

Response `201 Created`:
```json
{ "id": "g_my-team" }
```

The `g_` prefix is added automatically. The creator is added to the group ACL with ROOT and becomes a member.

### List groups (brief)

```http
GET /api/v1/global/groups
Authorization: Bearer <token>
```

```json
{
  "items": [
    { "id": "g_engineering", "meta": { ... }, "name": "engineering" },
    { "id": "g_leads",       "meta": { ... }, "name": "leads" }
  ]
}
```

Only groups where your ACL grants READ (or you have `adm_user_manager`) are returned.

### Get a single group (full)

```http
GET /api/v1/global/groups/g_engineering
Authorization: Bearer <token>
```

Returns the full document including `acl`, `meta`, `description`, `deletion`, `hash_code`.

### Update a group

```http
PUT /api/v1/global/groups/g_engineering
Content-Type: application/json
Authorization: Bearer <token>

{
  "name": "Engineering",
  "description": "Updated description"
}
```

Requires MODIFY on the group's ACL (or `adm_user_manager`).

### Delete a group

```http
DELETE /api/v1/global/groups/g_engineering
Authorization: Bearer <token>
```

Soft-deletes the group. Membership edges are captured in `disconnected_edges` and removed. If any parent group becomes empty, it is also soft-deleted.

### Add a member to a group

```http
POST /api/v1/global/memberships
Content-Type: application/json
Authorization: Bearer <token>

{
  "id": "u_bob::g_engineering",
  "principal": "u_bob",
  "group": "g_engineering"
}
```

Requires MODIFY on the target group's ACL.

---

## Rust Usage Examples

### Constructing a resource

```rust
use crit_shared::data_models::{Group, PersonalInfo};
use crit_shared::util_models::ResourceMeta;

let group = Group {
    id: "g_platform".to_string(),
    meta: ResourceMeta {
        created_at: chrono::Utc::now(),
        created_by: Some("u_alice".to_string()),
        ..Default::default()
    },
    name: "platform".to_string(),
    description: Some("Platform team".to_string()),
    // acl, deletion, hash_code use Default (injected by macro)
    ..Default::default()
};
```

### Using the generated methods

```rust
// Brief view (for list responses)
let brief = group.to_brief();

// Field names for AQL KEEP() projections
let fields = Group::brief_field_names(); // &["_key", "name", "acl", "meta"]

// Collection and prefix info
let coll = Group::collection_name(); // "groups"
let prefix = Group::id_prefix();     // "g_"

// Compute hash (call before storing to DB)
let hash = group.compute_hash();     // "a1b2c3d4e5f67890"
```

### Checking ACL from Rust

```rust
use crit_shared::util_models::Permissions;

let acl = &group.acl;
let user_principals = vec!["u_alice".to_string(), "g_platform".to_string()];

let can_read   = acl.check_permission(&user_principals, Permissions::READ);
let can_modify = acl.check_permission(&user_principals, Permissions::MODIFY);
```

### Writing a history entry (from a controller)

```rust
// After creating or updating a resource:
let snapshot = serde_json::to_value(&my_resource)?;
db.write_history_entry("groups", "g_engineering", snapshot, "u_alice").await?;
```

### Writing an event (from a handler)

```rust
// Non-fatal: login still succeeds if event writing fails
let _ = db.write_event("users", "u_alice", "sign_in", Some("u_alice"), None).await;
```
