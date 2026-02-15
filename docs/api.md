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
