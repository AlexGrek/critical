# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Critical (crit-cli) is a full-stack project management and ticketing system with a Rust backend and React TypeScript frontend, using ArangoDB as the primary database.

Whitepaper for architectural constraints: [`WHITEPAPER`](WHITEPAPER.md)

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
make populate-db            # Populate dev DB with test users, groups, projects (requires backend running)
make kill                   # Kill any stalled axum-api backend processes (by name + port 3742)
make test                   # Start ephemeral ArangoDB, run tests, tear down DB on exit
```

### Frontend (React Router + Vite)
```bash
cd frontend
npm run dev                 # Dev server on port 5173 (proxies API to localhost:3742)
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

> **IMPORTANT**: After resetting or restarting the dev database (`make reset-db`, `make run-fresh`, or any `make reset`), you **must restart the backend** (`make run` or rerun `cargo run --bin axum-api`). The backend creates all collections and initial schema on startup via `connect_basic` — if ArangoDB was not running when the backend started, the database will not be initialized and all API calls will fail with 500 errors.

### Test Database Seed Data (`test-db/`)

`make populate-db` imports predefined test data into a running dev database using `cr1t apply`. Idempotent — safe to run multiple times.

```bash
make run-fresh          # Terminal 1: clean DB + backend
make populate-db        # Terminal 2: populate with test data
```

**Bootstrap flow** (`test-db/populate.sh`):
1. Registers `admin` user via `/api/v1/register`
2. Logs in as `admin` via `cr1t login`
3. Applies YAML files in numbered order: permissions → users → groups → memberships → projects

**Test users** (all passwords are `{username}123`):

| User    | Password   | Role / Permissions |
| ------- | ---------- | ------------------ |
| admin   | admin123   | Godmode (all super-permissions) |
| alice   | alice123   | Engineering lead, group/project creator |
| bob     | bob123     | Senior dev, group/project creator |
| carol   | carol123   | DevOps engineer, group creator |
| dave    | dave123    | Junior dev (basic permissions) |
| eve     | eve123     | QA engineer (basic permissions) |

**Test groups**: `g_platform_admins`, `g_engineering`, `g_devops`, `g_viewers` — each with different ACLs and members.

**Test projects**: `critical`, `infra`, `docs` — with tiered ACLs (ROOT/WRITE/READ per group).

YAML files in `test-db/`: `00-permissions.yaml`, `01-users.yaml`, `02-groups.yaml`, `03-memberships.yaml`, `04-projects.yaml`.

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

Stack architecture: nginx gateway (:3742) routes `/api/*` to the backend and `/*` to the frontend SSR server. See [`dist/README.md`](dist/README.md) for env vars and details.

## Architecture

### Shared Library (`shared/`)
- **Crate name**: `crit-shared` (import as `crit_shared`)
- **Models** (`src/data_models.rs`, `src/util_models.rs`): Domain types and ACL utilities shared across backend and CLI
  - `data_models.rs`: Core entities (`User`, `Group`, `Ticket`, `Project`, `GroupMembership`)
  - `util_models.rs`: Bitflag-based `Permissions`, `AccessControlList` / `AccessControlStore` for ACL management, and super-permission constants
  - ArangoDB uses `_key` as the document ID field (note `#[serde(rename = "_key")]` on model ID fields)
  - User IDs are prefixed with `u_`, group IDs with `g_`
- **`#[crit_resource]` proc macro** (`shared/derive/`): Attribute macro that injects standard fields (`id`, `labels`, `annotations`, `state`, `acl`, `deletion`, `hash_code`) and generates `{Name}Brief` summary struct, `to_brief()`, `brief_field_names()`, `compute_hash()`, `collection_name()`, `id_prefix()`. `labels` and `annotations` are user-managed desired state; `state` (`ResourceState`) holds server-managed audit timestamps.

### Backend (`backend/`)
- **Framework**: Axum 0.8 with Tokio async runtime
- **Package name**: `axum-api` (Cargo.toml)
- **Entry point**: `src/main.rs` — creates `AppState`, connects to DB, builds router
- **Config**: `src/config.rs` — loads from env vars via `dotenvy` (.env file in `backend/`)
  - `DB_CONNECTION_STRING` — ArangoDB URL (required, e.g. `http://localhost:8529`)
  - `DB_NAME` — database name (default: `unnamed`)
  - `DB_USER` — ArangoDB user (default: `root`)
  - `DB_PASSWORD` — ArangoDB password (default: empty)
  - `PORT`, `HOST`, `JWT_SECRET`, `CLIENT_API_KEYS`
- Re-exports models from `crit-shared` via `pub use crit_shared::models` in `main.rs`

### Database Schema
- See [`DATABASE.md`](DATABASE.md) for collection definitions, relationships, edge graphs, and conventions
- **Always update `DATABASE.md`** when making schema changes (new collections, key changes, new edges/indexes)

### Database Layer (`backend/src/db/`)
- **`ArangoDb`** (`src/db/arangodb/mod.rs`): database layer using `arangors` crate (no trait abstraction — direct struct usage)
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

Controllers use a **trait-based dispatch** pattern for the generic gitops API (`/v1/global/{kind}/...`).

- **`KindController` trait** (`gitops_controller.rs`): Defines per-kind authorization and document transformation. Every resource kind must implement this trait. Methods:
  - `can_read(user_id, doc)` / `can_write(user_id, doc)` — authorization checks
  - `can_create(user_id, body)` — authorization for new document creation (default delegates to `can_write(user_id, None)`; override when the request body is needed for auth, e.g. MembershipController checks the target group's ACL)
  - `to_internal(body, auth)` / `to_external(doc)` — convert between external API format and internal ArangoDB format
  - `to_list_external(doc)` — convert for list responses (default delegates to `to_external`; override to return brief/summary fields only)
  - `list_projection_fields()` — ArangoDB field names to fetch for list queries (AQL `KEEP()`); `None` = fetch all fields
  - `prepare_create(body, user_id)` — pre-creation hook (e.g. inject creator ACL for projects and groups)
  - `after_create(key, user_id, db)` — post-creation hook (e.g. insert creator as group member)
  - `after_delete(key, db)` — post-deletion hook (e.g. cascade removal of memberships for users and groups)
  - `after_update(key, db)` — post-update hook (e.g. empty-group auto-deletion; only fires on actual updates, NOT on upsert-creates)
  - `is_scoped()` — returns `true` for project-scoped resource kinds (served at `/v1/projects/{project}/{kind}`)
  - `super_permission()` — returns a super-permission key that short-circuits ACL checks, or `None`
  - `check_hybrid_acl(doc, principals, required, project_acl)` — checks a document's own ACL; if empty, falls back to the project's full ACL (no scope filtering)
- **Dispatch**: `Controller::for_kind(kind)` in `mod.rs` returns `&dyn KindController`, matching `"users"` → `UserController`, `"groups"` → `GroupController`, `"projects"` → `ProjectController`, `"memberships"` → `MembershipController`, and falling back to `DefaultKindController` (fully permissive) for unknown kinds.
- **Shared helpers** (`gitops_controller.rs`): `standard_to_internal()`, `standard_to_external()`, `rename_id_to_key()`, `rename_key_to_id()`, `parse_acl()` — reused by all controller implementations.
- **Scoped ACL model**: Project-scoped resources (e.g. tasks, deployments) use a two-level ACL fallback. Each resource checks its own `acl.list` first; if empty, the parent project's full `acl.list` is used — **all entries apply regardless of `scope` field**. The `scope` field on `AccessControlList` is retained for backwards compatibility with old documents but is no longer evaluated during permission checks. Group membership changes may take up to 5 seconds to propagate to permission checks — principal resolution is cached with a 5s TTL via `AppState::get_cached_principals()` (see `cache::PRINCIPALS_CACHE`). There is no cache invalidation; the system relies on TTL expiry, which is acceptable because group membership changes are infrequent.

**When adding a new resource kind:**
1. Create a new controller file in `controllers/` with a struct holding `Arc<ArangoDb>`
2. Implement `KindController` for it (use `#[async_trait]`)
3. Add the controller as a field on `Controller` in `mod.rs`
4. Add a match arm in `Controller::for_kind()`
5. No changes needed in the gitops route handlers — dispatch is automatic

### Services (`backend/src/services/`)
- `github.rs` — GitHub integration
- `offloadmq.rs` — message queue integration

### Frontend (`frontend/`)
- **React 19** with **React Router 7.5** (SSR enabled)
- **TailwindCSS 4** for styling with **5 visual themes**: light, dark, barbie (very round), orange (very minimal), grayscale (no roundness)
- **Vite 6** as build tool
- See [`frontend/README.md`](frontend/README.md) for routes, setup, and architecture
- **Theme system**: All components use theme-dependent CSS variables for colors and border radius
  - Barbie theme: Very round edges (pill-shaped buttons, rounded cards)
  - Orange theme: Very minimal roundness (sharp, utilitarian aesthetic)
  - Grayscale theme: NO roundness (completely sharp, brutalist aesthetic)
  - Light/Dark themes: Standard roundness
  - CSS variables: `--radius-component`, `--radius-component-lg`, `--radius-component-xl`
- **App layout** (`frontend/app/layouts/app-layout.tsx`): wraps all routes — fixed `TopBar` (height 56px, z-50) with animated `{!}` logo toggle, collapsible `SideMenu` (width 256px, slides in/out with Framer Motion). Desktop: sidebar open by default, content shifts right. Mobile: sidebar overlays with backdrop.
- **Current routes** (all wrapped by `app-layout.tsx`):
  - `/` (home)
  - `/sign-in`, `/sign-up` (authentication)
  - `/groups` (groups listing with ACL display)
  - `/ui-gallery` (component showcase with theme switcher)
- **Data fetching**: Uses React Router loaders for server-side data fetching
- **API integration**: Fetches from `/api/v1/global/{kind}` endpoints with JWT authentication

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
- `AppState` holds config, auth, DB (`Arc<ArangoDb>`), controllers, and optional services
- Shared via `Arc<AppState>` across all routes

### Testing

Four test categories:

| Type                            | Location                            | Needs DB | Needs backend  | Command          |
| ------------------------------- | ----------------------------------- | -------- | -------------- | ---------------- |
| Rust unit + backend integration | `backend/src/test/`, CLI unit tests | yes      | no (axum-test) | `make test-unit` |
| CLI integration                 | `cli/tests/cli_test.rs`             | yes      | yes            | `make test-cli`  |
| Python API integration          | `backend/itests/`                   | yes      | yes            | `make test-api`  |
| Frontend E2E (Playwright)       | `e2e-tests/e2e/`                    | yes      | yes            | see below        |

#### Backend/CLI/Python Tests

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

#### Frontend E2E Tests (Playwright)

Location: `e2e-tests/` directory contains Playwright tests (`.spec.ts`)

**IMPORTANT**: These tests use **real API calls** to the actual dev server and database (not mocked).

Prerequisites:
```bash
make run              # Terminal 1: Start backend + ArangoDB
cd frontend && npm run dev  # Terminal 2: Start frontend dev server
```

Run tests:
```bash
cd e2e-tests
npm install           # First time only

npm run test          # All tests
npm run test:headed   # With visible browser
npm run test:debug    # Debug mode
npm run test:chrome   # Chrome only
npm run test:firefox  # Firefox only
npm run test:webkit   # Safari/WebKit only
```

Test characteristics:
- Creates **unique random users** per test run (`testuser_{timestamp}_{random}`) to avoid conflicts
- Creates test data (groups, etc.) via real API calls before tests
- Automatically cleans up created data after tests complete
- Uses real ArangoDB database, not mocks
- Multiple developers can run tests simultaneously without interference

Current test files:
- `auth.spec.ts` — Authentication flows
- `home.spec.ts` — Home page
- `groups.spec.ts` — Groups page (creation, display, ACL, real-time updates)
