# Database Schema

> **Database**: ArangoDB (RocksDB storage engine)
> **Default DB Name**: `unnamed` (configurable via `DB_NAME` env var)
> **Connection**: `http://localhost:8529` (configurable via `DB_CONNECTION_STRING`)

Collections are auto-created on first connection via `ArangoDb::connect_basic()` in `backend/src/db/arangodb/mod.rs`. There is no migration system — schema is defined by Rust types in [models.rs](shared/src/models.rs), and ArangoDB itself is schemaless (documents are JSON with no DB-enforced structure).

---

## Schema Evolution

There are no migration files, no versioning, and no automated schema diffing. The Rust structs define the **application-level** schema, not a DB-enforced one.

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

---

## Entity-Relationship Diagram

```
┌────────────────┐         ┌────────────────┐         ┌────────────────┐
│     users      │         │  memberships   │         │     groups     │
│  (Document)    │         │  (Edge)        │         │  (Document)    │
├────────────────┤         ├────────────────┤         ├────────────────┤
│ _key (u_xxx)   │◄─_from──│ _from          │         │ _key (g_xxx)   │
│                │         │ _to            │──_to───▶│                │
│                │         │ principal      │         │                │
│                │         │ group          │         │                │
│                │         │ _key           │         │                │
│                │         │ (principal::   │         │                │
│                │         │     group)     │         │                │
└────────────────┘         └────────────────┘         └────────────────┘
                                  ▲
                                  │                   ┌────────────────┐
                           groups can also            │  permissions   │
                           be principals              │  (Document)    │
                           (nested groups)            ├────────────────┤
                                                      │ _key (perm     │
                                                      │   name)        │
                                                      │ principals[]   │
                                                      └────────────────┘
```

### Membership Graph

```
 ┌────────┐                  ┌────────┐
 │ User A │──membership──┐   │ User B │──membership──┐
 │ u_alice│              │   │ u_bob  │              │
 └────────┘              ▼   └────────┘              ▼
                    ┌──────────┐                ┌──────────┐
                    │ Group X  │──membership──▶│ Group Y  │
                    │ g_devs   │               │ g_all    │
                    └──────────┘               └──────────┘

 Users and groups can both be members of groups (nested groups supported).
 Principals prefixed u_ are users, g_ are groups.
```

---

## Collections

### `users` — Document Collection

| Key Field | Key Format                   | Rust Struct |
| --------- | ---------------------------- | ----------- |
| `_key`    | `u_` prefix (e.g. `u_alice`) | `User`      |

### `groups` — Document Collection

| Key Field | Key Format                    | Rust Struct |
| --------- | ----------------------------- | ----------- |
| `_key`    | `g_` prefix (e.g. `g_admins`) | `Group`     |

### `memberships` — Edge Collection

| Key Field | Key Format                       | Rust Struct       |
| --------- | -------------------------------- | ----------------- |
| `_key`    | `{principal}::{group}` composite | `GroupMembership` |

**Edge fields**: `_from` (`users/{id}` or `groups/{id}`) → `_to` (`groups/{id}`). Also stores `principal` and `group` as plain string fields for backward-compatible queries.

Native graph traversal is supported via `_from`/`_to` (e.g. `FOR v IN 1..10 OUTBOUND "users/u_alice" memberships`).

**Key AQL queries**:
- Users in group: `FOR m IN memberships FILTER m.group == @group FILTER LIKE(m.principal, "u_%") RETURN m.principal`
- Sub-groups in group: `FOR m IN memberships FILTER m.group == @group FILTER LIKE(m.principal, "g:%") RETURN m.principal`
- All groups for user (recursive): `FOR v IN 1..10 OUTBOUND CONCAT("users/", @user) memberships RETURN v._key`

### `permissions` — Document Collection

| Key Field | Key Format                              | Rust Struct        |
| --------- | --------------------------------------- | ------------------ |
| `_key`    | permission name (e.g. `adm_user_manager`) | `GlobalPermission` |

**Fields**: `principals` (`Vec<String>`) — list of user/group IDs granted this permission.

**Defined super-permissions**:

| Permission | Description |
|------------|-------------|
| `adm_user_manager` | Create, edit, delete, disable, impersonate users |
| `adm_project_manager` | Full access to all projects |
| `usr_create_projects` | User-level permission to create projects |
| `adm_config_editor` | Admin-level core configuration editor |

Permission checks resolve through the membership graph — a user has a permission if they or any of their groups (including nested groups) appear in that permission's `principals` array.

---

### `projects` — Document Collection

| Key Field | Key Format       | Rust Struct |
| --------- | ---------------- | ----------- |
| `_key`    | project ID       | `Project`   |

Managed via gitops API (`/api/v1/global/projects`). Auto-created on startup.

---

## Gitops / CRD Collections

The gitops API (`/api/v1/global/{kind}`) supports **any** kind string as a collection name (CRD-style). Collections beyond the built-in ones above are auto-created on first access. All gitops operations use generic AQL queries against `serde_json::Value` documents.

**Special handling by kind:**
- `users`: `password` field in request is hashed to `password_hash`; `password_hash` stripped from responses.

---

## Planned Collections (Not Yet Persisted)

Models defined in [models.rs](shared/src/models.rs) but no DB operations implemented:

| Planned Collection | Rust Struct | Key Type     |
| ------------------ | ----------- | ------------ |
| tickets            | `Ticket`    | `i64`        |

`TicketGroup` is embedded within `Project`, not a separate collection.

---

## Indexes

ArangoDB auto-indexes `_key`, and auto-indexes `_from`/`_to` on edge collections. No additional explicit indexes defined. Candidates for manual indexes:
- `memberships.principal`
- `memberships.group`

---

## Transactions

All five active collections (`users`, `groups`, `memberships`, `permissions`, `projects`) participate in server-side transactions with `wait_for_sync: true`.

---

## Conventions

| Convention              | Detail                                    |
| ----------------------- | ----------------------------------------- |
| **ID prefixes**         | Users: `u_`, Groups: `g_`                 |
| **Serde rename**        | Rust `id` → ArangoDB `_key`               |
| **Soft deletes**        | `User.deactivated` flag                   |
| **Composite edge keys** | `{principal}::{group}`                    |
| **Auto-creation**       | Collections created on startup if missing |
| **No migrations**       | Schema defined entirely by Rust structs   |

---

## Authorization (Gitops API)

The gitops API enforces permission checks based on the collection kind. Unauthorized access returns **404** (hides resource existence) rather than 401/403.

### Rules by Kind

| Collection | Read Access | Write Access |
|------------|-------------|--------------|
| `users`, `groups` | All authenticated users | `adm_user_manager` super-permission |
| `projects` | `adm_project_manager` OR project ACL with READ | Create: `adm_project_manager` or `usr_create_projects`. Update/delete: `adm_project_manager` OR project ACL with WRITE |
| Other kinds | All authenticated users | All authenticated users (TODO: ACL inheritance) |

### ACL Permission Bits (Projects)

Projects embed an `acl: AccessControlStore` with fine-grained permission entries:

| Permission | Bit Value | Description |
|------------|-----------|-------------|
| FETCH | 1 | Read individual project |
| LIST | 2 | See project in listings |
| NOTIFY | 4 | Receive notifications |
| CREATE | 8 | Create child resources |
| MODIFY | 16 | Update/delete project |
| READ (composite) | 7 | FETCH + LIST + NOTIFY |
| WRITE (composite) | 31 | CREATE + MODIFY + READ |

ACL checks resolve through the user's principals (user ID + all group IDs via membership graph, up to 10 levels).

---

*Last updated: 2026-02-15*
