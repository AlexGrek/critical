# Database Schema

> **Database**: ArangoDB (RocksDB storage engine)
> **Default DB Name**: `unnamed` (configurable via `DB_NAME` env var)
> **Connection**: `http://localhost:8529` (configurable via `DB_CONNECTION_STRING`)

Collections are auto-created on first connection via `ArangoDb::connect_basic()`. There is no migration system — schema is defined by Rust types in `shared/src/data_models.rs` and `shared/src/util_models.rs`, and ArangoDB itself is schemaless (documents are JSON with no DB-enforced structure).

## Schema Evolution

There are no migration files, no versioning, and no automated schema diffing. The Rust structs in `shared/src/data_models.rs` and `shared/src/util_models.rs` define the **application-level** schema, not a DB-enforced one.

**On startup**, `connect_basic` creates collections if they don't exist (idempotent `create_collection` calls that silently ignore "already exists" errors). Existing collections and documents are never altered.

**How struct changes affect existing data:**

| Change | Effect | Action needed |
|--------|--------|---------------|
| Add `Option<T>` field | Old documents deserialize with `None` | None |
| Add field with `#[serde(default)]` | Old documents get the default value | None |
| Add required field (no default) | Old documents fail to deserialize | Manual backfill or add default |
| Remove a field | Old documents keep the extra field in DB, ignored on read | None (orphaned data remains) |
| Rename a field | Old documents have the old field name, new writes use the new name | Manual data fixup |
| Add a new collection | Must add `create_collection` call in `connect_basic` and `connect_anon`, plus a field on `ArangoDb` | Code change required |

## Entity-Relationship Diagram

```
┌──────────────────┐
│     users        │  u_ prefix
│  (Document)      │
└────────┬─────────┘
         │                      ┌──────────────────┐
┌────────┴─────────┐            │     groups       │  g_ prefix
│ service_accounts │  sa_       │  (Document)      │
│  (Document)      │            └────────┬─────────┘
└────────┬─────────┘                     │
         │            ┌──────────────────┘
┌────────┴─────────┐  │
│pipeline_accounts │  pa_
│  (Document)      │  │
└────────┬─────────┘  │
         │            │
         └─────┬──────┘
               │  _from (any principal)
        ┌──────▼───────┐
        │ memberships  │  (Edge)
        │  _from → _to │──────▶ groups
        └──────────────┘

┌──────────────────┐   ┌──────────────────┐
│ resource_history │   │ resource_events  │
│  (Document)      │   │  (Document)      │
│  immutable       │   │  runtime events  │
│  snapshots       │   │  (login, etc.)   │
└──────────────────┘   └──────────────────┘

┌──────────────────┐
│  permissions     │  (Document)
│  super-perms     │
│  principals[]    │
└──────────────────┘

┌──────────────────────────────────────────┐
│  projects        │  no prefix            │
│  (Document)      │  (namespace resource) │
│  name, desc      │                       │
│  repositories[]  │  RepoLink sub-type    │
│  enabled_services[] ProjectService enum  │
└──────────────────────────────────────────┘
```

### Membership Graph

```
 ┌─────────┐                  ┌────────────┐
 │ u_alice │──membership──┐   │ sa_ci-bot  │──membership──┐
 └─────────┘              │   └────────────┘              │
                          ▼                               ▼
                    ┌──────────┐                    ┌──────────┐
                    │ g_devs   │──membership──────▶│ g_all    │
                    └──────────┘                    └──────────┘

 All four principal types (users, groups, service_accounts, pipeline_accounts)
 can be members of groups. Groups can be nested (groups as members of groups).
```

## Collections

### `users` — Document Collection

| Key Field | Key Format | Rust Struct |
|-----------|------------|-------------|
| `_key` | `u_` prefix (e.g. `u_alice`) | `User` |

Standard fields injected by `#[crit_resource]`: `id`, `meta`, `deletion`, `hash_code`. No `acl` (users use super-permissions only).

### `groups` — Document Collection

| Key Field | Key Format | Rust Struct |
|-----------|------------|-------------|
| `_key` | `g_` prefix (e.g. `g_admins`) | `Group` |

Has full `acl` field. Members are edges in `memberships`, not a field on the group document.

### `service_accounts` — Document Collection

| Key Field | Key Format | Rust Struct |
|-----------|------------|-------------|
| `_key` | `sa_` prefix (e.g. `sa_ci-bot`) | `ServiceAccount` |

Non-human API principals for integrations. Authenticate via token.

### `pipeline_accounts` — Document Collection

| Key Field | Key Format | Rust Struct |
|-----------|------------|-------------|
| `_key` | `pa_` prefix (e.g. `pa_build-runner`) | `PipelineAccount` |

Non-human CI/CD principals. Same as service accounts but scoped to a specific pipeline or project.

### `memberships` — Edge Collection

| Key Field | Key Format | Rust Struct |
|-----------|------------|-------------|
| `_key` | `{principal}::{group}` composite | `GroupMembership` |

**Edge fields**: `_from` (any principal collection) → `_to` (`groups/{id}`). Also stores `principal` and `group` as plain string fields for direct queries.

Native graph traversal: `FOR v IN 1..10 OUTBOUND "users/u_alice" memberships`

**Key AQL queries**:
- All members of a group: `FOR m IN memberships FILTER m.group == @group RETURN m.principal`
- Users only: `FOR m IN memberships FILTER m.group == @group FILTER LIKE(m.principal, "u_%") RETURN m.principal`
- All groups for a principal (recursive): `FOR v IN 1..10 OUTBOUND CONCAT("users/", @user) memberships RETURN v._key`

### `permissions` — Document Collection

| Key Field | Key Format | Rust Struct |
|-----------|------------|-------------|
| `_key` | permission name (e.g. `adm_user_manager`) | `GlobalPermission` |

**Fields**: `principals` (`Vec<String>`) — list of principal IDs granted this super-permission.

**Active super-permissions**:

| Permission | Granted on Registration | Description |
|------------|------------------------|-------------|
| `adm_user_manager` | No | Full control over users, groups, memberships |
| `adm_config_editor` | No | Edit global configuration |
| `usr_create_groups` | Yes | Create new groups |

Permission checks resolve through the membership graph — a principal has a permission if they or any of their groups (including nested groups, up to 10 levels) appear in that permission's `principals` array.

### `projects` — Document Collection

| Key Field | Key Format | Rust Struct |
|-----------|------------|-------------|
| `_key` | No prefix — plain identifier (e.g. `api-v2`, `mobile-app`) | `Project` |

Has full `acl` field. Projects act as namespaces for work items (tasks, pipelines, wikis, deployments, etc.).

**Key fields**: `name`, `description`, `repositories` (Vec of `RepoLink`), `enabled_services` (Vec of `ProjectService`).

**`enabled_services`** controls which feature tabs are visible in the UI per project. Possible values (snake_case):

| Value | Feature |
|-------|---------|
| `integrations` | Webhooks, GitHub Apps, third-party integrations |
| `pipelines` | Built-in CI/CD pipeline engine |
| `deployments` | State-controlled deployment management |
| `secrets` | Secret management (Vault alternative) |
| `wikis` | Git-backed documentation (Confluence alternative) |
| `apps` | Custom internal tools |
| `tasks` | Issue tracking / kanban (JIRA alternative) |
| `talks` | Team discussion boards |
| `releases` | Version tagging and changelogs |
| `environments` | Dev/staging/prod config management |
| `insights` | Analytics and burndown charts |

### `resource_history` — Document Collection

Immutable change snapshots. Written after every create/update. Survives resource deletion.

| Key format | Example |
|------------|---------|
| `{kind}_{resource_key}_{revision:06}` | `groups_g_engineering_000003` |

Fields: `resource_kind`, `resource_key`, `revision` (1-based), `snapshot` (full JSON), `changed_by`, `changed_at`.

### `resource_events` — Document Collection

Runtime events associated with resources. Survives resource deletion.

| Key format | Example |
|------------|---------|
| `ev_{event_type}_{timestamp_ns}` | `ev_sign_in_1740312000123456789` |

Fields: `resource_kind`, `resource_key`, `event_type`, `timestamp`, `actor`, `details`.

## Indexes

ArangoDB auto-indexes `_key`, and auto-indexes `_from`/`_to` on edge collections. No additional explicit indexes defined. Candidates for future manual indexes:
- `memberships.principal`
- `memberships.group`
- `resource_history.resource_key` (for fast history lookups)
- `resource_events.resource_key`

## Transactions

Active document collections participate in server-side transactions with `wait_for_sync: true`: `users`, `groups`, `service_accounts`, `pipeline_accounts`, `memberships`, `permissions`, `resource_history`, `resource_events`, `projects`.

## Conventions

| Convention | Detail |
|------------|--------|
| **ID prefixes** | Users: `u_`, Groups: `g_`, Service accounts: `sa_`, Pipeline accounts: `pa_` |
| **Serde rename** | Rust `id` → ArangoDB `_key` |
| **Soft deletes** | `deletion: Option<DeletionInfo>` — `null` = active, present = deleted |
| **Deleted edge capture** | `DeletionInfo.disconnected_edges` stores removed membership edges for restore |
| **Composite edge keys** | `{principal}::{group}` |
| **Auto-creation** | Collections created on startup if missing (idempotent) |
| **No migrations** | Schema defined entirely by Rust structs |
| **Resource macro** | All resource structs use `#[crit_resource]` — not hand-rolled field lists |
