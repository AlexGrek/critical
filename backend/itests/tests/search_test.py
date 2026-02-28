"""Tests for GET /v1/global/{kind}/search?startwith={prefix}.

Verifies:
- Response shape: { "items": [...] } with no pagination fields
- Prefix filtering: only _key values starting with the prefix are returned
- Hard limit of 15 results even when more matches exist
- Brief fields only (matches the list endpoint projection)
- Results sorted by _key ASC
- ACL filtering: per-doc ACL hides resources from unauthorised users (groups)
- Admin bypass: root sees ACL-restricted resources
- Invalid kind rejected with 400
- Empty / missing startwith returns up to 15 items without prefix filtering
"""

import pytest
import random
import requests
from datetime import datetime, timezone


BASE = "http://localhost:3742/api"
URL_LOGIN = f"{BASE}/v1/login"
URL_REGISTER = f"{BASE}/v1/register"
URL_GLOBAL = f"{BASE}/v1/global"


def auth_headers(token: str) -> dict:
    return {"Authorization": f"Bearer {token}", "Content-Type": "application/json"}


def make_acl(*entries) -> dict:
    return {"list": list(entries), "last_mod_date": datetime.now(timezone.utc).isoformat()}


def acl_entry(permissions: int, *principal_ids: str) -> dict:
    return {"permissions": permissions, "principals": list(principal_ids)}


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------

@pytest.fixture(scope="module")
def admin_token():
    """Login as root — has ADM_GODMODE + ADM_USER_MANAGER."""
    resp = requests.post(URL_LOGIN, json={"user": "root", "password": "changeme"})
    assert resp.status_code == 200, f"Root login failed: {resp.text}"
    return resp.json()["token"]


@pytest.fixture(scope="module")
def regular_user():
    """Register a fresh random user (gets usr_create_groups by default)."""
    num = random.randint(100000, 999999)
    user = f"srch_itest_{num}"
    requests.post(URL_REGISTER, json={"user": user, "password": user})
    resp = requests.post(URL_LOGIN, json={"user": user, "password": user})
    assert resp.status_code == 200, f"Login failed: {resp.text}"
    return {"user_id": f"u_{user}", "token": resp.json()["token"]}


@pytest.fixture(scope="module")
def search_users(admin_token):
    """Create 16 users with a shared prefix (one more than the hard limit).

    IDs: u_srch_{run}_001 … u_srch_{run}_016  (lexicographic order preserved)
    """
    run = random.randint(100000, 999999)
    prefix = f"u_srch_{run}_"
    ids = [f"{prefix}{i:03d}" for i in range(1, 17)]  # 001–016
    headers = auth_headers(admin_token)

    for uid in ids:
        resp = requests.post(
            f"{URL_GLOBAL}/users",
            json={
                "id": uid,
                "password": "testpw123",
                "personal": {"name": uid, "gender": "", "job_title": "", "manager": None},
            },
            headers=headers,
        )
        assert resp.status_code == 201, f"Create {uid} failed: {resp.text}"

    yield prefix, ids

    for uid in ids:
        requests.delete(f"{URL_GLOBAL}/users/{uid}", headers=headers)


@pytest.fixture(scope="module")
def acl_groups(admin_token, regular_user):
    """Create two groups with the same prefix but different ACLs.

    - visible_group: regular_user has READ (permissions=7)
    - hidden_group:  only u_root in ACL — invisible to regular_user
    """
    run = random.randint(100000, 999999)
    prefix = f"g_srch_{run}_"
    visible_id = f"{prefix}visible"
    hidden_id = f"{prefix}hidden"
    headers = auth_headers(admin_token)

    for gid, acl in [
        (visible_id, make_acl(acl_entry(127, "u_root"), acl_entry(7, regular_user["user_id"]))),
        (hidden_id,  make_acl(acl_entry(127, "u_root"))),
    ]:
        resp = requests.post(
            f"{URL_GLOBAL}/groups",
            json={"id": gid, "name": gid, "acl": acl},
            headers=headers,
        )
        assert resp.status_code == 201, f"Create group {gid} failed: {resp.text}"

    yield prefix, visible_id, hidden_id

    for gid in (visible_id, hidden_id):
        requests.delete(f"{URL_GLOBAL}/groups/{gid}", headers=headers)


# ---------------------------------------------------------------------------
# Response shape
# ---------------------------------------------------------------------------

def test_response_shape(admin_token, search_users):
    prefix, _ = search_users
    resp = requests.get(
        f"{URL_GLOBAL}/users/search?startwith={prefix}",
        headers=auth_headers(admin_token),
    )
    assert resp.status_code == 200
    data = resp.json()
    assert "items" in data, "search response must have 'items'"
    assert "has_more" not in data, "search response must NOT have 'has_more'"
    assert "next_cursor" not in data, "search response must NOT have 'next_cursor'"
    assert isinstance(data["items"], list)


# ---------------------------------------------------------------------------
# Prefix filtering
# ---------------------------------------------------------------------------

def test_prefix_filters_to_matching_items(admin_token, search_users):
    """Only items whose _key starts with the prefix appear in the result."""
    prefix, ids = search_users
    resp = requests.get(
        f"{URL_GLOBAL}/users/search?startwith={prefix}",
        headers=auth_headers(admin_token),
    )
    assert resp.status_code == 200
    returned_ids = {item["id"] for item in resp.json()["items"]}
    for item_id in returned_ids:
        assert item_id.startswith(prefix), (
            f"Item '{item_id}' does not start with prefix '{prefix}'"
        )


def test_prefix_excludes_non_matching_items(admin_token, search_users):
    """u_root must not appear in a search that is scoped to our run-specific prefix."""
    prefix, _ = search_users
    resp = requests.get(
        f"{URL_GLOBAL}/users/search?startwith={prefix}",
        headers=auth_headers(admin_token),
    )
    assert resp.status_code == 200
    returned_ids = {item["id"] for item in resp.json()["items"]}
    assert "u_root" not in returned_ids


def test_no_match_returns_empty_items(admin_token):
    """A prefix that cannot match any _key returns an empty items list."""
    resp = requests.get(
        f"{URL_GLOBAL}/users/search?startwith=u_zzz_no_such_user_xyzzy_",
        headers=auth_headers(admin_token),
    )
    assert resp.status_code == 200
    assert resp.json()["items"] == []


# ---------------------------------------------------------------------------
# Hard limit of 15
# ---------------------------------------------------------------------------

def test_hard_limit_is_15(admin_token, search_users):
    """Even when 16 matching documents exist, search returns at most 15."""
    prefix, ids = search_users
    assert len(ids) == 16, "fixture must create 16 users to probe the hard limit"

    resp = requests.get(
        f"{URL_GLOBAL}/users/search?startwith={prefix}",
        headers=auth_headers(admin_token),
    )
    assert resp.status_code == 200
    assert len(resp.json()["items"]) == 15


# ---------------------------------------------------------------------------
# Brief fields
# ---------------------------------------------------------------------------

def test_brief_fields_present_and_sensitive_absent(admin_token, search_users):
    """Search items contain only brief fields — same projection as list."""
    prefix, _ = search_users
    resp = requests.get(
        f"{URL_GLOBAL}/users/search?startwith={prefix}",
        headers=auth_headers(admin_token),
    )
    assert resp.status_code == 200
    for item in resp.json()["items"]:
        assert "id" in item, f"Missing 'id' in {item}"
        assert "personal" in item, f"Missing 'personal' in {item}"
        assert "labels" in item, f"Missing 'labels' in {item}"
        assert "password_hash" not in item, "'password_hash' must never appear in search"
        assert "annotations" not in item, "'annotations' should not appear in brief search"
        assert "hash_code" not in item, "'hash_code' should not appear in brief search"


# ---------------------------------------------------------------------------
# Sort order
# ---------------------------------------------------------------------------

def test_results_sorted_by_key_asc(admin_token, search_users):
    """Items are returned sorted by _key (id) ascending."""
    prefix, _ = search_users
    resp = requests.get(
        f"{URL_GLOBAL}/users/search?startwith={prefix}",
        headers=auth_headers(admin_token),
    )
    assert resp.status_code == 200
    ids = [item["id"] for item in resp.json()["items"]]
    assert ids == sorted(ids), f"Items not sorted: {ids}"


# ---------------------------------------------------------------------------
# Empty / missing startwith
# ---------------------------------------------------------------------------

def test_missing_startwith_returns_items(admin_token, search_users):
    """Omitting ?startwith returns up to 15 items (no prefix filter applied)."""
    resp = requests.get(
        f"{URL_GLOBAL}/users/search",
        headers=auth_headers(admin_token),
    )
    assert resp.status_code == 200
    data = resp.json()
    assert "items" in data
    assert len(data["items"]) <= 15


def test_empty_startwith_returns_items(admin_token, search_users):
    """?startwith= (empty string) matches all keys — returns up to 15 items."""
    resp = requests.get(
        f"{URL_GLOBAL}/users/search?startwith=",
        headers=auth_headers(admin_token),
    )
    assert resp.status_code == 200
    data = resp.json()
    assert "items" in data
    assert len(data["items"]) <= 15


# ---------------------------------------------------------------------------
# Works on other kinds (groups)
# ---------------------------------------------------------------------------

def test_search_works_on_groups(admin_token, acl_groups):
    prefix, visible_id, hidden_id = acl_groups
    resp = requests.get(
        f"{URL_GLOBAL}/groups/search?startwith={prefix}",
        headers=auth_headers(admin_token),
    )
    assert resp.status_code == 200
    ids = {item["id"] for item in resp.json()["items"]}
    assert visible_id in ids
    assert hidden_id in ids


# ---------------------------------------------------------------------------
# ACL filtering
# ---------------------------------------------------------------------------

def test_acl_hidden_group_not_visible_to_regular_user(regular_user, acl_groups):
    """A group whose ACL excludes regular_user must not appear in their search."""
    prefix, visible_id, hidden_id = acl_groups
    resp = requests.get(
        f"{URL_GLOBAL}/groups/search?startwith={prefix}",
        headers=auth_headers(regular_user["token"]),
    )
    assert resp.status_code == 200
    ids = {item["id"] for item in resp.json()["items"]}
    assert hidden_id not in ids, (
        f"Hidden group '{hidden_id}' should not be visible to regular user"
    )


def test_acl_visible_group_appears_for_regular_user(regular_user, acl_groups):
    """A group whose ACL grants READ to regular_user must appear in their search."""
    prefix, visible_id, _ = acl_groups
    resp = requests.get(
        f"{URL_GLOBAL}/groups/search?startwith={prefix}",
        headers=auth_headers(regular_user["token"]),
    )
    assert resp.status_code == 200
    ids = {item["id"] for item in resp.json()["items"]}
    assert visible_id in ids, (
        f"Visible group '{visible_id}' should appear for regular user"
    )


def test_admin_sees_acl_restricted_groups(admin_token, acl_groups):
    """Root bypasses all ACL checks and sees both groups."""
    prefix, visible_id, hidden_id = acl_groups
    resp = requests.get(
        f"{URL_GLOBAL}/groups/search?startwith={prefix}",
        headers=auth_headers(admin_token),
    )
    assert resp.status_code == 200
    ids = {item["id"] for item in resp.json()["items"]}
    assert visible_id in ids
    assert hidden_id in ids


# ---------------------------------------------------------------------------
# Invalid kind
# ---------------------------------------------------------------------------

def test_invalid_kind_rejected(admin_token):
    """Kind strings containing special characters are rejected with 400."""
    resp = requests.get(
        f"{URL_GLOBAL}/bad-kind!/search?startwith=u_",
        headers=auth_headers(admin_token),
    )
    assert resp.status_code == 400
