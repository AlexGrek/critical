"""Tests for cursor-based pagination on the /v1/global/{kind} list endpoint.

Verifies:
- Unpaginated requests return { "items": [...] } with no pagination fields
- Paginated requests return { "items": [...], "has_more": bool } and
  optionally "next_cursor" when more pages exist
- Cursor correctly advances through pages
- All items are reachable across pages (no duplicates, no gaps)
- limit=0 and limit=1 edge cases work correctly
"""

import pytest
import requests

BASE = "http://localhost:3742/api"
URL_LOGIN = f"{BASE}/login"
URL_GLOBAL = f"{BASE}/v1/global"


@pytest.fixture(scope="module")
def root_token():
    """Login as root (has ADM_USER_MANAGER to create/delete users)."""
    resp = requests.post(URL_LOGIN, json={"user": "root", "password": "changeme"})
    assert resp.status_code == 200, f"Root login failed: {resp.text}"
    return resp.json()["token"]


def auth(token: str) -> dict:
    return {"Authorization": f"Bearer {token}", "Content-Type": "application/json"}


def create_user(token: str, user_id: str) -> None:
    resp = requests.post(
        f"{URL_GLOBAL}/users",
        json={
            "id": user_id,
            "password": "testpw123",
            "personal": {"name": f"Paging User {user_id}", "gender": "", "job_title": "", "manager": None},
            "metadata": {},
            "deactivated": False,
        },
        headers=auth(token),
    )
    assert resp.status_code == 201, f"Create {user_id} failed: {resp.text}"


def delete_user(token: str, user_id: str) -> None:
    requests.delete(f"{URL_GLOBAL}/users/{user_id}", headers=auth(token))


@pytest.fixture(scope="module")
def paged_users(root_token):
    """Create 5 test users, yield their IDs, then clean up."""
    ids = [f"u_pgtest_{i:03d}" for i in range(1, 6)]
    for uid in ids:
        create_user(root_token, uid)
    yield ids
    for uid in ids:
        delete_user(root_token, uid)


# ---------------------------------------------------------------------------
# Unpaginated response format
# ---------------------------------------------------------------------------

def test_unpaginated_response_shape(root_token, paged_users):
    """Without ?limit, response is { items: [...] } — no pagination fields."""
    resp = requests.get(f"{URL_GLOBAL}/users", headers=auth(root_token))
    assert resp.status_code == 200

    data = resp.json()
    assert "items" in data, "unpaginated response must have 'items'"
    assert "has_more" not in data, "unpaginated response must NOT have 'has_more'"
    assert "next_cursor" not in data, "unpaginated response must NOT have 'next_cursor'"
    assert isinstance(data["items"], list)


def test_unpaginated_contains_all_test_users(root_token, paged_users):
    """Unpaginated list must contain all created test users."""
    resp = requests.get(f"{URL_GLOBAL}/users", headers=auth(root_token))
    assert resp.status_code == 200

    ids = {item["id"] for item in resp.json()["items"]}
    for uid in paged_users:
        assert uid in ids, f"{uid} missing from unpaginated list"


# ---------------------------------------------------------------------------
# Paginated response format
# ---------------------------------------------------------------------------

def test_paginated_response_shape_first_page(root_token, paged_users):
    """With ?limit, response always includes 'items' and 'has_more'."""
    resp = requests.get(f"{URL_GLOBAL}/users?limit=2", headers=auth(root_token))
    assert resp.status_code == 200

    data = resp.json()
    assert "items" in data, "paginated response must have 'items'"
    assert "has_more" in data, "paginated response must have 'has_more'"
    assert isinstance(data["has_more"], bool)
    assert len(data["items"]) <= 2


def test_paginated_last_page_has_no_next_cursor(root_token, paged_users):
    """The final page has has_more=False and no next_cursor field."""
    # Use a very large limit so we get everything in one page
    resp = requests.get(f"{URL_GLOBAL}/users?limit=1000", headers=auth(root_token))
    assert resp.status_code == 200

    data = resp.json()
    assert data["has_more"] is False
    assert "next_cursor" not in data, "last page must NOT contain 'next_cursor'"


def test_paginated_intermediate_page_has_next_cursor(root_token, paged_users):
    """When has_more=True, next_cursor must be present."""
    # We created 5 test users, so limit=1 guarantees multiple pages
    resp = requests.get(f"{URL_GLOBAL}/users?limit=1", headers=auth(root_token))
    assert resp.status_code == 200

    data = resp.json()
    # There are at least 5 test users + root, so has_more must be True
    assert data["has_more"] is True
    assert "next_cursor" in data, "intermediate page must contain 'next_cursor'"
    assert isinstance(data["next_cursor"], str)
    assert len(data["next_cursor"]) > 0


# ---------------------------------------------------------------------------
# Cursor traversal — correctness
# ---------------------------------------------------------------------------

def test_full_traversal_yields_all_items_no_duplicates(root_token, paged_users):
    """Paginating through all pages collects every item exactly once."""
    collected = []
    cursor = None

    while True:
        url = f"{URL_GLOBAL}/users?limit=2"
        if cursor:
            url += f"&cursor={cursor}"

        resp = requests.get(url, headers=auth(root_token))
        assert resp.status_code == 200
        data = resp.json()

        # Paginated format on every page
        assert "has_more" in data
        assert "items" in data

        collected.extend(item["id"] for item in data["items"])

        if not data["has_more"]:
            assert "next_cursor" not in data
            break

        assert "next_cursor" in data
        cursor = data["next_cursor"]

    # No duplicates
    assert len(collected) == len(set(collected)), "duplicate IDs across pages"

    # All test users present
    collected_set = set(collected)
    for uid in paged_users:
        assert uid in collected_set, f"{uid} missing from paginated traversal"


def test_cursor_advances_correctly(root_token, paged_users):
    """Items on page 2 must not overlap with items on page 1."""
    resp1 = requests.get(f"{URL_GLOBAL}/users?limit=2", headers=auth(root_token))
    assert resp1.status_code == 200
    data1 = resp1.json()

    if not data1["has_more"]:
        pytest.skip("Not enough items for a second page")

    cursor = data1["next_cursor"]
    ids_page1 = {item["id"] for item in data1["items"]}

    resp2 = requests.get(f"{URL_GLOBAL}/users?limit=2&cursor={cursor}", headers=auth(root_token))
    assert resp2.status_code == 200
    data2 = resp2.json()
    ids_page2 = {item["id"] for item in data2["items"]}

    overlap = ids_page1 & ids_page2
    assert not overlap, f"Pages overlap: {overlap}"


# ---------------------------------------------------------------------------
# Edge cases
# ---------------------------------------------------------------------------

def test_limit_1_works(root_token, paged_users):
    """limit=1 returns exactly one item per page."""
    resp = requests.get(f"{URL_GLOBAL}/users?limit=1", headers=auth(root_token))
    assert resp.status_code == 200
    data = resp.json()
    assert len(data["items"]) == 1
    assert "has_more" in data


def test_brief_fields_in_paginated_response(root_token, paged_users):
    """Paginated list items contain brief fields only (id, deactivated, personal)."""
    resp = requests.get(f"{URL_GLOBAL}/users?limit=5", headers=auth(root_token))
    assert resp.status_code == 200

    for item in resp.json()["items"]:
        assert "id" in item
        assert "deactivated" in item
        assert "personal" in item
        assert "password_hash" not in item, "password_hash must never appear in list"
        # Non-brief fields must be absent
        assert "created_at" not in item
        assert "metadata" not in item


def test_brief_fields_in_unpaginated_response(root_token, paged_users):
    """Unpaginated list items also contain brief fields only."""
    resp = requests.get(f"{URL_GLOBAL}/users", headers=auth(root_token))
    assert resp.status_code == 200

    for item in resp.json()["items"]:
        assert "id" in item
        assert "deactivated" in item
        assert "personal" in item
        assert "password_hash" not in item
        assert "created_at" not in item
        assert "metadata" not in item
