# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Critical (crit-cli) is a full-stack project management and ticketing system with a Rust backend and React TypeScript frontend, using ArangoDB as the primary database.

Full documentation: [`docs/`](docs/README.md)

## Workspace Structure

Cargo workspace with three crates:
- **`shared/`** (`crit-shared`) — shared library with domain models, used by both backend and CLI
- **`backend/`** (`axum-api`) — Axum web server
- **`cli/`** (`crit-cli`) — gitops-style CLI tool (binary: `cr1t`), a full alternative to the frontend

## Build & Development Commands

### Workspace (Rust)
```bash
cargo build                 # Build all workspace crates
cargo build --bin cr1t      # Build CLI only
cargo build --bin axum-api  # Build backend only
cargo test                  # Run all Rust tests (requires ArangoDB running)
cargo test test_name        # Run a single test
make dev                    # Quick dev build (all crates)
```

### Running
```bash
make run                    # Start ArangoDB + run backend (persistent DB, stops container on exit)
make run-fresh              # Reset DB volumes, then run (clean slate)
make test                   # Start ephemeral ArangoDB, run tests, tear down DB on exit
```

### Frontend (React Router + Vite)
```bash
cd frontend
npm run dev                 # Dev server on port 5173 (proxies API to localhost:8080)
npm run build               # Production build
npm run typecheck           # react-router typegen && tsc
npm start                   # Serve production build
```

### Database (ArangoDB via Docker)
```bash
make run-db                 # Start ArangoDB container (port 8529)
make stop-db                # Stop container
make reset-db               # Stop and delete volumes
make logs-db                # Tail container logs
```

ArangoDB web UI is at `http://localhost:8529`. Makefile prefers docker, falls back to podman-compose.

### Cross-Compilation
```bash
make -f Makefile.xplatform build-all     # Build for all 9 target platforms
make -f Makefile.xplatform release       # Full release with archives
```

### Docker / Deployment (`dist/`)
```bash
cd dist
make build                  # Build API + frontend images locally (current arch)
make build-push             # Build multi-arch (amd64+arm64) and push to Docker Hub
make up                     # Start full stack (API + frontend + ArangoDB + nginx gateway)
make down                   # Stop stack
make logs                   # Tail all service logs
make status                 # Show running containers
make reset                  # Tear down + remove volumes
```

Images: `grekodocker/cr1t-api`, `grekodocker/cr1t-frontend` (Docker Hub, multi-arch)

Stack architecture: nginx gateway (:8080) routes `/api/*` to the backend and `/*` to the frontend SSR server. See [`dist/README.md`](dist/README.md) for env vars and details.

## Architecture

### Shared Library (`shared/`)
- **Crate name**: `crit-shared` (import as `crit_shared`)
- **Models** (`src/models.rs`): Domain types shared across backend and CLI
  - Bitflag-based `Permissions` (FETCH, LIST, NOTIFY, CREATE, MODIFY, CUSTOM1, CUSTOM2)
  - `AccessControlList` / `AccessControlStore` for ACL management
  - Core entities: `User`, `Group`, `Ticket`, `Project`, `GroupMembership`
  - ArangoDB uses `_key` as the document ID field (note `#[serde(rename = "_key")]` on model ID fields)
  - User IDs are prefixed with `u_`, group IDs with `g_`

### Backend (`backend/`)
- **Framework**: Axum 0.8 with Tokio async runtime
- **Package name**: `axum-api` (Cargo.toml)
- **Entry point**: `src/main.rs` — creates `AppState`, connects to DB, builds router
- **Config**: `src/config.rs` — loads from env vars via `dotenvy` (.env file in `backend/`)
  - `DB_CONNECTION_STRING` — ArangoDB URL (required, e.g. `http://localhost:8529`)
  - `DB_NAME` — database name (default: `unnamed`)
  - `DB_USER` — ArangoDB user (default: `root`)
  - `DB_PASSWORD` — ArangoDB password (default: empty)
  - `PORT`, `HOST`, `JWT_SECRET`, `MGMT_TOKEN`, `CLIENT_API_KEYS`
- Re-exports models from `crit-shared` via `pub use crit_shared::models` in `main.rs`

### Database Schema
- See [`DATABASE.md`](DATABASE.md) for collection definitions, relationships, edge graphs, and conventions
- **Always update `DATABASE.md`** when making schema changes (new collections, key changes, new edges/indexes)

### Database Layer (`backend/src/db/`)
- **`DatabaseInterface` trait** (`src/db/mod.rs`): async trait with `Transaction` support, defines all DB operations (CRUD for users, groups, memberships, permissions)
- **`ArangoDb`** (`src/db/arangodb/mod.rs`): sole implementation using `arangors` crate
- `connect_basic` auto-creates the database and collections on first connection (idempotent — silently ignores "already exists" errors)
- **No migration system**: ArangoDB is schemaless; Rust structs define the application-level schema, not a DB-enforced one
- Adding `Option<T>` or `#[serde(default)]` fields is safe — old documents deserialize fine. Adding required fields without defaults breaks deserialization of old documents. Renames require manual data fixup.
- Adding a new collection requires a `create_collection` call in both `connect_basic` and `connect_anon`, plus a new field on the `ArangoDb` struct

### Routing
- `/health` — health check (unauthenticated)
- `/register`, `/login` — authentication endpoints (unauthenticated)
- `/v1/*` — JWT-protected API routes (WebSocket at `/v1/ws`)
- `/swagger-ui` — OpenAPI docs (utoipa auto-discovery is commented out due to IDE issues)
- All API routes are nested under `/api` in the OpenAPI router

### Auth & Middleware (`backend/src/middleware/`)
- JWT authentication middleware applied to `/v1` routes
- `Auth` struct initialized with JWT secret bytes
- Three auth strategies: JWT, management token, API key

### Controllers (`backend/src/controllers/`)
- `user_controller`, `group_controller`, `project_controller`, `ticket_controller`

### Services (`backend/src/services/`)
- `github.rs` — GitHub integration
- `offloadmq.rs` — message queue integration

### Frontend (`frontend/`)
- **React 19** with **React Router 7.5** (SSR enabled)
- **TailwindCSS 4** for styling
- **Vite 6** as build tool
- API proxy configured in `vite.config.ts` pointing to `http://localhost:8080`
- UI toolkit in `app/toolkit/` (buttons, modals, typography)
- Routes in `app/routes/` (dashboard, auth, projects, tickets, pipelines)

### CLI (`cli/`)
- **Binary name**: `cr1t` (gitops-style, similar to `kubectl`)
- **Purpose**: Full-featured CLI alternative to the frontend for managing projects, tickets, pipelines, etc.
- **Auth**: Authenticates via long-lived JWT stored in `~/.cr1tical/context.yaml`
- **Context system**: Supports multiple server contexts (like kubeconfigs) — switch with `cr1t context use <name>`
- **Current commands**: `login`, `context list`, `context use`
- **Future login methods**: Only username/password for now; additional methods (API keys, SSO, etc.) planned
- **Registration**: Not supported from CLI — users must register via the frontend or API
- **Key files**:
  - `src/main.rs` — clap-based CLI entrypoint and command routing
  - `src/context.rs` — context file load/save (`~/.cr1tical/context.yaml`)
  - `src/api.rs` — HTTP client calls to the backend API
  - `src/commands/` — command implementations (one file per command group)

### State Management (`backend/src/state.rs`)
- `AppState` holds config, auth, DB interface (`Arc<dyn DatabaseInterface>`), and optional services
- Shared via `Arc<AppState>` across all routes

### Testing

Three test categories, all orchestrated via Makefile with ephemeral ArangoDB:

| Type | Location | Needs DB | Needs backend | Command |
|------|----------|----------|---------------|---------|
| Rust unit + backend integration | `backend/src/test/`, CLI unit tests | yes | no (axum-test) | `make test-unit` |
| CLI integration | `cli/tests/cli_test.rs` | yes | yes | `make test-cli` |
| Python API integration | `backend/itests/` | yes | yes | `make test-api` |

```bash
make test                   # Run ALL test types (DB + backend started automatically)
make test-unit              # Rust unit + backend tests only (starts ephemeral DB)
make test-cli               # CLI integration tests (starts DB + backend)
make test-api               # Python API tests (starts DB + backend)
```

- `make test` orchestrates: start DB → Rust tests → start backend → CLI tests → Python tests → cleanup
- Each target starts its own ephemeral ArangoDB and tears it down on exit
- `create_mock_shared_state()` in `main.rs` is async — connects to ArangoDB from `.env`
- CLI integration tests use `assert_cmd` to run `cr1t` binary with temp `HOME` for isolation
- Python itests use `pytest` with `requests` library against `localhost:3742`
