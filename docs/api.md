# API

## Routes

| Path | Auth | Description |
|------|------|-------------|
| `/health` | none | Health check |
| `/register` | none | User registration |
| `/login` | none | User login (returns JWT) |
| `/v1/static/{*path}` | none | Serve processed images from object store |
| `/v1/*` | JWT | Protected API routes |
| `/v1/principals/resolve` | JWT | Batch-resolve principal IDs to identity cards |
| `/v1/ws` | JWT | WebSocket endpoint |
| `/swagger-ui` | none | OpenAPI documentation |

All routes are nested under `/api` when accessed through the gateway (nginx or ingress).

## Scoped Gitops API (`/v1/projects/{project}/{kind}`)

Project-namespaced CRUD for resources belonging to a project (e.g. tasks, pipelines). The project must exist and the caller must have appropriate project or resource-level ACL.

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/v1/projects/{project}/{kind}` | List accessible objects in the project |
| `GET` | `/v1/projects/{project}/{kind}/{id}` | Fetch a single scoped object |
| `POST` | `/v1/projects/{project}/{kind}` | Create a new scoped object |
| `PUT` | `/v1/projects/{project}/{kind}/{id}` | Update a scoped object (fails if not exists) |
| `DELETE` | `/v1/projects/{project}/{kind}/{id}` | Delete a scoped object |

`{kind}` must be registered as a project-scoped kind (i.e. its `KindController` returns `is_scoped() = true`). Passing a global kind (e.g. `users`) returns `400 Bad Request`.

**Permission model**: Hybrid ACL — resource's own ACL if non-empty; otherwise the project's ACL filtered by `scope` matching the kind. See [access-control.md](access-control.md) for details.

**Pagination**: same `limit` / `cursor` query parameters as the global list endpoint.

**List response**: same brief/full document structure as the global API.

---

## Gitops API (`/v1/global/{kind}`)

A generic CRUD API for all resource kinds. `{kind}` maps to an ArangoDB collection name (e.g. `users`, `groups`, `projects`). Unknown kinds are auto-created on first access.

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/v1/global/{kind}` | List all accessible objects |
| `GET` | `/v1/global/{kind}/{id}` | Fetch a single object |
| `POST` | `/v1/global/{kind}` | Create a new object (id in body) |
| `POST` | `/v1/global/{kind}/{id}` | Upsert (create or replace) |
| `PUT` | `/v1/global/{kind}/{id}` | Update (fails if not exists) |
| `DELETE` | `/v1/global/{kind}/{id}` | Delete an object |

### Pagination

The list endpoint (`GET /v1/global/{kind}`) supports optional cursor-based pagination:

```
GET /v1/global/users?limit=10
GET /v1/global/users?limit=10&cursor=u_alice
```

**Query parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `limit` | integer | Number of items to return. If omitted, all items are returned (no pagination). |
| `cursor` | string | Opaque cursor from the previous page's `next_cursor` field. Omit for the first page. |

**Response without `limit`** (unchanged, backward-compatible):
```json
{ "items": [ ... ] }
```

**Response with `limit`:**
```json
{
  "items": [ ... ],
  "has_more": true,
  "next_cursor": "u_bob"
}
```

On the last page, `has_more` is `false` and `next_cursor` is omitted:
```json
{
  "items": [ ... ],
  "has_more": false
}
```

**Implementation notes:**
- Pagination is cursor-based using `_key` (ArangoDB primary key), which is already indexed and sorted.
- The DB query uses `SORT doc._key ASC` + `FILTER doc._key > @cursor`, making it efficient for millions of records.
- Pages may contain **fewer items than `limit`** when per-document ACL filtering removes some results. Keep paginating until `has_more: false`.

### List Response Shape (Brief)

List responses return a summary view of each resource (brief fields only), not the full document. Full documents are returned by the single-object GET endpoint.

| Kind | Brief fields |
|------|-------------|
| `users` | `id`, `meta`, `personal` |
| `groups` | `id`, `meta`, `name` |
| `service_accounts` | `id`, `meta`, `name` |
| `pipeline_accounts` | `id`, `meta`, `name` |

## Principal Resolution (`/v1/principals/resolve`)

Batch-resolve principal IDs (users, groups, service accounts, pipeline accounts) to lightweight identity cards. Designed for high-frequency use — results are cached per-principal with 10-minute TTL and a 2048-entry LRU bound.

```
POST /v1/principals/resolve
Content-Type: application/json

{ "ids": ["u_alice", "g_engineering", "sa_github", "nonexistent"] }
```

**Query parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `no-cache` | bool | `true` to bypass cache, force DB fetch and refresh cached entries. Default `false` |

**Response** `200 OK` (always — partial failures are inline):
```json
{
  "u_alice":        { "type": "user",            "name": "Alice",           "avatar_ulid": "01jz..." },
  "g_engineering":  { "type": "group",           "name": "Engineering" },
  "sa_github":      { "type": "service_account", "name": "GitHub Actions" },
  "nonexistent":    { "error": "not_found" }
}
```

**Contract:**
- Every requested ID appears as a key in the output map — no exceptions
- Found principals return `{ type, name, avatar_ulid? }` where `name` is the display name (`personal.name` for users, `name` for all others)
- Not-found or soft-deleted principals return `{ "error": "not_found" }`
- Maximum 500 IDs per request (returns `400` if exceeded)
- Negative results (not_found) are also cached

**Cache behavior:**
- Per-principal TTL cache: 10-minute expiry, max 2048 entries (LRU eviction)
- Cache hits are served without DB query; only misses trigger a batch AQL
- `?no-cache=true` forces DB fetch for all requested IDs and refreshes their cache entries
- Use `no-cache` sparingly — it's for cases where the frontend knows data just changed (e.g. after avatar upload or name edit)

---

## Media Upload (`/v1/global/{kind}/{id}/upload/{upload_type}`)

Upload an avatar or wallpaper image for a principal entity. The response is returned immediately after the raw file is stored; image processing (crop → resize → WebP encode) continues in a background task.

Supported kinds: `users`, `groups`.

```
POST /v1/global/users/{user_id}/upload/avatar
POST /v1/global/users/{user_id}/upload/wallpaper
POST /v1/global/groups/{group_id}/upload/avatar
POST /v1/global/groups/{group_id}/upload/wallpaper
Content-Type: multipart/form-data

file=<image bytes>   (JPEG / PNG / WebP, max 5 MB)
```

**Response** `201 Created`:
```json
{ "ulid": "01jz0a9rp700000000000000000" }
```

The returned ULID is immediately written to the user's `avatar_ulid` or `wallpaper_ulid` field. Once the background task completes, the processed WebP files are available at the static endpoint.

**Authorization:**

| Kind | Allowed callers |
|------|----------------|
| `users` | Self-upload; `ADM_USER_MANAGER`; `ADM_GODMODE` |
| `groups` | Caller with `MODIFY` ACL on the group; `ADM_USER_MANAGER`; `ADM_GODMODE` |

Unauthorized callers receive `404` (to avoid leaking whether the target entity exists).

**Background processing:**
1. Fetch raw bytes from `raw_uploads/`
2. Center-crop to target aspect ratio (1:1 avatar, 21:9 wallpaper)
3. Resize and encode two WebP variants (HD + thumbnail)
4. Store in `user_avatars/` or `user_wallpapers/`
5. Write a `persistent_files` record; delete the raw upload

Only one image conversion runs at a time (global `Semaphore(1)` in `AppState`). Additional uploads queue up and are processed in order.

Currently `kind = "users"` and `kind = "groups"` are supported.

---

## Static File Serving (`/v1/static/{*path}`)

Serves processed images from the object store without authentication. URLs are unguessable in practice because they are ULID-based.

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

**Response:** raw WebP bytes with:
- `Content-Type: image/webp`
- `Cache-Control: public, max-age=31536000, immutable`

**Restrictions:**
- Only `user_avatars/`, `user_wallpapers/`, `group_avatars/`, and `group_wallpapers/` directory prefixes are served — all other paths return `404`
- Path traversal (`..`) is rejected
- If the object store is not configured, returns `404`

Because each upload produces a new ULID, cached URLs never become stale — when a user re-uploads, the client fetches a new ULID from the user document and uses a new URL.

---

## Authentication

Three auth strategies:

| Strategy | Description |
|----------|-------------|
| **JWT** | Primary method. Issued on `/login`, required for `/v1/*` routes |
| **API key** | For service-to-service calls (`CLIENT_API_KEYS` env var) |

JWT middleware is applied to all `/v1` routes via the `Auth` struct initialized with `JWT_SECRET`.

### Login

```
POST /login
Content-Type: application/json

{ "user": "alice", "password": "secret" }
```

Response:
```json
{ "token": "<jwt>" }
```

### Registration

```
POST /register
Content-Type: application/json

{ "id": "u_alice", "password": "secret", ... }
```

## Configuration

Environment variables loaded via `dotenvy` from `backend/.env`:

| Variable | Default | Description |
|----------|---------|-------------|
| `DB_CONNECTION_STRING` | *(required)* | ArangoDB URL (e.g. `http://localhost:8529`) |
| `DB_NAME` | `unnamed` | Database name |
| `DB_USER` | `root` | ArangoDB user |
| `DB_PASSWORD` | *(empty)* | ArangoDB password |
| `PORT` | `3742` | Server port |
| `HOST` | `0.0.0.0` | Bind address |
| `JWT_SECRET` | *(required)* | JWT signing secret |
| `JWT_LIFETIME_SECS` | *(see config)* | JWT token lifetime in seconds |
| `CLIENT_API_KEYS` | *(optional)* | Comma-separated API keys |
