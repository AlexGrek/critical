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
```

### Frontend
```bash
cd frontend
npm run dev                 # Dev server on port 5173 (proxies API to localhost:3742)
npm run build               # Production build
npm run typecheck           # react-router typegen && tsc
```

### Database
```bash
make run-db                 # Start ArangoDB container (port 8529)
make stop-db                # Stop container
make reset-db               # Stop and delete volumes
```

> **IMPORTANT**: After resetting or restarting the dev database, you **must restart the backend**. The backend creates all collections on startup via `connect_basic` — if ArangoDB was not running when the backend started, all API calls will fail with 500 errors.

### Testing
```bash
make test                   # Run ALL test types (DB + backend started automatically)
make test-unit              # Rust unit + backend tests only (starts ephemeral DB)
make test-cli               # CLI integration tests (starts DB + backend)
make test-api               # Python API tests (starts DB + backend)
```

Playwright E2E tests: `cd e2e-tests && npm run test` (requires backend + frontend dev server running).

See [`docs/development.md`](docs/development.md) for full development and testing details.

### Test Database Seed Data (`test-db/`)

`make populate-db` imports test data using `cr1t apply`. Idempotent. All test user passwords are `{username}123`.

| User  | Role |
|-------|------|
| admin | Godmode (all super-permissions) |
| alice | Engineering lead, group/project creator |
| bob   | Senior dev, group/project creator |
| carol | DevOps engineer, group creator |
| dave  | Junior dev (basic permissions) |
| eve   | QA engineer (basic permissions) |

### Deployment

See [`docs/deployment.md`](docs/deployment.md) for Docker Compose, Helm/Kubernetes, and cross-compilation.

## Architecture

See [`docs/architecture.md`](docs/architecture.md) for full architecture details.

### Key Constraints

- **Database schema**: See [`DATABASE.md`](DATABASE.md). **Always update `DATABASE.md`** when making schema changes.
- **No migration system**: ArangoDB is schemaless; Rust structs define the schema. Adding `Option<T>` or `#[serde(default)]` fields is safe. Adding required fields without defaults breaks old documents.
- **Adding a new collection**: add to `VERTEX_COLLECTIONS`/`EDGE_COLLECTIONS` in `init.rs`, add handle in `open_collections`, add field on `ArangoDb` struct in `mod.rs`.
- **ID conventions**: users `u_`, groups `g_`, service accounts `sa_`, pipeline accounts `pa_`. ArangoDB `_key` ↔ Rust `id` via serde rename.
- **Soft-delete**: `deletion: Option<DeletionInfo>` — list/get queries filter `doc.deletion == null`; DELETE uses `generic_soft_delete`.
- **ACL security**: ACL denials return 404 (not 403) to avoid leaking resource existence.
- **Principal cache**: Group membership changes take up to 5s to propagate (TTL-based cache, no invalidation).

### Controllers

See [`docs/architecture.md`](docs/architecture.md) for the `KindController` trait and dispatch pattern.

**When adding a new resource kind:**
1. Create a new controller file in `controllers/` with a struct holding `Arc<ArangoDb>`
2. Implement `KindController` for it (use `#[async_trait]`)
3. Add the controller as a field on `Controller` in `mod.rs`
4. Add a match arm in `Controller::for_kind()`
5. No changes needed in the gitops route handlers — dispatch is automatic

### API Routes

See [`docs/api.md`](docs/api.md) for full route documentation.

### Gitops Handler Lifecycle & Response Contracts

See [`docs/gitops-controller.md`](docs/gitops-controller.md) for:
- Per-operation handler lifecycle (create / update / list / fetch / delete)
- **Response shapes**: POST create and PUT update return the **full document** (not just `{"id": ...}`) — callers can read `hash_code` directly from the write response
- The `doc_snapshot` pattern — why we clone before the DB call instead of issuing a follow-up GET
- How to implement `to_external` and `to_list_external` for new kinds
- Brief field control: `list_projection_fields()` (DB-level KEEP) + `to_list_external()` (Rust filter)

### Models & Resources

See [`docs/models.md`](docs/models.md) for the `#[crit_resource]` proc macro, model contracts, and brief structs.

### Access Control

See [`docs/access-control.md`](docs/access-control.md) for permissions, ACLs, and the scoped ACL fallback model.

### Frontend

See [`frontend/README.md`](frontend/README.md) and [`docs/HOW_TO_MAKE_FRONTEND_THEMES.md`](docs/HOW_TO_MAKE_FRONTEND_THEMES.md).

Key rules:
- **Theme-aware roundness**: Use `rounded-(--radius-component)` — NEVER hardcoded `rounded-md`, `rounded-lg`, etc.
- **5 themes**: light, dark, barbie (very round), orange (minimal roundness), grayscale (no roundness)
- **Custom components** in `frontend/app/components/` — reuse them, don't duplicate styling
- **Component styling encapsulation**: Never leak styling into usage sites. Use component props (e.g. `variant`, `size`) instead of className overrides.

### CLI

See [`docs/cli.md`](docs/cli.md) for CLI commands, context management, and gitops workflows.

### Integration Tests

- **Always write Python integration tests** for new API endpoints in `backend/itests/tests/`
- Managed with PDM: `cd backend/itests && pdm run pytest tests/ -v`
- Pattern: register random user in `@pytest.fixture(scope="module")`, clean up after tests
- **Playwright E2E tests** for frontend in `e2e-tests/e2e/` — add `data-testid` attributes for selectors
