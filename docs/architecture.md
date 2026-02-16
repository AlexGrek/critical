# Architecture

Critical is a full-stack project management and ticketing system with a Rust backend, React TypeScript frontend, and ArangoDB database.

## Workspace Structure

Cargo workspace with three crates plus a frontend:

```
├── shared/            # Shared Rust library (crit-shared) — domain models
├── backend/           # Axum API server (axum-api)
├── cli/               # CLI tool (binary: cr1t)
├── frontend/          # React Router 7 + Vite frontend (SSR)
├── dist/              # Docker deployment (compose, nginx, Helm chart)
│   ├── cli/           # CLI installer script
│   └── helm/          # Kubernetes Helm chart
├── Makefile           # Dev/test orchestration
└── Makefile.xplatform # Cross-compilation for CLI
```

## Component Overview

### Shared Library (`shared/`)

- **Crate**: `crit-shared` (import as `crit_shared`)
- Domain models shared across backend and CLI
- Core entities: `User`, `Group`, `GroupMembership`, `Ticket`, `Project`
- Bitflag-based `Permissions` (FETCH, LIST, NOTIFY, CREATE, MODIFY, CUSTOM1, CUSTOM2)
- `AccessControlList` / `AccessControlStore` for ACL management
- ArangoDB `_key` mapping via `#[serde(rename = "_key")]` on `id` fields
- ID prefixes: users `u_`, groups `g_`

### Backend (`backend/`)

- **Framework**: Axum 0.8 + Tokio
- **Package**: `axum-api`
- **Entry point**: `src/main.rs` — creates `AppState`, connects to DB, builds router
- **State** (`src/state.rs`): `AppState` holds config, auth, DB interface (`Arc<dyn DatabaseInterface>`), optional services; shared via `Arc<AppState>`
- **Database layer** (`src/db/`): `DatabaseInterface` trait with `ArangoDb` implementation using `arangors` crate
- **Controllers** (`src/controllers/`): `user_controller`, `group_controller`, `project_controller`, `ticket_controller`
- **Middleware** (`src/middleware/`): JWT auth applied to `/v1` routes
- **Services** (`src/services/`): `github.rs` (GitHub integration), `offloadmq.rs` (message queue)

### Frontend (`frontend/`)

- **React 19** with **React Router 7.5** (SSR enabled)
- **TailwindCSS 4** for styling, **Vite 6** as build tool
- API proxy in `vite.config.ts` → `http://localhost:3742`
- UI toolkit in `app/toolkit/` (buttons, modals, typography)
- Routes in `app/routes/` (dashboard, auth, projects, tickets, pipelines)

### CLI (`cli/`)

- **Binary**: `cr1t` — gitops-style tool, similar to `kubectl`
- Full alternative to the web frontend
- See [CLI documentation](cli.md)

## Production Stack

```
        :8080
          |
      [ gateway ]  (nginx:alpine)
       /        \
  /api/*         /*
    |              |
 [ api ]     [ frontend ]
(cr1t-api)  (cr1t-frontend)
    |
[ arangodb ]
```

All traffic enters through nginx on port 3742. `/api/*` routes to the Rust backend, everything else to the React SSR frontend.
