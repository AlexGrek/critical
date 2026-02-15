# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Critical (crit-cli) is a full-stack project management and ticketing system with a Rust backend and React TypeScript frontend, using ArangoDB as the primary database.

## Workspace Structure

Cargo workspace with three crates:
- **`shared/`** (`crit-shared`) — shared library with domain models, used by both backend and CLI
- **`backend/`** (`axum-api`) — Axum web server
- **`cli/`** (`crit-cli`) — CLI tool (placeholder)

## Build & Development Commands

### Workspace (Rust)
```bash
cargo build                 # Build all workspace crates
cargo build --bin crit-cli  # Build CLI only
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
- **`DatabaseInterface` trait** (`src/db/mod.rs`): async trait with `Transaction` support, defines all DB operations (CRUD for users, groups, memberships)
- **`ArangoDb`** (`src/db/arangodb/mod.rs`): sole implementation using `arangors` crate
- `connect_basic` auto-creates the database and collections on first connection

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

### State Management (`backend/src/state.rs`)
- `AppState` holds config, auth, DB interface (`Arc<dyn DatabaseInterface>`), and optional services
- Shared via `Arc<AppState>` across all routes

### Testing
- All tests require ArangoDB running (use `make test` for ephemeral instance)
- `backend/src/test/` contains Rust integration tests (e.g., `login_test.rs`)
- `create_mock_shared_state()` in `main.rs` is async — connects to ArangoDB from `.env`
- `backend/itests/` contains Python-based integration tests (pytest)
