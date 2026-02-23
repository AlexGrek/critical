# DATABASE.md — Critical ArangoDB Schema Reference

This document describes every collection in the Critical ArangoDB database, including field definitions, edge collections, and conventions.

**Always update this file** when making schema changes (new collections, field additions/removals, new indexes, new edge collections).

---

## Conventions

| Convention | Description |
|-----------|-------------|
| `_key` | ArangoDB document key; maps to the Rust `id` field via `#[serde(rename = "_key")]` |
| ID prefixes | `u_` users · `g_` groups · `p_` projects · `t_` tasks · `sp_` sprints · `f_` features · `pl_` pipelines · `plr_` pipeline runs · `art_` artifacts · `dep_` deployments · `rel_` releases · `pg_` pages · `pol_` policies |
| `ResourceMeta` | Embedded in every document; carries `labels`, `annotations`, `created_at`, `created_by`, `updated_at`, `updated_by` |
| `AccessControlStore` | Embedded ACL: `list: [{permissions: u8, principals: [id]}]`, `last_mod_date: DateTime` |
| Soft-delete | `state: LifecycleState` — `active` / `archived` / `deleted` |
| No migration system | ArangoDB is schemaless; adding `Option<T>` or `#[serde(default)]` fields is safe. Renames require manual data fixup. |

---

## Collections

### `users` (vertex)

Stores registered user accounts.

| Field | Type | Notes |
|-------|------|-------|
| `_key` | String | User ID, e.g. `u_alice` |
| `password_hash` | String | bcrypt hash; stripped from all API responses |
| `state` | LifecycleState | `active` / `archived` / `deleted` (replaces former `deactivated: bool`) |
| `personal.name` | String | Display name |
| `personal.gender` | String | |
| `personal.job_title` | String | |
| `personal.manager` | String? | Manager user ID |
| `meta` | ResourceMeta | labels, annotations, created\_at, created\_by, updated\_at, updated\_by |

Brief fields (returned in list queries): `_key`, `state`, `personal`, `meta`

---

### `groups` (vertex)

Stores user groups. Groups can be nested (a group can be a member of another group).

| Field | Type | Notes |
|-------|------|-------|
| `_key` | String | Group ID, e.g. `g_engineering` |
| `name` | String | Display name |
| `description` | String? | Optional description |
| `acl` | AccessControlStore | Document-level ACL |
| `meta` | ResourceMeta | |

Brief fields: `_key`, `name`

---

### `memberships` (edge) ⟶ graph traversal

Edge collection connecting principals (users or groups) to groups.

| Field | Type | Notes |
|-------|------|-------|
| `_key` | String | `{principal_id}::{group_id}` |
| `_from` | String | `users/{user_id}` or `groups/{group_id}` |
| `_to` | String | `groups/{group_id}` |
| `principal` | PrincipalId | Denormalised; used in AQL filters |
| `group` | PrincipalId | Denormalised; used in AQL filters |

Used for graph traversal: `FOR v IN 1..10 OUTBOUND CONCAT("users/", @user) memberships`

---

### `permissions` (vertex)

Stores global (super) permissions. Each document key is a permission name; the document lists which principals hold it.

| Field | Type | Notes |
|-------|------|-------|
| `_key` | String | Permission name, e.g. `adm_user_manager` |
| `principals` | Vec\<PrincipalId\> | Users/groups granted this permission |

**Seeded permissions** (granted to `u_root` on startup):

| Key | Purpose |
|-----|---------|
| `adm_user_manager` | Full CRUD on users and groups |
| `adm_project_manager` | Full CRUD on projects, tasks, sprints |
| `usr_create_projects` | Create new projects |
| `usr_create_groups` | Create new groups |
| `adm_config_editor` | Edit global configuration |
| `usr_create_pipelines` | Create pipeline definitions |
| `adm_policy_editor` | Create/modify approval policies |
| `adm_release_manager` | Create deployments and releases |

---

### `projects` (vertex)

Projects act as namespaces for tasks, sprints, features, pages, etc.

| Field | Type | Notes |
|-------|------|-------|
| `_key` | String | Project ID, e.g. `p_myproject` |
| `name` | String | Display name |
| `description` | String? | |
| `acl` | AccessControlStore | |
| `state` | LifecycleState | |
| `meta` | ResourceMeta | |

Brief fields: `_key`, `name`, `state`

---

### `tasks` (vertex)

Work items (tickets). Replaces the former `Ticket` and `TicketGroup` models.

| Field | Type | Notes |
|-------|------|-------|
| `_key` | String | Task ID, e.g. `t_fix_login` |
| `title` | String | |
| `description` | String | Markdown |
| `state` | TaskState | `backlog` / `open` / `in_progress` / `in_review` / `done` / `cancelled` |
| `priority` | Priority | `low` / `medium` / `high` / `critical` |
| `severity` | (u8, String)? | Optional numeric + label severity |
| `assigned_to` | PrincipalId? | User or group |
| `mentioned` | Vec\<PrincipalId\> | |
| `acl` | AccessControlStore | |
| `meta` | ResourceMeta | |

Brief fields: `_key`, `title`, `state`, `priority`

---

### `sprints` (vertex)

Time-boxed iterations grouping tasks.

| Field | Type | Notes |
|-------|------|-------|
| `_key` | String | Sprint ID, e.g. `sp_q1_sprint1` |
| `name` | String | |
| `goal` | String? | Sprint goal statement |
| `starts_at` | DateTime? | |
| `ends_at` | DateTime? | |
| `state` | SprintState | `planning` / `active` / `completed` / `cancelled` |
| `acl` | AccessControlStore | |
| `meta` | ResourceMeta | |

Brief fields: `_key`, `name`, `state`

---

### `features` (vertex)

Epics / product requirements.

| Field | Type | Notes |
|-------|------|-------|
| `_key` | String | Feature ID, e.g. `f_dark_mode` |
| `name` | String | |
| `description` | String? | |
| `state` | TaskState | Reuses TaskState enum |
| `acl` | AccessControlStore | |
| `meta` | ResourceMeta | |

Brief fields: `_key`, `name`, `state`

---

### `pipelines` (vertex)

CI/CD pipeline definitions.

| Field | Type | Notes |
|-------|------|-------|
| `_key` | String | Pipeline ID, e.g. `pl_build_api` |
| `name` | String | |
| `repo_url` | String? | Source repository URL |
| `triggers` | Vec\<String\> | Branch patterns / tag globs |
| `acl` | AccessControlStore | |
| `meta` | ResourceMeta | |

Brief fields: `_key`, `name`

---

### `pipeline_runs` (vertex)

Individual executions of a pipeline. Immutable after reaching a terminal state.

| Field | Type | Notes |
|-------|------|-------|
| `_key` | String | Run ID, e.g. `plr_20240101_abc` |
| `pipeline_id` | String | Parent pipeline `_key` |
| `state` | RunState | `pending` / `running` / `succeeded` / `failed` / `cancelled` |
| `started_at` | DateTime | |
| `finished_at` | DateTime? | |
| `triggered_by` | PrincipalId | |
| `log_url` | String? | Link to log storage |
| `meta` | ResourceMeta | |

Brief fields: `_key`, `pipeline_id`, `state`

---

### `artifacts` (vertex)

Build outputs: Docker images, binaries, npm packages, etc.

| Field | Type | Notes |
|-------|------|-------|
| `_key` | String | Artifact ID, e.g. `art_api_v1_2_3` |
| `name` | String | Human-readable name |
| `artifact_type` | String | `docker-image`, `binary`, `npm-package`, … |
| `uri` | String | Registry/storage location |
| `digest` | String? | Content digest, e.g. `sha256:…` |
| `meta` | ResourceMeta | |

Brief fields: `_key`, `name`

---

### `deployments` (vertex)

Records of artifact deployments to environments.

| Field | Type | Notes |
|-------|------|-------|
| `_key` | String | Deployment ID, e.g. `dep_prod_20240101` |
| `env` | String | Target environment: `prod`, `staging`, `dev` |
| `state` | RunState | |
| `started_at` | DateTime | |
| `finished_at` | DateTime? | |
| `deployed_by` | PrincipalId | |
| `acl` | AccessControlStore | |
| `meta` | ResourceMeta | |

Brief fields: `_key`, `env`, `state`

---

### `releases` (vertex)

Named product releases.

| Field | Type | Notes |
|-------|------|-------|
| `_key` | String | Release ID, e.g. `rel_v2_0_0` |
| `version` | String | SemVer or custom version string |
| `changelog` | String? | Markdown changelog |
| `state` | ReleaseState | `draft` / `candidate` / `released` / `yanked` |
| `released_at` | DateTime? | |
| `acl` | AccessControlStore | |
| `meta` | ResourceMeta | |

Brief fields: `_key`, `version`, `state`

---

### `pages` (vertex)

Wiki / documentation pages (Confluence replacement).

| Field | Type | Notes |
|-------|------|-------|
| `_key` | String | Page ID, e.g. `pg_onboarding` |
| `title` | String | |
| `content` | String | Markdown content |
| `acl` | AccessControlStore | |
| `meta` | ResourceMeta | |

Brief fields: `_key`, `title`

---

### `policies` (vertex)

Approval gate policies applied to resource operations.

| Field | Type | Notes |
|-------|------|-------|
| `_key` | String | Policy ID, e.g. `pol_prod_deploy` |
| `name` | String | |
| `match_kind` | String | Resource kind the policy applies to, e.g. `deployment` |
| `match_labels` | Labels | Label selector conditions |
| `require_approvers` | Vec\<PrincipalId\> | Users/groups whose approval is required |
| `meta` | ResourceMeta | |

---

### `relations` (edge) ⟶ generic graph

Generic edge collection for all relation types other than group membership.

| Field | Type | Notes |
|-------|------|-------|
| `_key` | String | Auto-generated |
| `_from` | String | Source document, e.g. `tasks/t_abc` |
| `_to` | String | Target document, e.g. `sprints/sp_xyz` |
| `kind` | RelationKind | `belongs_to` / `implements` / `caused_by` / `deploys` / `built_from` / `triggered_by` / `references` / `custom(str)` |
| `meta` | ResourceMeta | |

---

### `revisions` (vertex)

Immutable audit records. Written by controller hooks on create/update; never deleted via API.

| Field | Type | Notes |
|-------|------|-------|
| `_key` | String | `{kind}_{resource_id}_{revision}` |
| `resource_kind` | String | e.g. `tasks` |
| `resource_id` | String | The resource `_key` |
| `revision` | u64 | Monotonically increasing per resource |
| `snapshot` | JSON | Full document at this point in time |
| `changed_by` | PrincipalId | |
| `changed_at` | DateTime | |

---

## ER Diagram

```
users ──────────── memberships (edge) ──────────── groups
  │                                                   │
  │                  permissions                      │
  │                  (global perms)                   │
  │                                                   │
  ├── tasks ── relations (edge) ── sprints            │
  │              │                    │               │
  │              ├── features         │               │
  │              ├── releases         │               │
  │              └── pipeline_runs    │               │
  │                                   │               │
  ├── projects (namespace)            │               │
  │     ├── tasks                     │               │
  │     ├── sprints                   │               │
  │     ├── features                  │               │
  │     └── pages                     │               │
  │                                   │               │
  ├── pipelines ──► pipeline_runs ──► artifacts       │
  │                                   │               │
  └── deployments ◄── artifacts       │               │
        └── releases                  │               │
                                      │               │
  policies ──────────────────────────►(match any kind)
  revisions ─────────────────────────►(audit any kind)
```

---

## Active Collections Summary

| Collection | Type | Status |
|-----------|------|--------|
| `users` | vertex | Active |
| `groups` | vertex | Active |
| `memberships` | edge | Active |
| `permissions` | vertex | Active |
| `projects` | vertex | Active |
| `tasks` | vertex | Active |
| `sprints` | vertex | Active |
| `features` | vertex | Active |
| `pipelines` | vertex | Active |
| `pipeline_runs` | vertex | Active |
| `artifacts` | vertex | Active |
| `deployments` | vertex | Active |
| `releases` | vertex | Active |
| `pages` | vertex | Active |
| `policies` | vertex | Active |
| `relations` | edge | Active |
| `revisions` | vertex | Active |

---

*Last updated: 2026-02-23*
