import pytest
import requests

BASE = "http://localhost:3742/api"
URL_LOGIN = f"{BASE}/login"
URL_REGISTER = f"{BASE}/register"
URL_DEBUG = f"{BASE}/v1/debug"

ROOT_PASSWORD = "changeme"


def auth_headers(token: str) -> dict:
    return {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json",
    }


@pytest.fixture(scope="module")
def root_token():
    resp = requests.post(URL_LOGIN, json={"user": "root", "password": ROOT_PASSWORD})
    assert resp.status_code == 200, f"Root login failed: {resp.text}"
    return resp.json()["token"]


@pytest.fixture(scope="module")
def regular_token():
    import random
    num = random.randint(100000, 999999)
    user = f"debug_itest_{num}"
    requests.post(URL_REGISTER, json={"user": user, "password": user})
    resp = requests.post(URL_LOGIN, json={"user": user, "password": user})
    assert resp.status_code == 200, f"Login failed: {resp.text}"
    return resp.json()["token"]


# ------------------- GET /v1/debug/collections --------------------


def test_godmode_user_can_list_collections(root_token):
    resp = requests.get(f"{URL_DEBUG}/collections", headers=auth_headers(root_token))
    assert resp.status_code == 200, resp.text
    body = resp.json()
    assert "collections" in body
    cols = body["collections"]
    assert isinstance(cols, list)
    assert len(cols) > 0, "Expected at least one non-system collection"
    # Every item must have a name field
    for col in cols:
        assert "name" in col, f"Collection entry missing 'name': {col}"
    # Our known collections must be present
    names = {c["name"] for c in cols}
    for expected in ("users", "groups", "projects", "memberships", "permissions"):
        assert expected in names, f"Expected collection '{expected}' in listing, got: {names}"


def test_regular_user_cannot_list_collections(regular_token):
    resp = requests.get(f"{URL_DEBUG}/collections", headers=auth_headers(regular_token))
    assert resp.status_code == 401, f"Expected 401, got {resp.status_code}: {resp.text}"


def test_unauthenticated_cannot_list_collections():
    resp = requests.get(f"{URL_DEBUG}/collections")
    assert resp.status_code == 401, f"Expected 401, got {resp.status_code}: {resp.text}"


# ------------------- GET /v1/debug/collections/{name} --------------------


def test_godmode_user_can_dump_users_collection(root_token):
    resp = requests.get(f"{URL_DEBUG}/collections/users", headers=auth_headers(root_token))
    assert resp.status_code == 200, resp.text
    body = resp.json()
    assert body["collection"] == "users"
    assert isinstance(body["count"], int)
    assert isinstance(body["documents"], list)
    assert body["count"] == len(body["documents"])
    # Root user must exist in the dump
    ids = [d.get("_key", "") for d in body["documents"]]
    assert "u_root" in ids, f"u_root not found in users dump: {ids}"


def test_godmode_cannot_dump_system_collection(root_token):
    resp = requests.get(f"{URL_DEBUG}/collections/_system", headers=auth_headers(root_token))
    assert resp.status_code == 400, f"Expected 400 for system collection, got {resp.status_code}: {resp.text}"


def test_regular_user_cannot_dump_collection(regular_token):
    resp = requests.get(f"{URL_DEBUG}/collections/users", headers=auth_headers(regular_token))
    assert resp.status_code == 401, f"Expected 401, got {resp.status_code}: {resp.text}"


def test_unauthenticated_cannot_dump_collection():
    resp = requests.get(f"{URL_DEBUG}/collections/users")
    assert resp.status_code == 401, f"Expected 401, got {resp.status_code}: {resp.text}"
