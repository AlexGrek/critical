# Project Contents & Namespacing — Implementation Whitepaper

> How project-scoped resources, ACL inheritance, and permission resolution should work in Critical.

---

## TL;DR

- **Namespacing**: Add a `project: String` field to every project-scoped resource. One shared collection per resource kind (e.g., one `tasks` collection for all projects), not one collection per project. Filter by `project` field + persistent index.
- **ACL inheritance**: Override model — if a resource has a non-empty `acl.list`, use it; otherwise fall back to the project's ACL. Super-permissions remain the final fallback.
- **Performance**: Resolve the user's principal list once per HTTP request via the existing graph traversal. Pass it into all permission checks. No materialization, no permission flattening. Optionally add per-session caching later.
- **Key ArangoDB additions**: persistent index on `["project"]` for each scoped collection, `OPTIONS { uniqueVertices: "global", order: "bfs" }` on membership traversals.

---

## Table of Contents

1. [The Problem](#1-the-problem)
2. [Namespacing: How Resources Belong to Projects](#2-namespacing-how-resources-belong-to-projects)
3. [ACL Inheritance: Project → Resource](#3-acl-inheritance-project--resource)
4. [Permission Resolution Flow](#4-permission-resolution-flow)
5. [Performance & Caching](#5-performance--caching)
6. [ArangoDB-Specific Details](#6-arangodb-specific-details)
7. [API Design](#7-api-design)
8. [Impact on Existing Code](#8-impact-on-existing-code)
9. [Open Questions](#9-open-questions)
10. [Preference Notes](#10-preference-notes)

---

## 1. The Problem

We need a two-level hierarchy: **Project → Resources**. Resources like tasks, pipelines, releases, deployments, secrets, wikis, integrations, and talks are children of a project. Each project's resources are completely independent from other projects' resources — like namespaces in Kubernetes or schemas in a multi-tenant database.

Requirements:
- Two levels only (project → resource), no deeper nesting
- Resources inherit ACLs from their project unless they override
- Different users can have different access to different resources *within* the same project (e.g., QA sees QA tickets, devs see dev tickets)
- Permission checks happen on every API request, often multiple times per request
- Must work efficiently with ArangoDB graph traversal for group membership resolution (groups can contain groups, up to 10 levels)

---

## 2. Namespacing: How Resources Belong to Projects

### Variant A: `project` field on documents (PREFERRED)

Every project-scoped resource gets a `project: String` field pointing to the project's `_key`:

```rust
#[crit_derive::crit_resource(collection = "tasks", prefix = "t_")]
pub struct Task {
    pub project: String,       // "api-v2"
    #[brief]
    pub title: String,
    pub task_group: String,    // "BACK", "FRNT", "QA"
    // ...
}
```

ArangoDB document:
```json
{
  "_key": "t_BACK-3512",
  "project": "api-v2",
  "meta": { ... },
  "acl": { "list": [], "last_mod_date": "..." },
  "title": "Fix login regression",
  "task_group": "BACK"
}
```

A persistent index on `["project"]` (or `["project", "deletion"]`) makes project-scoped queries fast:

```aql
FOR doc IN tasks
    FILTER doc.project == @project
    FILTER doc.deletion == null
    SORT doc._key ASC
    RETURN doc
```

**Pros:**
- Simplest implementation — one field, one index, one filter
- Works with your existing `generic_list` pattern — just add an optional project parameter
- Single collection per resource kind = fewer collections, simpler maintenance
- ArangoDB explicitly recommends fewer large collections over many small ones
- Cross-project queries are trivial if ever needed (admin dashboards, global search)

**Cons:**
- Every query MUST include the project filter — a missing filter = data leak between projects
- No physical isolation between projects (a corrupted index affects all projects)

**Mitigation for the data-leak risk:** Enforce the project filter at the DB layer. Add a `generic_list_scoped(collection, project, ...)` function that always includes the project filter. The `#[crit_resource]` macro could inject `project` automatically for scoped resources (similar to how it injects `acl`).

---

### Variant B: Prefix-based composite keys

Encode the project into the `_key`: `_key: "api-v2::t_BACK-3512"`.

ArangoDB's primary index is a sorted RocksDB key, so range scans work:
```aql
FOR doc IN tasks
    FILTER doc._key >= "api-v2::" AND doc._key < "api-v2:;\xff"
    RETURN doc
```

**Pros:**
- No additional index needed (uses primary key range scan)
- The key alone tells you which project a resource belongs to

**Cons:**
- Ugly, long keys that leak into URLs: `GET /v1/projects/api-v2/tasks/api-v2::t_BACK-3512`
- The `::` separator needs escaping if project IDs can contain `::`
- Renaming a project (moving resources) requires rewriting all `_key` values (ArangoDB `_key` is immutable after insert — requires delete + re-insert)
- Your `#[crit_resource]` macro's `prefix` parameter would need reworking
- Cross-project queries are awkward

---

### Variant C: Separate collections per project

Create `tasks_api_v2`, `tasks_mobile_app`, etc.

**Pros:**
- Physical isolation — a bug in one project's queries can't leak data from another
- Clean `_key` values

**Cons:**
- ArangoDB explicitly advises against this: "ArangoDB scales well on fewer huge collections, not a huge number of small ones"
- With 50 projects × 10 resource kinds = 500 collections, each with its own memory overhead (indexes, WAL, file handles)
- Dynamic collection creation on project creation — fragile
- Your `generic_list` and all AQL queries need dynamic collection names
- Cross-project queries require `UNION` across all project collections
- Breaks the `#[crit_resource]` macro's `collection_name()` return value
- Transaction coordination across many collections gets complicated

---

### Variant D: Edge collection linking projects to resources

Create a `project_owns` edge collection:
```json
{ "_from": "projects/api-v2", "_to": "tasks/t_BACK-3512" }
```

Query: `FOR r IN 1..1 OUTBOUND "projects/api-v2" project_owns RETURN r`

**Pros:**
- Clean graph model — ownership is an explicit relationship
- Supports multi-project ownership if ever needed
- Consistent with the existing `memberships` edge pattern

**Cons:**
- Extra write per resource creation (document + edge)
- Graph traversal for a simple "list tasks in project" is overkill (a field filter is faster)
- Edge collection grows to be as large as all project-scoped resources combined
- Deleting a project requires cascading edge removal
- No benefit over a simple `project` field for a strict two-level hierarchy

---

### Comparison Table

| Criterion | A: `project` field | B: Prefix keys | C: Separate collections | D: Edge collection |
|-----------|:-:|:-:|:-:|:-:|
| Implementation complexity | Low | Medium | High | Medium |
| Query simplicity | High | Medium | Low | Medium |
| Read performance | High (indexed) | High (pk range) | High (isolated) | Medium (traversal) |
| Write performance | High | High | Medium (dynamic DDL) | Medium (2 writes) |
| ArangoDB recommendation | Yes | Neutral | No | Neutral |
| Cross-project queries | Easy | Awkward | Very hard | Easy |
| Data isolation | Logical | Logical | Physical | Logical |
| Fits existing patterns | Yes | Partial | No | Partial |
| Key aesthetics | Clean | Ugly | Clean | Clean |

---

## 3. ACL Inheritance: Project → Resource

The current system has per-document ACLs (`AccessControlStore`) on groups, service accounts, pipeline accounts, and projects. We need to extend this to project-scoped resources with **inheritance from the project**.

### Variant 1: Override model (PREFERRED)

**Rule:** If a resource's `acl.list` is non-empty, it governs access completely. If it's empty, fall back to the project's ACL.

```
Resource has acl.list entries?
  ├── YES → use resource's own ACL
  └── NO  → use the parent project's ACL
              └── Still no match? → check super-permissions
```

This is what Kubernetes does: namespace-level RBAC and resource-level RBAC are separate. If you set a RoleBinding on a specific resource, it applies; otherwise, namespace-level bindings apply.

**Pros:**
- Simple mental model: "this resource has its own rules" or "this resource follows the project's rules"
- Easy to implement: `let effective_acl = if resource.acl.list.is_empty() { &project.acl } else { &resource.acl }`
- Easy to debug: you look at one ACL, not a merged result
- No "which ACL entry won?" confusion

**Cons:**
- When overriding, you must replicate any project-level entries you still want (e.g., admins)
- A resource that overrides ACL but forgets to include the project admin = admin locked out

**Mitigation:** The project admin can always be re-added via super-permissions or by a `prepare_create` hook that auto-injects the project's admin principals.

---

### Variant 2: Merge model (union of project + resource ACLs)

**Rule:** The effective permissions are the union of the project's ACL and the resource's ACL. A principal gets the highest permission level from either source.

```
effective_permissions(user) = max(project_acl_permissions(user), resource_acl_permissions(user))
```

**Pros:**
- Project admins always retain their access (can't be locked out by a resource override)
- No need to duplicate project-level entries in resource ACLs
- Users see "more" access, never "less" than the project grants

**Cons:**
- Cannot restrict access at the resource level below what the project grants. If the project gives `g_engineering` READ, every resource in the project is readable by engineering — even if you want some resources to be hidden from them.
- This is a dealbreaker for the stated requirement: "QAs can only access QA tickets, devs can only access dev tickets"
- More complex to reason about: "why can this user see this?" requires checking two ACLs
- More complex to implement: merge logic, conflict resolution

---

### Variant 3: Deny-override model (resource can restrict project ACL)

**Rule:** Like the merge model, but resource ACLs can also contain **deny entries** that subtract permissions granted by the project.

```json
{
  "acl": {
    "list": [
      { "permissions": 7, "principals": ["g_engineering"], "effect": "allow" },
      { "permissions": 7, "principals": ["g_qa"], "effect": "deny" }
    ]
  }
}
```

**Pros:**
- Maximum flexibility: project grants broad access, resources can narrow it
- Familiar pattern (AWS IAM, etc.)

**Cons:**
- Significantly more complex to implement and reason about
- Deny + allow ordering semantics are notoriously confusing
- Not compatible with your current `AccessControlStore` structure (which has no `effect` field)
- Overkill for a two-level hierarchy

---

### Variant 4: Scoped ACL (project ACL with service/group scoping)

Instead of per-resource ACLs, define access at the **project level** with scoping by service type and/or resource group:

```json
{
  "acl": {
    "list": [
      { "permissions": 31, "principals": ["g_engineering"], "scope": "*" },
      { "permissions": 7,  "principals": ["g_qa"], "scope": "tasks:QA-*" },
      { "permissions": 7,  "principals": ["g_qa"], "scope": "tasks:GENRL-*" },
      { "permissions": 31, "principals": ["g_devops"], "scope": "deployments:*" },
      { "permissions": 31, "principals": ["g_devops"], "scope": "secrets:*" }
    ]
  }
}
```

**Pros:**
- All permission config lives in one place (the project document)
- Easy to audit: "show me all access rules for this project"
- Supports the "release-managers can access deployments+secrets, QAs can access QA tickets" pattern without per-resource ACLs

**Cons:**
- Scope matching logic is complex (glob patterns? regex? exact match?)
- Moves away from per-document ACL model (your existing `AccessControlStore` on resources)
- Harder to do fine-grained per-ticket overrides (one specific ticket needs special access)
- The project document could become very large if there are many scoped rules

---

### Hybrid: Override + Scoped Project ACL (WORTH CONSIDERING)

Combine variants 1 and 4:

- **Project ACL** uses scoped entries: `{ permissions, principals, scope }` where `scope` is a service kind (e.g., `"tasks"`, `"deployments"`, `"secrets"`, or `"*"` for all).
- **Resource ACL** is either empty (inherit from project, filtered by service kind) or non-empty (override completely).

This way:
- Project-level rules like "devops can access all deployments" are set once on the project
- Task groups (QA, BACK, FRNT) inherit from project's task-scoped ACL by default
- A specific task group that needs different rules overrides at the resource level

The `AccessControlList` struct would get an optional `scope` field:

```rust
pub struct AccessControlList {
    pub permissions: Permissions,
    pub principals: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,  // e.g. "tasks", "deployments", "*"
}
```

Inheritance resolution:
```
1. Resource has non-empty acl.list? → use it
2. Otherwise, find project ACL entries where scope matches the resource's kind (or scope == "*")
3. Still no match? → check super-permissions
```

**Pros:**
- Best of both worlds: project-level scoped defaults + per-resource overrides
- One project ACL update can change access for an entire service kind
- Per-resource override when needed for edge cases
- `scope` field is backward-compatible (existing ACL entries without scope behave as `"*"`)

**Cons:**
- Slightly more complex resolution logic
- The `scope` field adds a new concept to the ACL model
- Need to decide: does `scope` filter on service kind only, or also on resource sub-groups (e.g., `"tasks:QA"`)? If the latter, the matching logic gets more complex.

---

### Comparison Table

| Criterion | 1: Override | 2: Merge | 3: Deny-override | 4: Scoped | Hybrid (1+4) |
|-----------|:-:|:-:|:-:|:-:|:-:|
| Can restrict below project level | Yes | No | Yes | N/A (centralized) | Yes |
| Project admins can't be locked out | No* | Yes | Depends | Yes | Depends |
| Per-resource fine-grained control | Yes | Yes | Yes | No | Yes |
| Implementation complexity | Low | Medium | High | Medium | Medium |
| Audibility ("who can access what") | Medium | Hard | Very hard | Easy | Medium |
| Fits current `AccessControlStore` | Yes | Yes | No (needs `effect`) | Needs `scope` | Needs `scope` |
| "Release-managers for deployments" | Manual per resource | No | Yes | Yes | Yes |
| "QA sees only QA tickets" | Yes (override on ticket group) | No | Yes | Scoped rule | Both ways |

\* Mitigated by `prepare_create` hook or super-permissions.

---

## 4. Permission Resolution Flow

Regardless of which ACL inheritance model is chosen, the resolution flow is:

```
┌──────────────────────────────────────────────────────────┐
│ 1. RESOLVE USER PRINCIPALS                               │
│    get_user_principals(user_id)                          │
│    → [user_id, group1, group2, nested_group3, ...]       │
│    (ArangoDB graph traversal: 1..10 OUTBOUND memberships)│
│    Cache this for the entire HTTP request.                │
└──────────────────────┬───────────────────────────────────┘
                       │
┌──────────────────────▼───────────────────────────────────┐
│ 2. CHECK SUPER-PERMISSIONS (short-circuit)               │
│    has_any_super_permission(principals, kind)?            │
│    → If yes, grant immediately                           │
│    (e.g., ADM_USER_MANAGER can access everything)        │
└──────────────────────┬───────────────────────────────────┘
                       │ (no super-permission match)
┌──────────────────────▼───────────────────────────────────┐
│ 3. DETERMINE EFFECTIVE ACL                               │
│                                                          │
│  if resource.acl.list is non-empty:                      │
│      effective_acl = resource.acl                        │
│  else if resource.project is set:                        │
│      project = DOCUMENT("projects", resource.project)    │
│      effective_acl = project.acl (optionally filtered    │
│                      by scope if using hybrid model)     │
│  else:                                                   │
│      deny                                                │
└──────────────────────┬───────────────────────────────────┘
                       │
┌──────────────────────▼───────────────────────────────────┐
│ 4. CHECK PERMISSION                                      │
│    effective_acl.check_permission(principals, required)   │
│    → bool                                                │
└──────────────────────────────────────────────────────────┘
```

### Single AQL Query (compute-on-the-fly)

For list queries, the entire flow can be expressed as one AQL query:

```aql
LET user_principals = UNION_DISTINCT(
    [@user],
    (FOR v IN 1..10 OUTBOUND CONCAT("users/", @user) memberships
        OPTIONS { uniqueVertices: "global", order: "bfs" }
        RETURN v._key)
)

// Load project for ACL fallback
LET project = DOCUMENT("projects", @project)

FOR doc IN @@collection
    FILTER doc.project == @project
    FILTER doc.deletion == null

    // Determine effective ACL
    LET has_own_acl = LENGTH(doc.acl.list || []) > 0
    LET effective_acl = has_own_acl ? doc.acl.list : (project.acl.list || [])

    // Check permission
    LET permitted = (
        FOR entry IN effective_acl
            FILTER BIT_AND(entry.permissions, @required_perm) == @required_perm
            FILTER LENGTH(INTERSECTION(entry.principals, user_principals)) > 0
            LIMIT 1
            RETURN true
    )
    FILTER LENGTH(permitted) > 0

    SORT doc._key ASC
    LIMIT @limit
    RETURN KEEP(doc, @fields)
```

This query:
1. Resolves principals via graph traversal (once)
2. Loads the project document (once)
3. Filters resources by project
4. For each resource, determines effective ACL (own or inherited)
5. Checks if any principal has the required permission
6. Returns only accessible resources

**Alternative: Two-query approach** (resolve principals in Rust, then run filtered AQL):
1. Rust: `let principals = db.get_user_principals(user_id).await?;`
2. Rust: `let items = db.list_scoped_with_acl(collection, project, &principals, perm).await?;`

The two-query approach is cleaner code-wise and lets you cache `principals` across multiple calls in the same request.

---

## 5. Performance & Caching

### Current Cost Analysis

Each permission check currently costs:
1. **Principal resolution**: 1 AQL graph traversal (1..10 hops on `memberships`)
2. **Super-permission check**: 1 AQL query (loads `permissions` doc + graph traversal — duplicates step 1!)
3. **Document ACL check**: In-memory after document is loaded

For a list of 100 resources, the current code runs `can_read()` 100 times, each doing 2 AQL queries = **200 AQL queries per list request**. This is extremely inefficient.

### Proposed Optimization

**Per-request principal caching:**

Resolve `user_principals` once at the start of the request and pass it into all controller methods:

```rust
// In the request handler:
let principals = state.db.get_user_principals(&user_id).await?;

// Pass to all can_read/can_write calls
ctrl.can_read(&user_id, &principals, Some(&doc)).await?
```

This reduces the cost to:
- 1 graph traversal per request (for principal resolution)
- 1 document read per super-permission check (or batch-check all relevant permissions at once)
- Per-document ACL checks are in-memory (free)

**Single AQL list query with inline ACL filtering** (as shown in section 4) reduces the entire list operation to **1 AQL query**, regardless of result count.

### Caching Tiers

| Tier | Scope | TTL | Staleness risk | Implementation |
|------|-------|-----|----------------|----------------|
| Per-request | Single HTTP request | ~milliseconds | None | Pass `principals: Vec<String>` through handler |
| Per-session | User session | 30-60s | Low (group membership changes delayed) | `DashMap<UserId, (Vec<String>, Instant)>` in `AppState` |
| Materialized | Database | Until invalidated | Medium (requires trigger system) | Separate `user_permissions` collection |

**Recommendation:** Start with per-request caching. It's zero-risk and eliminates the most egregious waste. Add per-session caching later if profiling shows principal resolution is a bottleneck. Skip materialization — it adds write amplification and ArangoDB has no built-in change-data-capture.

---

## 6. ArangoDB-Specific Details

### Required Indexes

For each project-scoped collection (e.g., `tasks`, `pipelines`, `deployments`, `secrets`, `wikis`, `releases`, `environments`, `integrations`, `talks`):

```javascript
db.<collection>.ensureIndex({
    type: "persistent",
    fields: ["project", "deletion"],
    name: "idx_project_deletion"
});
```

This allows the optimizer to push both filters (`project == @project AND deletion == null`) into the index scan.

### Graph Traversal Improvement

Your current `get_user_principals()` and `has_permission()` queries should use traversal options:

```aql
FOR v IN 1..10 OUTBOUND CONCAT("users/", @user) memberships
    OPTIONS { uniqueVertices: "global", order: "bfs" }
    PRUNE v.deletion != null
    FILTER v.deletion == null
    RETURN v._key
```

- `uniqueVertices: "global"` — prevents revisiting vertices via different paths. **Critical for graphs with cycles** (group A in group B, group B in group A).
- `order: "bfs"` — required with `uniqueVertices: "global"`. More cache-friendly for shallow-but-wide graphs.
- `PRUNE v.deletion != null` — stops traversing into soft-deleted groups (avoids unnecessary edge lookups beyond deleted nodes).

### Why Not ArangoSearch Views?

ArangoSearch could index ACL principal arrays across collections, enabling queries like "find all resources user X can access across all kinds." However:
- ArangoSearch views have **eventual consistency** (writes are visible after a short delay, not immediately)
- For permission checks, we need **immediate consistency** — a just-granted permission must work on the next request
- ArangoSearch adds operational complexity (view definitions, synchronization lag monitoring)

**Verdict:** Not needed now. Revisit when implementing cross-project global search.

### BIT_AND for Permission Checks in AQL

AQL has a `BIT_AND()` function that works with your permission bitflags:

```aql
// Check if entry has at least READ (0x07) permission
FILTER BIT_AND(entry.permissions, 7) == 7

// Check if entry has MODIFY (0x10) permission
FILTER BIT_AND(entry.permissions, 16) == 16
```

This lets you push permission checking into AQL instead of doing it in Rust per-document.

---

## 7. API Design

### URL Structure

```
# Global resources (existing)
GET    /api/v1/global/users
GET    /api/v1/global/groups
GET    /api/v1/global/projects

# Project-scoped resources (new)
GET    /api/v1/projects/{project}/tasks
GET    /api/v1/projects/{project}/tasks/{id}
POST   /api/v1/projects/{project}/tasks
PUT    /api/v1/projects/{project}/tasks/{id}
DELETE /api/v1/projects/{project}/tasks/{id}

GET    /api/v1/projects/{project}/pipelines
GET    /api/v1/projects/{project}/deployments
GET    /api/v1/projects/{project}/secrets
GET    /api/v1/projects/{project}/wikis
GET    /api/v1/projects/{project}/releases
GET    /api/v1/projects/{project}/environments
GET    /api/v1/projects/{project}/integrations
GET    /api/v1/projects/{project}/talks
GET    /api/v1/projects/{project}/insights
```

### CLI

```sh
cr1t get tasks -p api-v2
cr1t get tasks -p api-v2 --task-group QA
cr1t describe task t_QA-345 -p api-v2
cr1t apply -f task.yaml -p api-v2
cr1t get deployments -p mobile-app
cr1t get secrets -p mobile-app
```

### Alternative: Flat URLs with query parameter

```
GET /api/v1/global/tasks?project=api-v2
```

This keeps the existing `/v1/global/{kind}` pattern but adds a required `project` query parameter for scoped kinds. Simpler routing but less RESTful.

---

## 8. Impact on Existing Code

### Shared (`shared/`)

- Add a `scoped: bool` flag to `#[crit_resource]` macro (or a separate `#[crit_scoped_resource]` macro) that injects a `project: String` field
- Or: manually add `project: String` to each scoped resource struct (simpler, no macro changes)
- Modify `AccessControlList` to optionally carry a `scope` field (if using hybrid ACL model)

### Backend (`backend/`)

- **DB layer**: Add `generic_list_scoped()` and `generic_get_scoped()` that include `FILTER doc.project == @project`
- **Controllers**: `KindController` trait gets a `is_scoped() -> bool` method. Scoped controllers include project-aware permission resolution.
- **Routing**: Add `/v1/projects/{project}/{kind}` routes alongside existing `/v1/global/{kind}`
- **Middleware**: Extract `project` from path, verify project exists, inject into request extensions
- **Permission checks**: Refactor `can_read`/`can_write` to accept `&principals` (pre-resolved) and resolve effective ACL with inheritance

### Frontend (`frontend/`)

- Project detail page with tabs based on `enabled_services`
- Each tab route: `/projects/{id}/tasks`, `/projects/{id}/pipelines`, etc.
- API calls include project in the URL path

### CLI (`cli/`)

- `-p / --project` flag on all resource commands
- Context can optionally set a default project

### Database

- New collections for each resource kind (tasks, pipelines, etc.) — created on startup
- Persistent indexes on `["project", "deletion"]` for each scoped collection
- No changes to existing collections (users, groups, memberships, permissions, etc.)

---

## 9. Open Questions

1. **Should `enabled_services` be enforced at the API level?** If a project doesn't have `tasks` enabled, should `POST /projects/{id}/tasks` return 403? Or just hide the tab in the UI?

2. **Task group IDs** (QA, BACK, FRNT) — are these a first-class resource within a project, or just a string field on tasks? If first-class, they need their own ACL (which enables "QA can only see QA tickets"). If just a field, ACL must be at the individual task level.

3. **Should the `project` field on scoped resources be mutable?** (Moving a task between projects.) If yes, we need to handle ACL re-evaluation. If no, simpler.

4. **Default project ACL on creation**: Should the project creator automatically get ROOT? (Like groups do now.) Should there be a configurable template?

5. **Cross-project references**: Can a user be assigned to a task in a project they don't have access to? (e.g., a manager assigns a task to someone who can't see the project.) If yes, the assignment doesn't imply read access.

---

## 10. Preference Notes

These are my leanings — not conclusions. Your call.

**Namespacing: Strongly prefer Variant A** (`project` field). It's the simplest, most ArangoDB-idiomatic, and most maintainable option. The alternatives have real drawbacks with no proportional benefits for a two-level hierarchy.

**ACL inheritance: Lean toward the Hybrid model** (Override + Scoped project ACL). Pure override (Variant 1) is simpler to implement but forces you to copy project-level rules into every resource that needs different access for just one group. The hybrid model lets you set "devops owns deployments+secrets" once on the project and only override at the resource level when truly needed. The `scope` field on `AccessControlList` is a small addition with a large expressiveness gain.

That said, **pure override (Variant 1) is also fine** if you want to keep things as simple as possible for v1 and add scoping later. The override model is strictly a subset of the hybrid model — you can add `scope` later without breaking anything.

**Merge model: Dislike.** It can't restrict access below the project level, which is a stated requirement.

**Deny-override: Dislike.** AWS IAM-style deny logic is powerful but notoriously hard to debug. Not worth the complexity for a two-level hierarchy.

**Permission resolution: Strongly prefer per-request principal caching + single AQL queries.** The current approach of running 200 AQL queries per list request is the first thing to fix, regardless of namespacing decisions.

**API structure: Lean toward `/v1/projects/{project}/{kind}`** over flat URLs. It's more RESTful, makes the hierarchy explicit in the URL, and maps cleanly to the CLI `-p` flag. The router can share most logic with the existing `/v1/global/{kind}` routes.

---

*Last updated: 2026-02-24*
