# Database Schema

> **Database**: ArangoDB (RocksDB storage engine)
> **Default DB Name**: `unnamed` (configurable via `DB_NAME` env var)
> **Connection**: `http://localhost:8529` (configurable via `DB_CONNECTION_STRING`)

Collections are auto-created on first connection via `ArangoDb::connect_basic()`. No migration files — schema is derived from Rust types in `shared/src/models.rs`.

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

## Collections

### `users` — Document Collection

| Key Field | Key Format | Rust Struct |
|-----------|------------|-------------|
| `_key` | `u_` prefix (e.g. `u_alice`) | `User` |

### `groups` — Document Collection

| Key Field | Key Format | Rust Struct |
|-----------|------------|-------------|
| `_key` | `g_` prefix (e.g. `g_admins`) | `Group` |

### `memberships` — Edge Collection

| Key Field | Key Format | Rust Struct |
|-----------|------------|-------------|
| `_key` | `{principal}::{group}` composite | `GroupMembership` |

**Edges**: `principal` (u_ or g_) → `group` (g_)

**Key AQL queries**:
- Users in group: `FOR m IN memberships FILTER m.group == @group FILTER LIKE(m.principal, "u_%") RETURN m.principal`
- Sub-groups in group: `FOR m IN memberships FILTER m.group == @group FILTER LIKE(m.principal, "g:%") RETURN m.principal`

## Indexes

No explicit indexes defined. ArangoDB auto-indexes `_key`. Candidates for manual indexes:
- `memberships.principal`
- `memberships.group`

## Transactions

All three active collections participate in server-side transactions with `wait_for_sync: true`.

## Conventions

| Convention | Detail |
|------------|--------|
| **ID prefixes** | Users: `u_`, Groups: `g_` |
| **Serde rename** | Rust `id` → ArangoDB `_key` |
| **Soft deletes** | `User.deactivated` flag |
| **Composite edge keys** | `{principal}::{group}` |
| **Auto-creation** | Collections created on startup if missing |
| **No migrations** | Schema defined entirely by Rust structs |
