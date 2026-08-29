# Development

## Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Task](https://taskfile.dev) (`brew install go-task`) — every command below is a task in [`Taskfile.yml`](../Taskfile.yml)
- [Docker](https://www.docker.com/) (for ArangoDB and cross-compilation)
- [Node.js](https://nodejs.org/) (for frontend)
- [Python 3](https://www.python.org/) + pytest (for API integration tests)

## Build Commands

### Workspace (Rust)

```bash
cargo build                 # Build all workspace crates
cargo build --bin cr1t      # Build CLI only
cargo build --bin axum-api  # Build backend only
task build                  # Quick dev build (all crates)
```

Run `task` with no arguments to list every available task.

### Running Locally

```bash
task dev                    # ArangoDB + backend (cargo watch) + frontend dev server, all in one
task run                    # Start ArangoDB + run backend only (persistent DB)
task run:fresh              # Reset DB volumes, then run (clean slate)
task kill                   # Kill stalled backend (:3742) / frontend (:5173) processes
```

Frontend on its own (separate terminal):
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
task db:up                  # Start ArangoDB container (port 8529), wait for readiness
task db:down                # Stop container
task db:reset               # Stop and delete volumes
task db:logs                # Tail container logs
task db:populate            # Seed the dev DB (requires a running backend)
task db:show                # Dump all dev DB data as admin (requires a running backend)
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

The Taskfile prefers `docker compose`, falls back to `podman-compose`.

## Testing

All test targets start an ephemeral ArangoDB container and clean up on exit.

```bash
task test                   # Run ALL tests (Rust + CLI + Python API + E2E)
task test:unit              # Rust unit & backend integration tests only
task test:cli               # CLI integration tests (starts backend)
task test:api               # Python API integration tests (starts backend)
task test:e2e               # Playwright E2E tests (starts backend)
```

### Test Matrix

| Type | Location | Needs DB | Needs backend | Command |
|------|----------|----------|---------------|---------|
| Rust unit + backend integration | `backend/src/test/`, CLI unit tests | yes | no (axum-test) | `task test:unit` |
| CLI integration | `cli/tests/cli_test.rs` | yes | yes | `task test:cli` |
| Python API integration | `backend/itests/` | yes | yes | `task test:api` |
| Playwright E2E | `e2e-tests/e2e/` | yes | yes | `task test:e2e` |

Against an already-running backend, `task itest` (parallel) and `task itest:seq`
run the Python suite without managing any infrastructure.

### How `task test` Works

1. Start ephemeral ArangoDB
2. Run Rust unit + backend integration tests (`cargo test -p axum-api -p crit-cli`)
3. Start backend process
4. Run CLI integration tests (`cargo test -p crit-cli --test cli_test`)
5. Run Python API tests (`pytest backend/itests/`)
6. Run Playwright E2E tests (`e2e-tests/`)
7. Tear down ArangoDB

### Test Details

- Backend integration tests use `axum-test` (in-memory server, no backend process)
- CLI integration tests use `assert_cmd` to run `cr1t` binary with temp `HOME` for isolation
- Python itests use `pytest` with `requests` against `localhost:3742`
- `cargo test test_name` runs a single test (requires ArangoDB running)
