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
│ _key (u_xxx)   │◄────────│ principal      │         │ _key (g_xxx)   │
│                │         │ group          │────────▶│                │
│                │         │ _key           │         │                │
│                │         │ (principal::   │         │                │
│                │         │     group)     │         │                │
└────────────────┘         └────────────────┘         └────────────────┘
                                  ▲
                                  │
                           groups can also
                           be principals
                           (nested groups)
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

**Edges**: `principal` (u_ or g_) → `group` (g_)

**Key AQL queries**:
- Users in group: `FOR m IN memberships FILTER m.group == @group FILTER LIKE(m.principal, "u_%") RETURN m.principal`
- Sub-groups in group: `FOR m IN memberships FILTER m.group == @group FILTER LIKE(m.principal, "g:%") RETURN m.principal`

---

## Planned Collections (Not Yet Persisted)

Models defined in [models.rs](shared/src/models.rs) but no DB operations implemented:

| Planned Collection | Rust Struct | Key Type     |
| ------------------ | ----------- | ------------ |
| projects           | `Project`   | `uuid::Uuid` |
| tickets            | `Ticket`    | `i64`        |

`TicketGroup` is embedded within `Project`, not a separate collection.

---

## Indexes

No explicit indexes defined. ArangoDB auto-indexes `_key`. Candidates for manual indexes:
- `memberships.principal`
- `memberships.group`

---

## Transactions

All three active collections participate in server-side transactions with `wait_for_sync: true`.

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

*Last updated: 2026-02-15*
