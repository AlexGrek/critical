# API Reference

> **For agents**: This file is the authoritative API reference. **Read this before opening any backend Rust files.**
> Source files are only needed for implementation details not covered here.

## Table of Contents

- [Authentication Routes](#authentication-routes)
- [Gitops API — Global](#gitops-api--global-v1globalkind)
- [Gitops API — Project-Scoped](#gitops-api--project-scoped-v1projectsprojectkind)
- [Principal Resolution](#principal-resolution)
- [Permissions Management](#permissions-management)
- [Media Upload](#media-upload)
- [Repository Check](#repository-check)
- [Static File Serving](#static-file-serving)
- [Access Check](#access-check)
- [Debug Endpoints](#debug-endpoints)
- [WebSocket](#websocket)
- [Resource Models](#resource-models)
- [Request / Response Shapes](#request--response-shapes)
- [Permission Model](#permission-model)
- [Authentication](#authentication)
- [Collections Reference](#collections-reference)
- [Special Behaviors](#special-behaviors)
- [Configuration](#configuration)

---

## Authentication Routes

No JWT required.

| Method | Path                 | Description                                 | Success                            |
| ------ | -------------------- | ------------------------------------------- | ---------------------------------- |
| `POST` | `/v1/register`       | Register a new user                         | `201` (empty body)                 |
| `POST` | `/v1/login`          | Login (returns JWT + sets cookie)           | `200 { "token": "..." }`           |
| `POST` | `/v1/logout`         | Logout                                      | `204`                              |
| `GET`  | `/v1/static/{*path}` | Serve processed images (avatars/wallpapers) | Raw WebP bytes                     |
| `GET`  | `/health`            | Health check                                | `200 { "status": "healthy", ... }` |

All routes are nested under `/api` when accessed through the gateway (nginx/ingress):
`http://localhost:3742/api/v1/...`

### Register

```
POST /v1/register
{ "user": "alice", "password": "secret" }
→ 201 (empty body)
```

### Login

```
POST /v1/login
{ "user": "alice", "password": "secret" }
→ 200 { "token": "<jwt>" }
```

Also sets `token` cookie (`HttpOnly; Secure; SameSite=Lax`).

---

## Gitops API — Global (`/v1/global/{kind}`)

JWT required. `{kind}` maps to an ArangoDB collection (e.g. `users`, `groups`, `projects`). Unknown kinds are auto-created on first access.

| Method   | Path                       | Description                                | Success        |
| -------- | -------------------------- | ------------------------------------------ | -------------- |
| `GET`    | `/v1/global/{kind}`        | List all accessible objects (brief)        | `200`          |
| `GET`    | `/v1/global/{kind}/{id}`   | Fetch single object (full)                 | `200`          |
| `GET`    | `/v1/global/{kind}/search` | Prefix search on `_key` (up to 15 results) | `200`          |
| `POST`   | `/v1/global/{kind}`        | Create new object (id in body)             | `201` full doc |
| `POST`   | `/v1/global/{kind}/{id}`   | Upsert (create or replace)                 | `201`/`200`    |
| `PUT`    | `/v1/global/{kind}/{id}`   | Update (fails if not exists)               | `200` full doc |
| `DELETE` | `/v1/global/{kind}/{id}`   | Soft-delete                                | `204`          |

**Query params:**
- `?limit=N` — paginate; omit for all items (no pagination envelope)
- `?cursor={opaque}` — next page cursor from previous response
- `?with_history=true` — attach `_history` field with latest `HistoryEntry` (GET single only)
- `?startwith={prefix}` — prefix search filter (search endpoint only)

---

## Gitops API — Project-Scoped (`/v1/projects/{project}/{kind}`)

JWT required. `{kind}` must be registered as project-scoped. Passing a global kind returns `400`.

| Method   | Path                                 | Description                | Success        |
| -------- | ------------------------------------ | -------------------------- | -------------- |
| `GET`    | `/v1/projects/{project}/{kind}`      | List scoped objects        | `200`          |
| `GET`    | `/v1/projects/{project}/{kind}/{id}` | Fetch single scoped object | `200`          |
| `POST`   | `/v1/projects/{project}/{kind}`      | Create scoped object       | `201` full doc |
| `POST`   | `/v1/projects/{project}/{kind}/{id}` | Upsert scoped object       | `201`/`200`    |
| `PUT`    | `/v1/projects/{project}/{kind}/{id}` | Update scoped object       | `200` full doc |
| `DELETE` | `/v1/projects/{project}/{kind}/{id}` | Delete scoped object       | `204`          |

Same `?limit` / `?cursor` query params as global list.

**Permission model**: Hybrid ACL — resource's own ACL if non-empty; otherwise the project's ACL filtered by `scope` matching the kind. See [access-control.md](access-control.md).

---

## Principal Resolution

```
POST /v1/principals/resolve
{ "ids": ["u_alice", "g_engineering", "sa_github", "nonexistent"] }
```

**Query params:**
- `?no-cache=true` — bypass cache, force DB fetch and refresh entries

**Response** `200 OK` (always — partial failures are inline):
```json
{
  "u_alice":       { "type": "user",            "name": "Alice", "avatar_ulid": "01jz..." },
  "g_engineering": { "type": "group",           "name": "Engineering" },
  "sa_github":     { "type": "service_account", "name": "GitHub Actions" },
  "nonexistent":   { "error": "not_found" }
}
```

**Contract:**
- Every requested ID appears as a key — no exceptions
- Found: `{ type, name, avatar_ulid? }` (name = `personal.name` for users, `name` for all others)
- Not found / soft-deleted: `{ "error": "not_found" }`
- Max 500 IDs per request (`400` if exceeded)
- Per-principal cache: 10-minute TTL, 2048-entry LRU; `?no-cache=true` bypasses

---

## Permissions Management

JWT + `ADM_USER_MANAGER` or `ADM_GODMODE` required.

```
POST /v1/global/permissions/{key}/grant
POST /v1/global/permissions/{key}/revoke
{ "principals": ["u_alice", "g_admins"] }
```

**Response** `200 OK`:
```json
{ "permission": "adm_user_manager", "granted_to": ["u_alice", "g_admins"] }
```

---

## Media Upload

JWT required. Upload avatar or wallpaper for `users` or `groups`.

```
POST /v1/global/users/{user_id}/upload/avatar
POST /v1/global/users/{user_id}/upload/wallpaper
POST /v1/global/groups/{group_id}/upload/avatar
POST /v1/global/groups/{group_id}/upload/wallpaper
Content-Type: multipart/form-data

file=<image bytes>  (JPEG / PNG / WebP, max 5 MB)
```

**Response** `201 Created`:
```json
{ "ulid": "01jz0a9rp700000000000000000" }
```

ULID is immediately written to the resource's `avatar_ulid` / `wallpaper_ulid` field. Image processing (crop → resize → WebP) runs async in a background task.

**Authorization:**

| Kind     | Allowed callers                                                          |
| -------- | ------------------------------------------------------------------------ |
| `users`  | Self; `ADM_USER_MANAGER`; `ADM_GODMODE`                                  |
| `groups` | Caller with `MODIFY` ACL on the group; `ADM_USER_MANAGER`; `ADM_GODMODE` |

Unauthorized → `404` (to avoid leaking entity existence).

**Background processing steps:**
1. Center-crop (1:1 avatar, 21:9 wallpaper)
2. Resize + encode two WebP variants (HD + thumbnail)
3. Store in `user_avatars/` or `user_wallpapers/`
4. Write `persistent_files` record; delete raw upload

Only one conversion runs at a time (global `Semaphore(1)`).

---

## Repository Check

JWT required. Probes a project repository link for connectivity/auth, without saving anything — used by the frontend to validate a `RepoLink` before or after adding it.

```
POST /v1/global/projects/{id}/repocheck
Content-Type: application/json

{ "url": "...", "provider": "github", "default_branch": "main",
  "auth_method": "none", "credential": "rc_..." }
```

Body is a full `RepoLink` (not an index into the project's saved list), so the same endpoint validates both an unsaved form entry and an already-saved repo row.

Fetches exactly one hardcoded file, `pipelines.js`, from the resolved default (or explicit) branch:
- GitHub provider → GitHub Contents API (via `octocrab`), authenticated with the referenced credential's token when `auth_method: "github_token"`
- Everything else → a shallow (depth-1) bare git clone (via `gix`), over plain HTTPS (anonymous) or SSH (using the referenced credential's private key)

**Response** `200 OK` for any outcome the probe actually reaches — the file being missing or the connection failing are not HTTP errors:
```json
{ "status": "found", "branch": "main", "size": 1234, "message": "pipelines.js found on branch main" }
```
`status` is one of `found` / `missing` / `error`. `branch` and `size` are omitted when not applicable.

`4xx` is reserved for request-shape problems: unknown project (`404`), caller lacks `MODIFY` on the project (`404`), `credential` references a `repo_credentials` document the caller lacks `READ` on (`404`), or an unparseable/unsupported repository URL (`400`).

Bounded by a 30s timeout per probe.

---

## Static File Serving

No auth required.

```
GET /v1/static/user_avatars/{ulid}_hd.webp
GET /v1/static/user_avatars/{ulid}_thumb.webp
GET /v1/static/user_wallpapers/{ulid}_hd.webp
GET /v1/static/user_wallpapers/{ulid}_thumb.webp
GET /v1/static/group_avatars/{ulid}_hd.webp
GET /v1/static/group_avatars/{ulid}_thumb.webp
GET /v1/static/group_wallpapers/{ulid}_hd.webp
GET /v1/static/group_wallpapers/{ulid}_thumb.webp
```

**Response:** Raw WebP bytes with `Content-Type: image/webp`, `Cache-Control: public, max-age=31536000, immutable`.

Allowed directory prefixes: `user_avatars/`, `user_wallpapers/`, `group_avatars/`, `group_wallpapers/` only. Path traversal (`..`) rejected. Returns `404` if object store not configured.

---

## Access Check

JWT required.

| Method | Path                                             | Description                          |
| ------ | ------------------------------------------------ | ------------------------------------ |
| `GET`  | `/v1/accesscheck/me/permissions`                 | Get caller's super-permissions       |
| `GET`  | `/v1/accesscheck/me/acls`                        | Get caller's detailed ACL report (groups/projects + why) |
| `GET`  | `/v1/accesscheck/global/{kind}/{id}`             | Check permissions on global resource |
| `GET`  | `/v1/accesscheck/projects/{project}/{kind}/{id}` | Check permissions on scoped resource |

**My permissions response** — also the app's lightweight "whoami": the auth cookie is
HttpOnly, so the frontend cannot decode its own user ID client-side and reads `user_id`
from this response instead:
```json
{ "user_id": "u_alice", "super_permissions": ["adm_godmode", "usr_create_groups"] }
```

**My ACLs response** — everything the caller can see about their own access. Reports
document-level ACL grants only; does not fold in super-permission bypasses (a godmode
user may still see empty `groups`/`projects` here despite `is_godmode: true`):
```json
{
  "user_id": "u_alice",
  "is_godmode": false,
  "super_permissions": ["usr_create_groups"],
  "principals": ["u_alice", "g_engineering", "g_leads"],
  "direct_memberships": ["g_engineering"],
  "groups": [
    {
      "id": "g_engineering",
      "name": "Engineering",
      "permission": { "bits": 127, "can_fetch": true, "can_list": true, "can_notify": true, "can_create": true, "can_modify": true },
      "via_principals": ["u_alice"],
      "scoped_grants": []
    }
  ],
  "projects": [
    {
      "id": "proj1",
      "name": "Proj One",
      "permission": { "bits": 7, "can_fetch": true, "can_list": true, "can_notify": true, "can_create": false, "can_modify": false },
      "via_principals": ["g_leads"],
      "scoped_grants": [
        { "scope": "tasks", "permission": { "bits": 31, "can_fetch": true, "can_list": true, "can_notify": true, "can_create": true, "can_modify": true }, "via_principals": ["g_leads"] }
      ]
    }
  ]
}
```
`principals` is the full transitive chain (self + nested group memberships, up to 10 levels).
`direct_memberships` is one hop only. `groups`/`projects` only list resources with at least
one matching ACL entry — `permission` covers unscoped entries (or `scope: "*"`); `scoped_grants`
lists any entries restricted to a specific resource kind (project ACLs only).

**Resource access check response:**
```json
{
  "kind": "groups",
  "id": "g_engineering",
  "effective_permissions": {
    "bits": 15,
    "flags": ["fetch", "list", "notify", "create"]
  }
}
```

---

## Debug Endpoints

JWT + `ADM_GODMODE` required.

| Method | Path                           | Description                                                                       |
| ------ | ------------------------------ | --------------------------------------------------------------------------------- |
| `GET`  | `/v1/debug/collections`        | List all collections                                                              |
| `GET`  | `/v1/debug/collections/{name}` | Dump all documents in collection (`400` for system collections starting with `_`) |
| `GET`  | `/v1/debug/access`             | Inspect ACL resolution for user                                                   |
| `GET`  | `/v1/debug/events`             | List system events (supports `?kind=`, `?priority=`, `?limit=`, `?cursor=`)       |
| `GET`  | `/v1/debug/history`            | List change history                                                               |

**Collections list response:**
```json
{ "collections": [{ "name": "users" }, { "name": "groups" }, ...] }
```

**Collection dump response:**
```json
{ "collection": "users", "count": 3, "documents": [ /* raw ArangoDB docs */ ] }
```

**Events response:**
```json
{ "events": [...], "has_more": true, "next_cursor": "...", "count": 50 }
```

---

## WebSocket

JWT required.

```
WS /v1/ws
```

Real-time event subscription for the authenticated user.

---

## Resource Models

All models use the **`#[crit_derive::crit_resource]`** proc macro which injects these standard fields:

| Field         | Type                      | Description                                                            |
| ------------- | ------------------------- | ---------------------------------------------------------------------- |
| `id`          | `String`                  | Resource key (maps to ArangoDB `_key`)                                 |
| `labels`      | `HashMap<String, String>` | Queryable key-value metadata (desired state)                           |
| `annotations` | `HashMap<String, String>` | Non-queryable freeform strings (desired state)                         |
| `state`       | `ResourceState`           | Server-managed: `created_at`, `created_by`, `updated_at`, `updated_by` |
| `acl`         | `AccessControlStore`      | Per-document ACL (unless `no_acl` specified)                           |
| `deletion`    | `Option<DeletionInfo>`    | Soft-delete marker (null = alive)                                      |
| `hash_code`   | `String`                  | FNV-1a 64-bit hash of desired state (16-char hex)                      |

### `users` (collection: `users`, prefix: `u_`, **no ACL**)

```
id: String
personal:
  name: String
  gender: String
  job_title: String
  manager: Option<String>      # manager user ID
avatar_ulid: Option<String>
wallpaper_ulid: Option<String>
password_hash: String          # bcrypt — NEVER returned in API
```

**Brief fields (list queries):** `id`, `labels`, `annotations`, `personal`, `avatar_ulid`

Access controlled by super-permissions only (no per-resource ACL).

### `groups` (collection: `groups`, prefix: `g_`)

```
id: String
name: String
description: Option<String>
avatar_ulid: Option<String>
wallpaper_ulid: Option<String>
```

**Brief fields:** `id`, `labels`, `annotations`, `name`, `avatar_ulid`

### `service_accounts` (collection: `service_accounts`, prefix: `sa_`)

```
id: String
name: String
description: Option<String>
avatar_ulid: Option<String>
wallpaper_ulid: Option<String>
token_hash: String             # bcrypt — NEVER returned in API
```

**Brief fields:** `id`, `labels`, `annotations`, `name`, `avatar_ulid`

### `pipeline_accounts` (collection: `pipeline_accounts`, prefix: `pa_`)

```
id: String
name: String
description: Option<String>
scope: Option<String>          # scoped to pipeline/project
avatar_ulid: Option<String>
wallpaper_ulid: Option<String>
token_hash: String             # bcrypt — NEVER returned in API
```

**Brief fields:** `id`, `labels`, `annotations`, `name`, `avatar_ulid`

### `projects` (collection: `projects`, no prefix)

```
id: String
name: String
description: Option<String>
repositories: Vec<RepoLink>
  url: String
  provider: RepoProvider       # git | github | gitlab | bitbucket | svn | mercurial | custom
  name: Option<String>
  default_branch: Option<String>
  auth_method: RepoAuthMethod  # none (default) | ssh | github_token
  credential: Option<String>   # id of a repo_credentials document
enabled_services: Vec<ProjectService>
  # integrations | pipelines | deployments | secrets | wikis | apps
  # tasks | talks | releases | environments | insights
```

**Brief fields:** `id`, `labels`, `annotations`, `name`

### `repo_credentials` (collection: `repo_credentials`, prefix: `rc_`)

Reusable SSH keys / access tokens referenced by a project's `RepoLink.credential`. `secret` and `passphrase` are write-only — accepted on create/update, never returned; `to_external` adds a computed `has_secret: bool` instead.

```
id: String
name: String
method: RepoAuthMethod         # ssh | github_token
description: Option<String>
username: Option<String>       # SSH user, defaults to "git"
secret: Option<String>         # write-only — SSH private key PEM or GitHub token
passphrase: Option<String>     # write-only — passphrase for an encrypted SSH key
```

**Brief fields:** `id`, `labels`, `name`, `method`

### `ticketgroups` (collection: `ticketgroups`, prefix: `tg_`, **project-scoped**)

```
id: String
name: String
description: Option<String>
project: String                # injected by scoped handler
ticket_types: Vec<TicketTypeDef>
  name: String
  description: Option<String>
  statuses: Vec<TicketStatus>
    name: String
    category: StatusCategory   # todo | in_progress | done
  fields: Vec<TicketFieldDef>
    name: String
    field_type: TicketFieldType  # text | number | boolean | date | user | single_select | multi_select
    required: bool
    description: Option<String>
    options: Vec<String>         # for select types
```

**Brief fields:** `id`, `labels`, `annotations`, `name` (ticket_types excluded from list queries)

### `crds` (collection: `crds`, no ACL, all authenticated users can read)

```
id: String
scope: CrdScope                # global | project
acl_mode: CrdAclMode           # special | inherit | custom
nouns:
  singular: String             # e.g. "deployment"
  plural: String               # e.g. "deployments"
relations: Vec<CrdRelation>
  edge_collection: String
  direction: RelationDirection  # outbound | inbound
  target_kind: String
  label: Option<String>
fields: HashMap<String, FieldDef>
id_prefix: String              # e.g. "dep_"
super_permission: Option<String>
description: Option<String>
```

### `memberships` (edge collection — no `#[crit_resource]` macro)

```
id: String                     # "{principal_id}::{group_id}"
_from: String                  # e.g. "users/u_alice", "groups/g_eng"
_to: String                    # e.g. "groups/g_admins"
principal: String              # denormalized principal ID
group: String                  # denormalized group ID
```

---

## Request / Response Shapes

### List (no `?limit`)

```json
{ "items": [ /* brief objects */ ] }
```

### Paginated list (`?limit=N`)

```json
{
  "items": [ /* brief objects */ ],
  "has_more": true,
  "next_cursor": "opaque-cursor-string"
}
```

Last page: `has_more: false`, **no** `next_cursor` key.

Pages may contain fewer items than `limit` — ACL filtering removes some results. Continue until `has_more: false`.

### Brief field reference

| Kind                | Brief fields                                             |
| ------------------- | -------------------------------------------------------- |
| `users`             | `id`, `labels`, `annotations`, `personal`, `avatar_ulid` |
| `groups`            | `id`, `labels`, `annotations`, `name`, `avatar_ulid`     |
| `service_accounts`  | `id`, `labels`, `annotations`, `name`, `avatar_ulid`     |
| `pipeline_accounts` | `id`, `labels`, `annotations`, `name`, `avatar_ulid`     |
| `projects`          | `id`, `labels`, `annotations`, `name`                    |
| `repo_credentials`  | `id`, `labels`, `name`, `method`                         |
| `ticketgroups`      | `id`, `labels`, `annotations`, `name`                    |

### Single resource (GET `/{id}`, POST create, PUT update)

POST create and PUT update both return the **full document** — read `hash_code` directly from the write response; no follow-up GET needed.

```json
{
  "id": "u_alice",
  "labels": {},
  "annotations": {},
  "state": {
    "created_at": "2026-03-15T12:00:00Z",
    "created_by": "u_root",
    "updated_at": "2026-03-15T12:00:00Z",
    "updated_by": null
  },
  "acl": { "list": [...], "last_mod_date": "..." },
  "hash_code": "a1b2c3d4e5f6g7h8",
  "personal": { "name": "Alice", "gender": "", "job_title": "", "manager": null },
  "avatar_ulid": null
}
```

### Create request body

```json
{
  "id": "resource_id",
  "field1": "value",
  "labels": { "env": "prod" },
  "annotations": { "note": "..." },
  "acl": {
    "list": [
      { "permissions": 127, "principals": ["u_alice"] },
      { "permissions": 7,   "principals": ["g_viewers"] }
    ],
    "last_mod_date": "2026-03-15T12:00:00Z"
  }
}
```

**Status codes:**
- `201 Created` — POST create, POST upsert to non-existent
- `200 OK` — PUT update, POST upsert to existing

### Error response

```json
{
  "error": {
    "message": "Authorization failed: Unauthorized",
    "status": 401,
    "type": "authorization_error"
  }
}
```

**ACL denial returns `404` (not `403`)** to avoid leaking resource existence.

---

## Permission Model

### Permission bits (`u8` bitmask)

```
FETCH   = 1   (0x01)  # Read single object
LIST    = 2   (0x02)  # List objects
NOTIFY  = 4   (0x04)  # Receive notifications
CREATE  = 8   (0x08)  # Create new objects
MODIFY  = 16  (0x10)  # Update / delete

READ    = FETCH | LIST | NOTIFY    = 7
WRITE   = CREATE | MODIFY | READ   = 31
ROOT    = WRITE | CUSTOM1 | CUSTOM2 = 127
```

### Per-resource ACL entry

```json
{ "permissions": 31, "principals": ["u_alice", "g_engineering"], "scope": null }
```

`scope` — optional; restricts the entry to one resource kind (e.g. `"ticketgroups"`). Absent or `"*"` matches all kinds. Only used in project ACL entries.

### Super-permissions (bypass per-resource ACL)

| Key                   | Grants                                                  |
| --------------------- | ------------------------------------------------------- |
| `adm_godmode`         | Full bypass of all ACLs; full access to debug endpoints |
| `adm_user_manager`    | Full CRUD on users/groups; auto-granted to `u_root`     |
| `adm_config_editor`   | Edit global config and projects                         |
| `usr_create_groups`   | Create new groups (granted on registration)             |
| `usr_create_projects` | Create new projects                                     |

Stored in the `permissions` collection (key = permission name, value = `{ "principals": [...] }`).

### Godmode

Users with `ADM_GODMODE` bypass all ACL checks and receive `ROOT` (127) permissions on every resource.

### Principal caching

- Group membership cached with 5-second TTL (no invalidation)
- Changes propagate with up to 5-second latency

---

## Authentication

Three strategies:

| Strategy    | Description                                                                              |
| ----------- | ---------------------------------------------------------------------------------------- |
| **JWT**     | Primary. Issued on `/login`, passed as `Authorization: Bearer <token>` or `token` cookie |
| **API key** | Service-to-service. Set via `CLIENT_API_KEYS` env var (comma-separated)                  |

JWT middleware applied to all `/v1/*` routes.

---

## Collections Reference

### Vertex collections

| Collection           | Prefix   | Notes                     |
| -------------------- | -------- | ------------------------- |
| `users`              | `u_`     | No ACL                    |
| `groups`             | `g_`     |                           |
| `service_accounts`   | `sa_`    |                           |
| `pipeline_accounts`  | `pa_`    |                           |
| `projects`           | *(none)* |                           |
| `repo_credentials`   | `rc_`    |                           |
| `crds`               | *(none)* | No ACL                    |
| `ticketgroups`       | `tg_`    | Project-scoped            |
| `permissions`        | *(none)* | Super-permissions store   |
| `resource_history`   |          | Immutable snapshots       |
| `resource_events`    |          | Per-resource events       |
| `system_events`      |          | Server lifecycle events   |
| `unprocessed_images` |          | Temporary, pre-processing |
| `persistent_files`   |          | Processed image metadata  |

### Edge collections

| Collection    | Direction                           | Notes            |
| ------------- | ----------------------------------- | ---------------- |
| `memberships` | `_from` (principal) → `_to` (group) | Group membership |

### Dynamic collections

Ad-hoc collections created on first access to `/v1/global/{kind}` or `/v1/projects/{project}/{kind}`. Kind must match `[a-z0-9_]+`.

---

## Special Behaviors

### Soft delete

When a resource is deleted:
1. `deletion` field set to `{ deleted_at, deleted_by, disconnected_edges }`
2. All connected edges captured in `disconnected_edges`
3. All list/get queries filter `doc.deletion == null`
4. Document persists in history and events

### Hash code (conflict detection)

- FNV-1a 64-bit hash of desired-state fields (excludes `hash_code`, `deletion`, `_id`, `_rev`)
- Returned as 16-character hex string in all responses
- Used for optimistic locking on upsert/update

### Pagination

- Cursor-based using `_key` (already indexed and sorted in ArangoDB)
- Query: `SORT doc._key ASC` + `FILTER doc._key > @cursor`
- Efficient for millions of records

### The `doc_snapshot` pattern

In handlers, the fully-prepared internal doc is cloned **before** passing to `generic_create`/`generic_update`, then used for both history recording and the response. **Never** issue a follow-up `generic_get` to build the response — it fails silently under load.

---

## Configuration

Environment variables (loaded from `backend/.env`):

| Variable               | Default        | Description                                 |
| ---------------------- | -------------- | ------------------------------------------- |
| `DB_CONNECTION_STRING` | *(required)*   | ArangoDB URL (e.g. `http://localhost:8529`) |
| `DB_NAME`              | `unnamed`      | Database name                               |
| `DB_USER`              | `root`         | ArangoDB user                               |
| `DB_PASSWORD`          | *(empty)*      | ArangoDB password                           |
| `PORT`                 | `3742`         | Server port                                 |
| `HOST`                 | `0.0.0.0`      | Bind address                                |
| `JWT_SECRET`           | *(required)*   | JWT signing secret                          |
| `JWT_LIFETIME_SECS`    | *(see config)* | JWT token lifetime in seconds               |
| `CLIENT_API_KEYS`      | *(optional)*   | Comma-separated API keys                    |
| `OBJECT_STORE_BACKEND` | *(optional)*   | `local` / `s3` / `webdav`                   |
| `OBJECT_STORE_PATH`    | `./data`       | Path for local object store                 |
| `OBJECT_STORE_BUCKET`  |                | S3 bucket name                              |
| `OBJECT_STORE_KEY`     |                | S3 access key                               |
| `OBJECT_STORE_SECRET`  |                | S3 secret key                               |
| `OBJECT_STORE_REGION`  |                | S3 region                                   |
| `OBJECT_STORE_URL`     |                | S3 custom endpoint or WebDAV URL            |
| `CRITICAL_EVENTS_TTL`  | `30`           | Event retention in days                     |

**Dev defaults** (from test fixtures): `root` / `changeme`, database `devdb`, port `8529`.
