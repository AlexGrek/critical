# Critical — Whitepaper

Critical is a developer-friendly project management platform with web and CLI interfaces. The goal: replace Jira and Confluence, use the same app to control pipelines, deployments, and the full product lifecycle — bug tracking, sprint and kanban boards, project wiki — all from a single, hackable, API-first system.

## Core Philosophy

### Critical = GitOps for the product lifecycle

Every entity in the company is a **resource**.

Every resource is **declarative** — you describe desired state, the system converges to it.

Every change is **auditable** — an immutable history trail is kept for every resource.

Every action can be **automated** — the API and CLI are identical entry points; no hidden operations.

Critical is not a tracker — it is an **operating system for development teams**.

### UI is not special

```
Everything the UI can do:
  → the CLI can do
  → the API can do
  → automation can do
```

No hidden operations. No magic behind the frontend.

---

## Architecture Overview

```
┌───────────────┐     ┌──────────────────┐     ┌──────────────┐
│  React/SSR    │     │   cr1t (CLI)     │     │  Automation  │
│  Frontend     │     │   Rust binary    │     │  (CI/CD)     │
└──────┬────────┘     └────────┬─────────┘     └──────┬───────┘
       │                       │                       │
       └───────────────────────┼───────────────────────┘
                               │  HTTPS + JWT
                      ┌────────▼─────────┐
                      │  Axum Backend    │
                      │  (Rust)          │
                      │                  │
                      │  /api/v1/global/ │  ← gitops API
                      │  /api/register   │
                      │  /api/login      │
                      └────────┬─────────┘
                               │
                      ┌────────▼─────────┐
                      │   ArangoDB       │
                      │  (Graph DB)      │
                      └──────────────────┘
```

**Stack:**
- Backend: Rust (Axum 0.8, Tokio async runtime)
- Frontend: React 19, React Router 7.5 (SSR), TailwindCSS 4, Vite 6
- CLI: Rust binary `cr1t` — shares models with the backend via `crit-shared` crate
- Database: ArangoDB (graph database, schemaless, no migration system)
- Auth: JWT (cookie + header), bcrypt password hashing

---

## Data Model

### Resources

Every entity in Critical is a **resource**. Resources have a uniform shape injected by the `#[crit_resource]` Rust attribute macro:

```
┌──────────────────────────────────────────────────────┐
│  Resource                                            │
├──────────────────────────────────────────────────────┤
│  id           String          ArangoDB _key          │
│  labels       Labels          queryable key-values   │
│  annotations  Labels          freeform strings       │
│  state        ResourceState   created_at, created_by,│
│                               updated_at, updated_by │
│  acl          AccessControl   per-document ACL       │
│               Store           (omitted for users)    │
│  deletion     DeletionInfo?   null = active          │
│                               present = soft-deleted │
│  hash_code    String          FNV-1a hash for        │
│                               write-conflict detect  │
└──────────────────────────────────────────────────────┘
  + kind-specific fields (name, description, etc.)

labels and annotations are user-managed desired state.
state is server-managed (audit timestamps) — NOT part of desired state.
```

This is enforced by the `#[crit_derive::crit_resource]` proc macro — not by convention. All resource structs use it; edge collections and simple key-value collections are defined manually.

### Principals

Principals are the entities that can authenticate, hold permissions, and be members of groups. All four principal types are **equal-level nodes** in the membership graph — there is no principal hierarchy.

| Kind | ID Prefix | Purpose |
|------|-----------|---------|
| `users` | `u_` | Human accounts (password auth) |
| `groups` | `g_` | Principal aggregators; can nest |
| `service_accounts` | `sa_` | Non-human API principals (token auth) |
| `pipeline_accounts` | `pa_` | CI/CD-scoped non-human principals (token auth) |

```
users ──────────────────┐
service_accounts ───────┤── memberships (edge) ──▶ groups ──┐
pipeline_accounts ──────┘                                    │
                                                             │ (groups can be
                                                             │  members of groups)
                                                        ◀────┘
```

Membership is stored as an ArangoDB edge collection (`memberships`). Graph traversal resolves transitive group membership for ACL evaluation.

### Access Control

Every resource (except `users`) carries a per-document ACL:

```json
{
  "acl": {
    "list": [
      { "permissions": 31, "principals": ["u_alice", "g_engineering"] }
    ],
    "last_mod_date": "2026-02-23T12:00:00Z"
  }
}
```

Permissions are a **bitflag** (`u8`):

| Bit | Name | Meaning |
|-----|------|---------|
| 0 | FETCH | Read single document |
| 1 | LIST | Appear in list queries |
| 2 | NOTIFY | Receive real-time events |
| 3 | CREATE | Create child resources |
| 4 | MODIFY | Update or delete this document |
| 5-6 | CUSTOM1/2 | Reserved for future use |
| — | READ | FETCH \| LIST \| NOTIFY |
| — | WRITE | CREATE \| MODIFY \| READ |
| — | ROOT | All bits |

**Super-permissions** are global capabilities stored in the `permissions` collection (not per-document):

| Key | Purpose |
|-----|---------|
| `adm_user_manager` | Full CRUD on users and groups |
| `adm_config_editor` | Edit global configuration |
| `usr_create_groups` | Create new groups |

Super-permissions are granted to principals and checked server-side before per-document ACL. New users receive `usr_create_groups` on registration.

### Soft Deletion

Resources are **never hard-deleted** by default. Deletion marks the document with a `DeletionInfo` object and captures connected membership edges for possible restore:

```json
{
  "deletion": {
    "deleted_at": "2026-02-23T12:00:00Z",
    "deleted_by": "u_alice",
    "disconnected_edges": [
      {
        "collection": "memberships",
        "key": "u_bob::g_engineering",
        "from": "users/u_bob",
        "to": "groups/g_engineering"
      }
    ]
  }
}
```

All list and get queries filter `doc.deletion == null` — soft-deleted documents are invisible to normal API calls. Restore (future) will replay `disconnected_edges`.

Group auto-deletion cascades: if deleting a user or group leaves a parent group with zero members, that parent is also soft-deleted recursively.

### Change History & Events

Two auxiliary collections survive resource deletion:

**`resource_history`** — immutable desired-state snapshots, written on every create/update:
```
key format: {kind}_{resource_key}_{revision:06}
e.g.:       users_u_alice_000003
```

**`resource_events`** — runtime events (logins, deployments, etc.):
```
key format: ev_{event_type}_{timestamp_ns}
e.g.:       ev_sign_in_1740312000123456789
```

Currently recorded events: `sign_in` (users, on successful login).

### Storage Relations

Instead of foreign keys, Critical uses **graph edges** for relations. Currently active:

```
user          ──── memberships (edge) ──▶  group
service_acct  ──── memberships (edge) ──▶  group
pipeline_acct ──── memberships (edge) ──▶  group
group         ──── memberships (edge) ──▶  group    (nested groups)
```

Planned future relations (not yet implemented):

```
task          ──── belongs_to ──▶  sprint
task          ──── implements ──▶  feature
bug           ──── caused_by ──▶   release
deployment    ──── deploys ──▶     artifact
artifact      ──── built_from ──▶  pipeline_run
pipeline      ──── triggered_by ─▶ repo
page          ──── references ──▶  task
```

All relations will be first-class and queryable.

---

## API — GitOps Endpoints

The primary API follows a **gitops pattern**: every resource kind is accessed at `/api/v1/global/{kind}`.

```
GET    /api/v1/global/{kind}           list resources (paginated)
POST   /api/v1/global/{kind}           create (id from body)
GET    /api/v1/global/{kind}/{id}      get one resource
POST   /api/v1/global/{kind}/{id}      upsert (create or replace)
PUT    /api/v1/global/{kind}/{id}      update (fails if not exists)
DELETE /api/v1/global/{kind}/{id}      soft-delete
```

Kind dispatch is handled by a **`KindController` trait** — each resource kind has its own controller implementing authorization (`can_read`, `can_write`), document transformation (`to_internal`, `to_external`), and lifecycle hooks (`prepare_create`, `after_create`, `after_delete`, `after_update`). Route handlers are kind-agnostic; all kind-specific logic lives in controllers.

Authentication endpoints:
```
POST /api/register    create user account
POST /api/login       get JWT token (cookie + response body)
POST /api/logout      expire cookie
GET  /health          unauthenticated health check
```

---

## Projects (Namespaces)

Projects act as namespaces for namespaced resources. Every namespaced resource specifies a project:

```
cr1t get tasks -p my-project
cr1t get sprints --project api-v2
```

Projects themselves are global resources (no parent namespace). Each project declares which **services** (feature modules) it enables — controlling which tabs appear in the UI and which resource kinds are accessible under that project's namespace:

| Service | Description |
|---------|-------------|
| `tasks` | Issue tracking with kanban boards (JIRA alternative) |
| `pipelines` | Built-in CI/CD pipeline engine |
| `deployments` | State-controlled deployment management |
| `wikis` | Git-backed documentation (Confluence alternative) |
| `secrets` | Secret management (HashiCorp Vault alternative) |
| `integrations` | Webhooks, GitHub Apps, and third-party integrations |
| `apps` | Custom internal tools and micro-apps |
| `talks` | Team discussion boards |
| `releases` | Version tagging and changelog management |
| `environments` | Dev/staging/prod environment configuration |
| `insights` | Analytics, burndown charts, and project metrics |

Projects also carry a list of linked source code repositories (`repositories`), supporting multiple providers (GitHub, GitLab, Bitbucket, plain Git, SVN, Mercurial).

---

## `cr1t` CLI

`cr1t` is a gitops-style CLI — the Rust equivalent of `kubectl` for Critical. It uses the same public API as the frontend, making Critical fully scriptable and automation-friendly.

Auth is stored in `~/.cr1tical/context.yaml` (like `kubeconfig`) — multiple server contexts are supported with `cr1t context use <name>`.

### Implemented

```sh
cr1t login                          # authenticate and store JWT
cr1t context list                   # show available contexts
cr1t context use <name>             # switch active context

cr1t get users                      # list users
cr1t get groups                     # list groups
```

### Planned

**Get resources** (brief, paginated):

```sh
cr1t get projects --limit 4
cr1t get tasks -l priority=high
cr1t get bugs --field-selector state=Open
```

Output modes: `-o table` (default) | `-o json` | `-o yaml` | `-o wide` | `-o name`

**Describe** (full resource with history and events):

```sh
cr1t describe user u_alice
cr1t describe group g_engineering
```

Returns: kind, desired state, computed state, recent events, change history.

**Apply** (GitOps mode — declarative create/update):

```sh
cr1t apply -f group.yaml
cr1t apply -k ./team-setup/
```

Supports: single files, directory trees, stdin.

**Diff** (intent vs. actual state):

```sh
cr1t diff -f group.yaml
```

**Edit** (open `$EDITOR`, patch resource):

```sh
cr1t edit group g_engineering
```

**Patch** (inline partial update):

```sh
cr1t patch group g_engineering --type merge -p '{"description":"updated"}'
```

Patch types: merge | json | strategic

**Delete** (soft-delete, history preserved):

```sh
cr1t delete user u_alice
cr1t delete -f old_sprint/
```

**Watch** (live updates via WebSocket):

```sh
cr1t get tasks -w
cr1t watch deployments
```

**Logs** (for stateful automation resources):

```sh
cr1t logs run build-frontend-123
cr1t logs deployment api-prod
```

**Snapshot** (offline / airgapped mode — least priority):

```sh
cr1t snapshot export myproject
cr1t snapshot import myproject
```

---

## Tagline

**Critical is Kubernetes for the software development lifecycle.**
