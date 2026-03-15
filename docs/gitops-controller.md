# Gitops Controller — Handler Lifecycle & Response Contracts

This document describes how the generic gitops API handlers work internally, the
response shapes each operation returns, and how to implement `KindController`
correctly for new resource kinds.

---

## Handler Lifecycle

### POST `/v1/global/{kind}` — Create

```
1. Auth check  → ctrl.can_create(user_id, &body)
2. ctrl.prepare_create(&mut body, user_id)      ← inject defaults (state, labels, annotations)
3. db.ensure_collection(kind)
4. ctrl.to_internal(body, &auth)                ← rename id→_key, strip unknown fields, etc.
5. compute_value_hash(&doc) → inject hash_code
6. ctrl.validate_acl_principals(&doc, db)
7. let doc_snapshot = doc.clone()               ← snapshot BEFORE db call
8. db.generic_create(kind, doc)
9. ctrl.after_create(final_id, user_id, db)
10. db.write_history_entry(kind, final_id, doc_snapshot.clone(), user_id)
11. events.entity_lifecycle(Lifecycle, "created")
12. Return 201 ctrl.to_external(doc_snapshot)
```

**Key invariant**: the response is built from `doc_snapshot` — the fully-prepared
internal document cloned before it was moved into the DB write. No extra GET is
issued. This means the response always reflects exactly what was written.

### PUT `/v1/global/{kind}/{id}` — Update

```
1. Fetch existing from DB (404 if missing)
2. Optional: validate client hash_code matches server hash (OCC)
3. Auth check  → ctrl.can_write(user_id, Some(&existing))
4. ctrl.to_internal(body, &auth)
5. compute_value_hash → inject hash_code
6. ctrl.validate_acl_principals(&doc, db)
7. let doc_snapshot = doc.clone()
8. db.generic_update(kind, id, doc)
9. ctrl.after_update(id, db)
10. db.write_history_entry(kind, id, doc_snapshot.clone(), user_id)
11. events.entity_lifecycle(Note, "updated")
12. Return 200 ctrl.to_external(doc_snapshot)
```

**Note**: `generic_update` does an ArangoDB `UPDATE existing WITH @doc` (merge,
not replace). The `doc_snapshot` contains only the fields that the caller sent
(plus computed fields like `hash_code`). Fields that exist on the stored document
but were not in the PUT body remain in the DB but are **not** reflected in the
response. For a correct round-trip, callers should send all fields they want to
preserve.

### POST `/v1/global/{kind}/{id}` — Upsert

Same lifecycle as create/update but selects the branch dynamically. Response is
`200 { "id": ... }` only (no full doc returned — the upsert endpoint is intended
for idempotent machine-driven apply workflows where callers already know the shape).

### GET `/v1/global/{kind}/{id}` — Fetch

```
1. db.generic_get(kind, id)  → 404 if missing/soft-deleted
2. ACL check → ctrl.can_read(user_id, Some(&doc))  (404 on denial)
3. Optional: attach history snapshot if ?with_history=true
4. Return 200 ctrl.to_external(doc)
```

### GET `/v1/global/{kind}` — List

```
1. Resolve principals, check super-permission bypass
2. db.generic_list_acl(kind, principals, bits, super_bypass,
                        ctrl.list_projection_fields(), limit, cursor)
3. items.map(|doc| ctrl.to_list_external(doc))
4. Return 200 { "items": [...] }
```

Brief field control is two-layered:
- **`list_projection_fields()`** — AQL-level `KEEP()` projection (reduces network
  payload from DB); return `None` to fetch all fields.
- **`to_list_external()`** — post-processing Rust filter; default delegates to
  `to_external()`.

Use both for maximum efficiency, or just `to_list_external()` for simplicity.

### DELETE `/v1/global/{kind}/{id}` — Soft Delete

```
1. db.generic_get(kind, id)  → 404 if missing
2. ACL check → ctrl.can_write(user_id, Some(&existing))
3. db.generic_soft_delete(kind, id, user_id)
4. ctrl.after_delete(id, db)
5. events.entity_lifecycle(Lifecycle, "deleted")
6. Return 204 No Content
```

---

## Response Shapes

### Create (POST) and Update (PUT)

Both return the **full document** as produced by `ctrl.to_external(doc_snapshot)`:

```json
{
  "id": "my-resource",
  "hash_code": "abc123...",
  "state": {
    "created_at": "2026-01-01T00:00:00Z",
    "created_by": "u_alice",
    "updated_at": "2026-01-01T00:00:00Z"
  },
  "labels": {},
  "annotations": {},
  "acl": { "list": [], "last_mod_date": "..." }
  // ... resource-specific fields
}
```

Callers can read `hash_code` from the create/update response directly — no
follow-up GET is needed for OCC (optimistic concurrency control).

### List (GET without `?limit`)

```json
{ "items": [ /* brief objects via to_list_external() */ ] }
```

Brief objects must **not** include `hash_code`, `annotations`, or other full-only
fields. Enforce this with `to_list_external()`.

### Fetch (GET `/{id}`)

Full document identical in shape to the create/update response.

---

## Implementing `to_external` and `to_list_external`

### `to_external` — used for single-resource responses

Always call `standard_to_external(doc)` which:
- Renames `_key` → `id`
- Strips `_id` and `_rev` (ArangoDB internal fields)
- Ensures `labels` defaults to `{}`

```rust
fn to_external(&self, doc: Value) -> Value {
    standard_to_external(doc)
}
```

### `to_list_external` — used for list items

The default delegates to `to_external()`, returning the full document. Override
to produce a brief view:

```rust
fn to_list_external(&self, doc: Value) -> Value {
    let mut doc = standard_to_external(doc);
    if let Some(obj) = doc.as_object_mut() {
        obj.remove("hash_code");
        obj.remove("annotations");
        // remove any other full-only fields
    }
    doc
}
```

Or use `filter_to_brief(doc, &["id", "labels", "name", "state"])` to keep only
an explicit allowlist of fields.

---

## The `doc_snapshot` Pattern

Before the fix introduced in March 2026, create and update handlers issued a
second `generic_get` after the write to build the response. This was fragile:

- `generic_get` errors were silently swallowed (`.ok().flatten()`)
- If the collection wasn't reachable at the moment of the second read, callers
  received just `{"id": "..."}` with no other fields
- Extra DB round-trip on every write

The current approach clones the fully-prepared internal document before moving it
into `generic_create`/`generic_update`:

```rust
let doc_snapshot = doc.clone();
state.db.generic_create(&kind, doc).await?;
// ...
Ok(ctrl.to_external(doc_snapshot))
```

**Caveat**: the snapshot reflects only the fields that were sent in the request
body (plus computed fields). For `PUT`, fields that exist on the stored document
but were omitted from the request are not included in the response (they remain
in the DB). For a full view after a PUT, issue a separate GET.

---

## ACL Denial Conventions

- All ACL denials return **404** (not 403) to avoid leaking resource existence
- Godmode users (`ADM_GODMODE` super-permission) bypass all `can_read`/`can_write`
  checks but still hit the same handler code
- `super_permission()` returning `Some(perm)` short-circuits list-level ACL
  filtering for users who hold that permission

---

## Adding a New Resource Kind

1. `backend/src/controllers/{kind}_controller.rs` — create struct + `impl KindController`
2. Add to `Controller` struct in `controllers/mod.rs`
3. Add match arm in `Controller::for_kind()`
4. No changes needed in route handlers

See [architecture.md](architecture.md) for the full checklist.
