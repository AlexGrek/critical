---
name: itest-pytest
description: >
  Expert knowledge of this project's Python integration test suite. Use when writing,
  reviewing, or debugging pytest integration tests — fixtures, auth patterns, ACL coverage,
  pagination, debug-route tooling, or adding tests for new endpoints. Enforces project
  conventions and provides ready-to-use debugging utilities built on debug routes.
user-invocable: true
---

You are working on the **Critical (crit-cli)** Python integration test suite.
Apply the following knowledge to every test you write or review.

---

## Location & Commands

```
backend/itests/
  pyproject.toml        # PDM project, dependencies: pytest, pytest-xdist, requests, websockets
  tests/
    auth_test.py
    gitops_test.py
    gitops_permissions_test.py
    group_test.py
    pagination_test.py
    debug_test.py
```

```bash
# ALWAYS kill any stale axum-api process BEFORE running tests:
pkill -x axum-api 2>/dev/null; sleep 1   # kill stale backend if running

# ALWAYS use the Makefile — it spawns an ephemeral ArangoDB and tears it down on exit:
make test-api                           # run all Python integration tests

# Only use pdm directly when debugging specific cases that require an already-running
# backend + database with specific state (e.g. inspecting leftover data after a failure):
cd backend/itests
pdm run pytest tests/ -v               # all tests
pdm run pytest tests/group_test.py -v  # single file
pdm run pytest tests/ -k "test_creator_has_root_acl" -v  # single test
pdm run pytest tests/ -v --tb=long     # full tracebacks
```

**Default**: always use `make test-api` — it handles the full lifecycle (start DB → start backend → run tests → teardown).

**CRITICAL — stale backend problem**: `make test-api` starts a fresh backend on port 3742. If a dev backend from `make run` is already on that port, the new binary fails to start with `Address already in use`, the health check passes on the OLD process, and tests silently run against a stale binary. Symptoms:
- Debug routes return 404 (routes added after the old binary was compiled)
- Brief-field filtering is wrong (e.g. `annotations` appears in list responses)
- Other subtle mismatches between test expectations and actual behavior

**Always check and kill before running tests**:
```bash
pgrep -la axum-api          # check if stale backend is running
pkill -x axum-api           # kill it
```

**Direct `pdm run pytest` only when**: you need to inspect a specific DB state, replay a failure against a live backend, or iterate quickly on a single test while the stack is already running (`make run`).

Root credentials: `root` / `changeme`.

---

## Base URLs & Endpoint Map

```python
BASE       = "http://localhost:3742/api"
URL_REGISTER = f"{BASE}/register"
URL_LOGIN    = f"{BASE}/login"
URL_GLOBAL   = f"{BASE}/v1/global"      # generic gitops CRUD
URL_DEBUG    = f"{BASE}/v1/debug"       # godmode-only debug endpoints
URL_WS       = "ws://localhost:3742/api/v1/ws"
```

### Gitops Endpoints (JWT required)

| Method   | Path                            | Action              | Success |
|----------|---------------------------------|---------------------|---------|
| `GET`    | `/v1/global/{kind}`             | list (brief)        | 200     |
| `POST`   | `/v1/global/{kind}`             | create              | 201     |
| `GET`    | `/v1/global/{kind}/{id}`        | fetch (full)        | 200     |
| `POST`   | `/v1/global/{kind}/{id}`        | upsert              | 200     |
| `PUT`    | `/v1/global/{kind}/{id}`        | update (replace)    | 200     |
| `DELETE` | `/v1/global/{kind}/{id}`        | soft-delete         | 204     |

`{kind}` values: `users`, `groups`, `service_accounts`, `pipeline_accounts`,
`memberships`, `projects`.

### Debug Endpoints (ADM_GODMODE required)

| Method | Path                            | Returns                                              |
|--------|---------------------------------|------------------------------------------------------|
| `GET`  | `/v1/debug/collections`         | `{ "collections": [{ "name": "..." }, ...] }`        |
| `GET`  | `/v1/debug/collections/{name}`  | `{ "collection", "count", "documents": [...] }`      |

System collections (names starting with `_`) are rejected with `400`.

---

## ID Conventions (match `shared/src/data_models.rs`)

| Kind              | Prefix  | Example              |
|-------------------|---------|----------------------|
| users             | `u_`    | `u_alice`            |
| groups            | `g_`    | `g_backend_team`     |
| service_accounts  | `sa_`   | `sa_ci_runner`       |
| pipeline_accounts | `pa_`   | `pa_deploy_prod`     |
| projects          | *(none)*| `my-project`         |
| root user         | `u_`    | `u_root`             |

**Always prefix IDs yourself** when building test resources — the backend stores exactly what you send.

---

## Response Shapes

### Single resource (GET /{id}, POST create, PUT)
```json
{
  "id": "u_alice",
  "labels": {},
  "annotations": {},
  "state": { "created_at": "...", "updated_at": "...", ... },
  "acl": { "list": [...], "last_mod_date": "..." },
  "hash_code": "..."
  // ... resource-specific fields
}
```

### List (GET without ?limit)
```json
{ "items": [ /* brief objects — only #[brief] fields + id + labels */ ] }
```
No `has_more`, no `next_cursor`.

### Paginated list (GET with ?limit)
```json
{
  "items": [...],
  "has_more": true,
  "next_cursor": "opaque-cursor-string"   // absent on last page
}
```
Last page: `has_more: false`, **no** `next_cursor` key at all.

### Error
```json
{
  "error": {
    "message": "Authorization failed: Unauthorized",
    "status": 401,
    "type": "authorization_error"
  }
}
```

### ACL denial: **404**, not 403
The backend deliberately returns 404 (not 403) when ACL check fails, to avoid
leaking resource existence. Always assert `== 404` for denied writes.

---

## Permissions Bitflags (`shared/src/util_models.rs`)

```python
PERM_FETCH  = 1       # 0b00000001
PERM_LIST   = 2       # 0b00000010
PERM_NOTIFY = 4       # 0b00000100
PERM_CREATE = 8       # 0b00001000
PERM_MODIFY = 16      # 0b00010000  (includes deletion)
PERM_CUSTOM1 = 32
PERM_CUSTOM2 = 64

PERM_READ  = PERM_FETCH | PERM_LIST | PERM_NOTIFY   # 7
PERM_WRITE = PERM_CREATE | PERM_MODIFY | PERM_READ   # 31
PERM_ROOT  = PERM_WRITE | PERM_CUSTOM1 | PERM_CUSTOM2  # 127
```

Use these constants in ACL list entries:
```python
{"permissions": 127, "principals": [user_id]}   # ROOT — full access
{"permissions": 7,   "principals": [user_id]}   # READ only
{"permissions": 31,  "principals": [user_id]}   # WRITE
```

---

## Assets

You can use graphical and binary assets in tests by placing them in `backend/itests/assets/` and referencing them by relative path (e.g. for avatar upload tests). Reuse available assets where possible to avoid cluttering the repo with test-only files.

Use object_storage configuration with local filesystem backend (pointing to `backend/itests/temp/{random directory name}`) for tests that need to upload or access files. You can check the contents of the storage directory after a test failure for debugging, add directory content assertions in tests, or use the debug route utilities to inspect DB records that reference stored objects. remember to clean up any uploaded files in test teardown to avoid filling up disk over time.

---

## Super-Permissions

Stored in the `permissions` collection (key = permission name, value = `{ "principals": [...] }`).
Root user gets `adm_godmode` at server startup (idempotent).

| Constant                | Key                    | Grants                                     |
|-------------------------|------------------------|--------------------------------------------|
| `ADM_GODMODE`           | `adm_godmode`          | Full bypass of all ACLs + debug endpoints  |
| `ADM_USER_MANAGER`      | `adm_user_manager`     | Create/delete/update users; root has it    |
| `ADM_CONFIG_EDITOR`     | `adm_config_editor`    | System config changes                      |
| `USR_CREATE_GROUPS`     | `usr_create_groups`    | Regular users get this on registration     |
| `USR_CREATE_PROJECTS`   | `usr_create_projects`  | Create projects                            |

---

## Standard Fixture Patterns

### Module-scoped auth fixtures (copy-paste starting points)

```python
import random, requests

BASE = "http://localhost:3742/api"

def auth_headers(token: str) -> dict:
    return {"Authorization": f"Bearer {token}", "Content-Type": "application/json"}


@pytest.fixture(scope="module")
def admin_token():
    """Login as root — has ADM_GODMODE + ADM_USER_MANAGER."""
    resp = requests.post(f"{BASE}/login", json={"user": "root", "password": "changeme"})
    assert resp.status_code == 200, f"Root login failed: {resp.text}"
    return resp.json()["token"]


@pytest.fixture(scope="module")
def regular_user():
    """Register a fresh random user. Gets usr_create_groups by default."""
    num = random.randint(100000, 999999)
    user = f"itest_{num}"
    requests.post(f"{BASE}/register", json={"user": user, "password": user})
    resp = requests.post(f"{BASE}/login", json={"user": user, "password": user})
    assert resp.status_code == 200, f"Login failed: {resp.text}"
    return {"user_id": f"u_{user}", "token": resp.json()["token"]}
```

### Resource helper fixtures (use `yield` + teardown)

```python
@pytest.fixture(scope="module")
def test_group(admin_token):
    """Create a group for the module, clean up after."""
    num = random.randint(100000, 999999)
    gid = f"g_itest_{num}"
    headers = auth_headers(admin_token)
    resp = requests.post(
        f"{BASE}/v1/global/groups",
        json={"id": gid, "name": "Test Group",
              "acl": {"list": [{"permissions": 127, "principals": ["u_root"]}],
                      "last_mod_date": datetime.now(timezone.utc).isoformat()}},
        headers=headers,
    )
    assert resp.status_code == 201
    yield gid
    requests.delete(f"{BASE}/v1/global/groups/{gid}", headers=headers)
```

---

## Fixture Decomposition Rules

1. **`scope="module"`** for all auth tokens and shared resources — one login per file.
2. **`scope="function"`** only when tests mutate state and need isolation between tests.
3. **Random IDs** (`random.randint(100000, 999999)`) prevent collisions between parallel workers.
4. **No `conftest.py`** — each test file is self-contained with its own fixtures.
5. **yield fixtures** for resource lifecycle: create before yield, delete after.
6. **Accept 201 or 409** on register in auth fixtures — safe for parallel xdist workers.
7. **Inline cleanup** at the end of each test that creates ad-hoc resources; fixture teardown for shared resources.
8. **Use `DebugClient` in teardown and assertion messages** to inspect DB state on failures.
9. Always have root token fixture available for debug route access and admin-bypass tests. Print it so you can use it to send debug requests manually if needed.

---

## ACL Body Builder

```python
from datetime import datetime, timezone

def acl_entry(permissions: int, *principal_ids: str) -> dict:
    return {"permissions": permissions, "principals": list(principal_ids)}

def make_acl(*entries) -> dict:
    return {"list": list(entries), "last_mod_date": datetime.now(timezone.utc).isoformat()}

# Examples:
# make_acl(acl_entry(127, "u_root"))                        # root-only, full access
# make_acl(acl_entry(127, creator_id), acl_entry(7, viewer_id))  # creator+viewer
```

---

## Debug Route Utilities (use these for diagnostics)

The following class wraps debug routes into a usable inspection API.
**Include these helpers at the top of any test file that needs DB inspection.**

```python
class DebugClient:
    """HTTP wrapper around /v1/debug endpoints for test diagnostics.

    Requires a token with ADM_GODMODE (root token).
    Use dump() in pytest.fail() messages to show raw DB state on failures.
    """

    def __init__(self, root_token: str, base: str = "http://localhost:3742/api"):
        self._headers = {
            "Authorization": f"Bearer {root_token}",
            "Content-Type": "application/json",
        }
        self._base = base

    def list_collections(self) -> list[str]:
        """Return names of all non-system collections."""
        resp = requests.get(f"{self._base}/v1/debug/collections", headers=self._headers)
        assert resp.status_code == 200, f"debug/collections failed: {resp.text}"
        return [c["name"] for c in resp.json()["collections"]]

    def dump(self, collection: str) -> list[dict]:
        """Return all raw documents in a collection (as stored in ArangoDB)."""
        resp = requests.get(
            f"{self._base}/v1/debug/collections/{collection}",
            headers=self._headers,
        )
        assert resp.status_code == 200, f"debug dump of '{collection}' failed: {resp.text}"
        body = resp.json()
        return body["documents"]

    def count(self, collection: str) -> int:
        """Return document count for a collection."""
        resp = requests.get(
            f"{self._base}/v1/debug/collections/{collection}",
            headers=self._headers,
        )
        assert resp.status_code == 200, f"debug count of '{collection}' failed: {resp.text}"
        return resp.json()["count"]

    def find(self, collection: str, **field_matchers) -> list[dict]:
        """Return documents where all field_matchers match (equality).

        Example:
            debug.find("memberships", group="g_backend_123")
            debug.find("users", _key="u_alice")
        """
        docs = self.dump(collection)
        result = []
        for doc in docs:
            if all(doc.get(k) == v for k, v in field_matchers.items()):
                result.append(doc)
        return result

    def find_one(self, collection: str, **field_matchers) -> dict | None:
        """Like find() but returns first match or None."""
        matches = self.find(collection, **field_matchers)
        return matches[0] if matches else None

    def assert_exists(self, collection: str, msg: str = "", **field_matchers) -> dict:
        """Assert a document matching field_matchers exists; return it."""
        doc = self.find_one(collection, **field_matchers)
        if doc is None:
            all_docs = self.dump(collection)
            raise AssertionError(
                f"{msg or 'Expected document not found'}\n"
                f"Filters: {field_matchers}\n"
                f"Collection '{collection}' contents ({len(all_docs)} docs):\n"
                + "\n".join(f"  {d}" for d in all_docs)
            )
        return doc

    def assert_not_exists(self, collection: str, msg: str = "", **field_matchers) -> None:
        """Assert no document matching field_matchers exists."""
        doc = self.find_one(collection, **field_matchers)
        if doc is not None:
            raise AssertionError(
                f"{msg or 'Document unexpectedly found'}\n"
                f"Filters: {field_matchers}\n"
                f"Found: {doc}"
            )

    def assert_soft_deleted(self, collection: str, key: str) -> dict:
        """Assert document exists AND has a deletion marker (soft-deleted).

        Use this instead of asserting 404 from the API when you need to
        verify that soft-delete actually wrote the deletion field to the DB.
        """
        docs = self.dump(collection)
        for doc in docs:
            if doc.get("_key") == key:
                assert "deletion" in doc and doc["deletion"] is not None, (
                    f"Document {key} in '{collection}' exists but is NOT soft-deleted.\n"
                    f"Document: {doc}"
                )
                return doc
        raise AssertionError(
            f"Document {key} not found in '{collection}' at all (not even soft-deleted).\n"
            f"Keys present: {[d.get('_key') for d in docs]}"
        )

    def snapshot(self, *collections: str) -> dict[str, list[dict]]:
        """Dump multiple collections at once; useful for before/after comparisons."""
        return {col: self.dump(col) for col in collections}
```

### Debug fixture

```python
@pytest.fixture(scope="module")
def debug(admin_token) -> DebugClient:
    """DebugClient backed by root's ADM_GODMODE token."""
    return DebugClient(admin_token)
```

### Usage patterns in tests

```python
def test_soft_delete_marks_deletion_field(admin_token, debug):
    # ... create resource, then delete via API ...
    resp = requests.delete(f"{URL_GLOBAL}/users/{user_id}", headers=auth_headers(admin_token))
    assert resp.status_code == 204

    # API says 404 now — verify the DB actually has deletion marker:
    doc = debug.assert_soft_deleted("users", user_id)
    assert doc["deletion"]["deleted_by"] is not None


def test_cascade_removes_membership(admin_token, debug):
    # ... setup, then delete user ...
    debug.assert_not_exists("memberships", group=group_id,
                            msg=f"Membership for {user_id} should have been cascade-deleted")


def test_creator_acl_injected(regular_user, debug):
    # ... create group ...
    doc = debug.assert_exists("groups", _key=group_id.lstrip("g_"))
    acl_principals = [
        p for entry in doc.get("acl", {}).get("list", [])
        for p in entry.get("principals", [])
    ]
    assert regular_user["user_id"] in acl_principals, (
        f"Creator should be in ACL.\nDB state:\n{debug.dump('groups')}"
    )
```

---

## Membership Body Builder

```python
def make_membership(principal_id: str, group_id: str) -> dict:
    """Build a memberships POST body from principal and group IDs.

    Works for user→group and group→group edges.
    Assumes principal_id has its prefix (u_alice, g_child).
    """
    # Derive ArangoDB collection name from ID prefix
    prefix = principal_id.split("_")[0] if "_" in principal_id else "users"
    collection_map = {"u": "users", "g": "groups", "sa": "service_accounts", "pa": "pipeline_accounts"}
    from_col = collection_map.get(prefix, "users")
    to_col = "groups"

    key = f"{principal_id}::{group_id}"
    return {
        "id": key,
        "_from": f"{from_col}/{principal_id}",
        "_to": f"{to_col}/{group_id}",
        "principal": principal_id,
        "group": group_id,
    }
```

---

## User Body Builder

```python
def make_user(user_id: str, name: str = "", password: str = "test123") -> dict:
    return {
        "id": user_id,
        "password": password,
        "personal": {"name": name or user_id, "gender": "", "job_title": "", "manager": None},
    }
```

---

## Common Test Patterns

### Test that ACL denials return 404

```python
# CORRECT: ACL denial → 404 (not 403)
resp = requests.get(f"{URL_GLOBAL}/groups/{gid}", headers=auth_headers(other_user_token))
assert resp.status_code == 404, f"Expected ACL denial 404, got {resp.status_code}: {resp.text}"
```

### Test godmode bypass

```python
# Root always bypasses ACL
resp = requests.get(f"{URL_GLOBAL}/groups/{gid}", headers=auth_headers(admin_token))
assert resp.status_code == 200  # root sees everything
```

### Test pagination traversal

```python
def collect_all_pages(token, kind, limit=2):
    """Collect all items from paginated list endpoint."""
    collected, cursor = [], None
    while True:
        url = f"{URL_GLOBAL}/{kind}?limit={limit}"
        if cursor:
            url += f"&cursor={cursor}"
        data = requests.get(url, headers=auth_headers(token)).json()
        collected.extend(data["items"])
        if not data["has_more"]:
            break
        cursor = data["next_cursor"]
    return collected
```

### Test brief field presence

```python
def assert_brief(item: dict, must_have: list[str], must_not_have: list[str]):
    for f in must_have:
        assert f in item, f"Brief item missing expected field '{f}': {item}"
    for f in must_not_have:
        assert f not in item, f"Brief item has forbidden field '{f}': {item}"

# Usage:
assert_brief(item,
    must_have=["id", "labels", "personal"],
    must_not_have=["password_hash", "annotations", "hash_code"])
```

---

## What to Test for Every New Endpoint

When adding tests for a new resource kind, cover all of these:

1. **Create** — success (201), creator gets ROOT ACL if applicable
2. **Read** — success (200), response shape correct, no sensitive fields leaked
3. **List** — success (200), response is `{ "items": [...] }`, brief fields only
4. **Update (PUT)** — success (200) for owner, 404 for non-owner
5. **Delete** — success (204) for owner, 404 for non-owner
6. **ACL denied read** — returns 404 (not 403)
7. **ACL denied write** — returns 404 (not 403)
8. **Admin bypass** — root can read/write regardless of ACL
9. **Cleanup** — all test resources deleted (use yield fixtures or inline `requests.delete`)
10. **Soft-delete verification** (optional) — use `debug.assert_soft_deleted()`

---

## File Naming Convention

`{topic}_test.py` — matches `python_files = "*_test.py"` in `pyproject.toml`.

Section markers inside a file:
```python
# ------------------- Section title --------------------
```

---

## Self-Review Checklist

- [ ] All fixtures use `scope="module"` unless per-test isolation is required
- [ ] Random suffixes on all resource IDs (prevent inter-test collisions)
- [ ] Every created resource is deleted (yield teardown or inline cleanup)
- [ ] ACL denials assert `== 404`, not 403
- [ ] Sensitive fields (`password_hash`) never appear in list/brief responses
- [ ] `DebugClient` used in `pytest.fail()` / assertion messages to show DB state
- [ ] Admin-bypass scenario covered for resources with ACL
- [ ] No `conftest.py` — fixtures live in the same file as the tests that use them
- [ ] Tests pass against a clean DB (idempotent, no order dependency)
