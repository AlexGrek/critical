# Architecture

Critical is a full-stack project management platform with a Rust backend, React TypeScript frontend, and ArangoDB database.

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
- **Core entities** (`data_models.rs`): `User`, `Group`, `ServiceAccount`, `PipelineAccount`, `GroupMembership`, `GlobalPermission`
- **Utility types** (`util_models.rs`): Bitflag `Permissions`, `AccessControlStore`, `ResourceState`, `RuntimeState`, `DeletionInfo`, `HistoryEntry`, `ResourceEvent`, `FullResource`, `UnprocessedImage`, `PersistentFile`, `PersistentFileUri`
- ArangoDB `_key` mapping via `#[serde(rename = "_key")]` on `id` fields
- ID prefixes: users `u_`, groups `g_`, service accounts `sa_`, pipeline accounts `pa_`
- **`#[crit_resource]` proc macro** (`shared/derive/`): injects standard fields (`id`, `labels`, `annotations`, `state`, `acl`, `deletion`, `hash_code`) into resource structs; generates `{Name}Brief`, `to_brief()`, `brief_field_names()`, `compute_hash()`, `collection_name()`, `id_prefix()`

### Backend (`backend/`)

- **Framework**: Axum 0.8 + Tokio
- **Package**: `axum-api`
- **Entry point**: `src/main.rs` — creates `AppState`, connects to DB, builds router
- **State** (`src/state.rs`): `AppState` holds config, auth, DB (`Arc<ArangoDb>`), controllers, optional services, and `image_processing_semaphore: Arc<Semaphore>` (limits background image conversion to one task at a time); shared via `Arc<AppState>`
- **Database layer** (`src/db/arangodb/mod.rs`): Direct `ArangoDb` struct using `arangors` crate — auto-creates collections on startup
- **Controllers** (`src/controllers/`): `user_controller`, `group_controller`, `membership_controller`; all implement `KindController` trait
- **Middleware** (`src/middleware/`): JWT auth applied to all `/v1` routes; `/v1/static/*` is registered on the outer router and intentionally bypasses this layer
- **Services** (`src/services/`):
  - `objectstore.rs` — pluggable object storage (local filesystem, S3, WebDAV) via the `object_store` crate; selected by `OBJECT_STORE_BACKEND` env var; optional at runtime
  - `image_processing.rs` — pure-Rust image pipeline: magic-byte format detection, center-crop (integer arithmetic, no float rounding), Lanczos3 resize, in-memory WebP encode; produces HD + thumbnail for avatars (480×480 / 128×128 px) and wallpapers (1400×600 / 300×128 px)
  - `github.rs` — GitHub integration
  - `offloadmq.rs` — message queue integration

### Frontend (`frontend/`)

- **React 19** with **React Router 7.5** (SSR enabled)
- **TailwindCSS 4** for styling, **Vite 6** as build tool
- API proxy in `vite.config.ts` → `http://localhost:3742`
- UI components in `app/components/` (Button, Input, Modal, Card, etc.)
- 5 visual themes: `light`, `dark`, `barbie`, `orange`, `grayscale`
- Routes in `app/routes/`: `/`, `/sign-in`, `/sign-up`, `/groups`, `/ui-gallery`

### CLI (`cli/`)

- **Binary**: `cr1t` — gitops-style tool, similar to `kubectl`
- Full alternative to the web frontend
- See [CLI documentation](cli.md)

## Controller Dispatch (KindController)

The gitops API routes (`/v1/global/{kind}`) use a **trait-based dispatch** pattern:

```
Request → Controller::for_kind(kind) → &dyn KindController
                                              │
              ┌───────────────────────────────┤
              ▼                               ▼
        UserController             GroupController
        MembershipController       DefaultKindController
```

Each `KindController` implementation handles:
- `can_read` / `can_write` / `can_create` — authorization
- `to_internal` / `to_external` / `to_list_external` — document transformation
- `prepare_create` / `after_create` / `after_delete` / `after_update` — lifecycle hooks

Adding a new resource kind: new controller file → implement `KindController` → add to `Controller::for_kind()`. No changes to route handlers.

## Production Stack

```
        :3742
          |
      [ gateway ]  (nginx:alpine)
       /        \
  /api/*         /*
    |              |
 [ api ]     [ frontend ]
(axum-api)  (cr1t-frontend)
    |
[ arangodb ]
```

All traffic enters through nginx on port 3742. `/api/*` routes to the Rust backend, everything else to the React SSR frontend.
