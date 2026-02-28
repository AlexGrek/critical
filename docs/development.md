# Development

## Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Docker](https://www.docker.com/) (for ArangoDB and cross-compilation)
- [Node.js](https://nodejs.org/) (for frontend)
- [Python 3](https://www.python.org/) + pytest (for API integration tests)

## Build Commands

### Workspace (Rust)

```bash
cargo build                 # Build all workspace crates
cargo build --bin cr1t      # Build CLI only
cargo build --bin axum-api  # Build backend only
make dev                    # Quick dev build (all crates)
```

### Running Locally

```bash
make run                    # Start ArangoDB + run backend (persistent DB)
make run-fresh              # Reset DB volumes, then run (clean slate)
```

Frontend (separate terminal):
```bash
cd frontend
npm install
npm run dev                 # Dev server on port 5173 (proxies API to localhost:3742)
```

### Frontend

```bash
cd frontend
npm run dev                 # Dev server with HMR
npm run build               # Production build
npm run typecheck           # react-router typegen && tsc
npm start                   # Serve production build
```

### Database

```bash
make run-db                 # Start ArangoDB container (port 8529)
make stop-db                # Stop container
make reset-db               # Stop and delete volumes
make logs-db                # Tail container logs
```

ArangoDB web UI: `http://localhost:8529`

## Object Store

The backend supports pluggable object storage via the `object_store` crate. Set `OBJECT_STORE_BACKEND` in `backend/.env` to enable it (the app starts without it if the var is unset).

| Env var | Default | Description |
|---------|---------|-------------|
| `OBJECT_STORE_BACKEND` | *(unset — disabled)* | `local` \| `s3` \| `webdav` |
| `OBJECT_STORE_PATH` | `./data` | Root path (local backend only) |
| `OBJECT_STORE_BUCKET` | | S3 bucket name |
| `OBJECT_STORE_URL` | | S3 custom endpoint or WebDAV server URL |
| `OBJECT_STORE_KEY` | | S3 access key ID or WebDAV username |
| `OBJECT_STORE_SECRET` | | S3 secret key or WebDAV password |
| `OBJECT_STORE_REGION` | `us-east-1` | S3 region |

Local filesystem example (`backend/.env`):
```bash
OBJECT_STORE_BACKEND=local
OBJECT_STORE_PATH=./data
```

Makefile prefers `docker compose`, falls back to `podman-compose`.

## Testing

All test targets start an ephemeral ArangoDB container and clean up on exit.

```bash
make test                   # Run ALL tests (Rust + CLI + Python API)
make test-unit              # Rust unit & backend integration tests only
make test-cli               # CLI integration tests (starts backend)
make test-api               # Python API integration tests (starts backend)
```

### Test Matrix

| Type | Location | Needs DB | Needs backend | Command |
|------|----------|----------|---------------|---------|
| Rust unit + backend integration | `backend/src/test/`, CLI unit tests | yes | no (axum-test) | `make test-unit` |
| CLI integration | `cli/tests/cli_test.rs` | yes | yes | `make test-cli` |
| Python API integration | `backend/itests/` | yes | yes | `make test-api` |

### How `make test` Works

1. Start ephemeral ArangoDB
2. Run Rust unit + backend integration tests (`cargo test -p axum-api -p crit-cli`)
3. Start backend process
4. Run CLI integration tests (`cargo test -p crit-cli --test cli_test`)
5. Run Python API tests (`pytest backend/itests/`)
6. Tear down ArangoDB

### Test Details

- Backend integration tests use `axum-test` (in-memory server, no backend process)
- CLI integration tests use `assert_cmd` to run `cr1t` binary with temp `HOME` for isolation
- Python itests use `pytest` with `requests` against `localhost:3742`
- `cargo test test_name` runs a single test (requires ArangoDB running)
