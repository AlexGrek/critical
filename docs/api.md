# API

## Routes

| Path | Auth | Description |
|------|------|-------------|
| `/health` | none | Health check |
| `/register` | none | User registration |
| `/login` | none | User login (returns JWT) |
| `/v1/*` | JWT | Protected API routes |
| `/v1/ws` | JWT | WebSocket endpoint |
| `/swagger-ui` | none | OpenAPI documentation |

All routes are nested under `/api` when accessed through the gateway (nginx or ingress).

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
