# API

## Routes

| Path | Auth | Description |
|------|------|-------------|
| `/health` | none | Health check |
| `/register` | none | User registration |
| `/login` | none | User login (returns JWT) |
| `/v1/static/{*path}` | none | Serve processed images from object store |
| `/v1/*` | JWT | Protected API routes |
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

## Media Upload (`/v1/global/{kind}/{id}/upload/{upload_type}`)

Upload an avatar or wallpaper image for a user. The response is returned immediately after the raw file is stored; image processing (crop → resize → WebP encode) continues in a background task.

```
POST /v1/global/users/{user_id}/upload/avatar
POST /v1/global/users/{user_id}/upload/wallpaper
Content-Type: multipart/form-data

file=<image bytes>   (JPEG / PNG / WebP, max 5 MB)
```

**Response** `201 Created`:
```json
{ "ulid": "01jz0a9rp700000000000000000" }
```

The returned ULID is immediately written to the user's `avatar_ulid` or `wallpaper_ulid` field. Once the background task completes, the processed WebP files are available at the static endpoint.

**Authorization:**
- A user may upload their own media (self-upload)
- `ADM_USER_MANAGER` may upload for any user
- `ADM_GODMODE` bypasses all checks
- Any other caller receives `404` (to avoid leaking whether the target user exists)

**Background processing:**
1. Fetch raw bytes from `raw_uploads/`
2. Center-crop to target aspect ratio (1:1 avatar, 21:9 wallpaper)
3. Resize and encode two WebP variants (HD + thumbnail)
4. Store in `user_avatars/` or `user_wallpapers/`
5. Write a `persistent_files` record; delete the raw upload

Only one image conversion runs at a time (global `Semaphore(1)` in `AppState`). Additional uploads queue up and are processed in order.

Currently only `kind = "users"` is supported.

---

## Static File Serving (`/v1/static/{*path}`)

Serves processed images from the object store without authentication. URLs are unguessable in practice because they are ULID-based.

```
GET /v1/static/user_avatars/{ulid}_hd.webp
GET /v1/static/user_avatars/{ulid}_thumb.webp
GET /v1/static/user_wallpapers/{ulid}_hd.webp
GET /v1/static/user_wallpapers/{ulid}_thumb.webp
```

**Response:** raw WebP bytes with:
- `Content-Type: image/webp`
- `Cache-Control: public, max-age=31536000, immutable`

**Restrictions:**
- Only `user_avatars/` and `user_wallpapers/` directory prefixes are served — all other paths return `404`
- Path traversal (`..`) is rejected
- If the object store is not configured, returns `404`

Because each upload produces a new ULID, cached URLs never become stale — when a user re-uploads, the client fetches a new ULID from the user document and uses a new URL.

---

## Authentication

Three auth strategies:

| Strategy | Description |
|----------|-------------|
| **JWT** | Primary method. Issued on `/login`, required for `/v1/*` routes |
| **Management token** | For admin/ops tooling (`MGMT_TOKEN` env var) |
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
| `MGMT_TOKEN` | *(optional)* | Management API token |
| `CLIENT_API_KEYS` | *(optional)* | Comma-separated API keys |
