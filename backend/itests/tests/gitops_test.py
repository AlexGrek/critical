import pytest
import requests
import random


BASE = "http://localhost:3742/api"
URL_LOGIN = f"{BASE}/v1/login"
URL_GLOBAL = f"{BASE}/v1/global"


@pytest.fixture(scope="module")
def auth_token():
    """Login as root (has ADM_USER_MANAGER required for user CRUD)."""
    resp = requests.post(URL_LOGIN, json={"user": "root", "password": "changeme"})
    assert resp.status_code == 200, f"Root login failed: {resp.text}"
    return resp.json()["token"]


def auth_headers(token: str) -> dict:
    return {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json",
    }


def test_gitops_create_user(auth_token):
    """Create a user via gitops POST /global/users, then verify via GET."""
    num = random.randint(100000, 999999)
    user_id = f"u_gitops_{num}"
    headers = auth_headers(auth_token)

    # Create
    resp = requests.post(
        f"{URL_GLOBAL}/users",
        json={
            "id": user_id,
            "password": "testpassword123",
            "personal": {
                "name": "Gitops Test User",
                "gender": "",
                "job_title": "Tester",
                "manager": None,
            },
        },
        headers=headers,
    )
    assert resp.status_code == 201, f"Expected 201, got {resp.status_code}: {resp.text}"
    assert resp.json()["id"] == user_id

    # Get
    resp = requests.get(f"{URL_GLOBAL}/users/{user_id}", headers=headers)
    assert resp.status_code == 200, f"Expected 200, got {resp.status_code}: {resp.text}"
    data = resp.json()
    assert data["id"] == user_id
    assert data["personal"]["name"] == "Gitops Test User"
    assert "password_hash" not in data, "password_hash should be stripped from response"

    # List
    resp = requests.get(f"{URL_GLOBAL}/users", headers=headers)
    assert resp.status_code == 200
    items = resp.json()["items"]
    ids = [item["id"] for item in items]
    assert user_id in ids

    # Cleanup
    resp = requests.delete(f"{URL_GLOBAL}/users/{user_id}", headers=headers)
    assert resp.status_code == 204


# ---------------------------------------------------------------------------
# Hash-based Optimistic Concurrency Control (OCC)
# ---------------------------------------------------------------------------

def _create_group(headers: dict, suffix: str) -> str:
    """Helper: create a group and return its id."""
    group_id = f"g_occ_{suffix}"
    resp = requests.post(
        f"{URL_GLOBAL}/groups",
        json={"id": group_id, "name": f"OCC Test {suffix}"},
        headers=headers,
    )
    assert resp.status_code == 201, f"Create group failed: {resp.text}"
    return group_id


def test_hash_present_after_create(auth_token):
    """GET after CREATE returns a non-empty hash_code."""
    headers = auth_headers(auth_token)
    num = random.randint(100000, 999999)
    group_id = _create_group(headers, str(num))

    resp = requests.get(f"{URL_GLOBAL}/groups/{group_id}", headers=headers)
    assert resp.status_code == 200
    data = resp.json()
    assert "hash_code" in data, "hash_code field must be present in GET response"
    assert data["hash_code"], "hash_code must be non-empty"

    # Cleanup
    requests.delete(f"{URL_GLOBAL}/groups/{group_id}", headers=headers)


def test_hash_changes_after_update(auth_token):
    """PUT changes the stored hash_code."""
    headers = auth_headers(auth_token)
    num = random.randint(100000, 999999)
    group_id = _create_group(headers, str(num))

    resp = requests.get(f"{URL_GLOBAL}/groups/{group_id}", headers=headers)
    assert resp.status_code == 200
    original_hash = resp.json()["hash_code"]

    group = resp.json()
    group["name"] = "OCC Updated Name"
    resp = requests.put(f"{URL_GLOBAL}/groups/{group_id}", json=group, headers=headers)
    assert resp.status_code == 200, f"PUT failed: {resp.text}"

    resp = requests.get(f"{URL_GLOBAL}/groups/{group_id}", headers=headers)
    assert resp.status_code == 200
    new_hash = resp.json()["hash_code"]
    assert new_hash != original_hash, "hash_code must change after an update"

    # Cleanup
    requests.delete(f"{URL_GLOBAL}/groups/{group_id}", headers=headers)


def test_put_with_stale_hash_returns_409(auth_token):
    """PUT with an outdated hash_code returns 409 Conflict."""
    headers = auth_headers(auth_token)
    num = random.randint(100000, 999999)
    group_id = _create_group(headers, str(num))

    # Fetch and do a first update — now the stored hash has changed.
    resp = requests.get(f"{URL_GLOBAL}/groups/{group_id}", headers=headers)
    assert resp.status_code == 200
    original = resp.json()
    stale_hash = original["hash_code"]

    original["name"] = "OCC First Update"
    resp = requests.put(f"{URL_GLOBAL}/groups/{group_id}", json=original, headers=headers)
    assert resp.status_code == 200, f"First PUT failed: {resp.text}"

    # Now send the stale hash from the first GET — should conflict.
    original["hash_code"] = stale_hash
    original["name"] = "OCC Stale Conflict"
    resp = requests.put(f"{URL_GLOBAL}/groups/{group_id}", json=original, headers=headers)
    assert resp.status_code == 409, f"Expected 409, got {resp.status_code}: {resp.text}"

    # Cleanup
    requests.delete(f"{URL_GLOBAL}/groups/{group_id}", headers=headers)


def test_put_without_hash_always_succeeds(auth_token):
    """PUT without hash_code in body never triggers OCC — allows blind overwrites."""
    headers = auth_headers(auth_token)
    num = random.randint(100000, 999999)
    group_id = _create_group(headers, str(num))

    # Send an update with no hash_code field at all.
    resp = requests.put(
        f"{URL_GLOBAL}/groups/{group_id}",
        json={"id": group_id, "name": "OCC No Hash Update"},
        headers=headers,
    )
    assert resp.status_code == 200, f"PUT without hash failed: {resp.text}"

    # Cleanup
    requests.delete(f"{URL_GLOBAL}/groups/{group_id}", headers=headers)


def test_upsert_with_stale_hash_returns_409(auth_token):
    """POST (upsert) with an outdated hash_code returns 409 on update path."""
    headers = auth_headers(auth_token)
    num = random.randint(100000, 999999)
    group_id = _create_group(headers, str(num))

    resp = requests.get(f"{URL_GLOBAL}/groups/{group_id}", headers=headers)
    assert resp.status_code == 200
    original = resp.json()
    stale_hash = original["hash_code"]

    # First upsert — advances the hash.
    original["name"] = "OCC Upsert First"
    resp = requests.post(f"{URL_GLOBAL}/groups/{group_id}", json=original, headers=headers)
    assert resp.status_code == 200, f"First upsert failed: {resp.text}"

    # Second upsert with the stale hash — should conflict.
    original["hash_code"] = stale_hash
    original["name"] = "OCC Upsert Stale"
    resp = requests.post(f"{URL_GLOBAL}/groups/{group_id}", json=original, headers=headers)
    assert resp.status_code == 409, f"Expected 409, got {resp.status_code}: {resp.text}"

    # Cleanup
    requests.delete(f"{URL_GLOBAL}/groups/{group_id}", headers=headers)


# ── Resource History ──────────────────────────────────────────────────────


def test_history_revision_1_after_create(auth_token):
    """Creating a resource writes revision 1 to resource_history (via ?with_history=true)."""
    num = random.randint(100000, 999999)
    group_id = f"g_hist_create_{num}"
    headers = auth_headers(auth_token)

    # Create group
    resp = requests.post(
        f"{URL_GLOBAL}/groups",
        json={"id": group_id, "name": "History Create Test"},
        headers=headers,
    )
    assert resp.status_code == 201, f"Create failed: {resp.text}"

    # GET with history
    resp = requests.get(f"{URL_GLOBAL}/groups/{group_id}?with_history=true", headers=headers)
    assert resp.status_code == 200, f"GET failed: {resp.text}"
    data = resp.json()

    assert "_history" in data, "Expected _history field in response"
    history = data["_history"]
    assert history["revision"] == 1, f"Expected revision 1, got {history['revision']}"
    assert history["resource_kind"] == "groups"
    assert history["resource_key"] == group_id

    # Cleanup
    requests.delete(f"{URL_GLOBAL}/groups/{group_id}", headers=headers)


def test_history_revision_increments_after_update(auth_token):
    """Updating a resource increments the history revision."""
    num = random.randint(100000, 999999)
    group_id = f"g_hist_upd_{num}"
    headers = auth_headers(auth_token)

    # Create group
    resp = requests.post(
        f"{URL_GLOBAL}/groups",
        json={"id": group_id, "name": "History Update Test"},
        headers=headers,
    )
    assert resp.status_code == 201, f"Create failed: {resp.text}"

    # GET to retrieve current state (including hash_code for OCC)
    resp = requests.get(f"{URL_GLOBAL}/groups/{group_id}", headers=headers)
    assert resp.status_code == 200
    doc = resp.json()

    # PUT update
    doc["name"] = "History Update Test — Modified"
    resp = requests.put(f"{URL_GLOBAL}/groups/{group_id}", json=doc, headers=headers)
    assert resp.status_code == 200, f"PUT failed: {resp.text}"

    # GET with history — expect revision 2
    resp = requests.get(f"{URL_GLOBAL}/groups/{group_id}?with_history=true", headers=headers)
    assert resp.status_code == 200
    data = resp.json()

    assert "_history" in data, "Expected _history field after update"
    history = data["_history"]
    assert history["revision"] == 2, f"Expected revision 2 after update, got {history['revision']}"

    # Cleanup
    requests.delete(f"{URL_GLOBAL}/groups/{group_id}", headers=headers)


def test_get_without_history_param_has_no_history(auth_token):
    """GET without ?with_history=true never includes _history field."""
    num = random.randint(100000, 999999)
    group_id = f"g_hist_no_{num}"
    headers = auth_headers(auth_token)

    resp = requests.post(
        f"{URL_GLOBAL}/groups",
        json={"id": group_id, "name": "No History Param Test"},
        headers=headers,
    )
    assert resp.status_code == 201

    resp = requests.get(f"{URL_GLOBAL}/groups/{group_id}", headers=headers)
    assert resp.status_code == 200
    assert "_history" not in resp.json(), "Unexpected _history field without param"

    # Cleanup
    requests.delete(f"{URL_GLOBAL}/groups/{group_id}", headers=headers)


def test_get_with_history_false_has_no_history(auth_token):
    """GET with ?with_history=false does not include _history field."""
    num = random.randint(100000, 999999)
    group_id = f"g_hist_false_{num}"
    headers = auth_headers(auth_token)

    resp = requests.post(
        f"{URL_GLOBAL}/groups",
        json={"id": group_id, "name": "History False Test"},
        headers=headers,
    )
    assert resp.status_code == 201

    resp = requests.get(f"{URL_GLOBAL}/groups/{group_id}?with_history=false", headers=headers)
    assert resp.status_code == 200
    assert "_history" not in resp.json(), "Unexpected _history field when with_history=false"

    # Cleanup
    requests.delete(f"{URL_GLOBAL}/groups/{group_id}", headers=headers)
