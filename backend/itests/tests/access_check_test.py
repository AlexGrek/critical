"""
Integration tests for:
  - GET /v1/accesscheck/global/{kind}/{id}      — resource-level permission bits
  - GET /v1/accesscheck/projects/{proj}/{kind}/{id} — scoped resource permission bits
  - GET /v1/accesscheck/me/permissions          — caller's own super-permissions

Security contract:
  - Returns 404 when the resource does not exist OR when the user has no FETCH
    permission, so the endpoint does not leak resource existence information.
  - Returns the effective permission bitmask and named boolean flags.
  - Godmode users (root) always receive ROOT permissions (all bits set).
"""

import random

import pytest
import requests

BASE = "http://localhost:3742/api"
URL_LOGIN = f"{BASE}/v1/login"
URL_REGISTER = f"{BASE}/v1/register"
URL_GLOBAL = f"{BASE}/v1/global"
URL_AC = f"{BASE}/v1/accesscheck"

ROOT_PASSWORD = "changeme"

# Permission bit constants (mirror shared/src/util_models.rs)
FETCH = 1
LIST = 2
NOTIFY = 4
CREATE = 8
MODIFY = 16
READ = FETCH | LIST | NOTIFY
WRITE = READ | CREATE | MODIFY
ROOT_BITS = 127  # FETCH|LIST|NOTIFY|CREATE|MODIFY|CUSTOM1|CUSTOM2


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
def reader_user(root_token):
    """A plain user with no super-permissions; used as a resource-read subject."""
    num = random.randint(100000, 999999)
    username = f"ac_reader_{num}"
    requests.post(URL_REGISTER, json={"user": username, "password": username})
    resp = requests.post(URL_LOGIN, json={"user": username, "password": username})
    assert resp.status_code == 200
    user = {"id": f"u_{username}", "token": resp.json()["token"]}
    yield user
    requests.delete(
        f"{URL_GLOBAL}/users/{user['id']}", headers=auth_headers(root_token)
    )


@pytest.fixture(scope="module")
def test_group(root_token, reader_user):
    """A group whose ACL gives reader_user READ access."""
    num = random.randint(100000, 999999)
    gid = f"g_ac_test_{num}"
    body = {
        "id": gid,
        "name": f"AC Test Group {num}",
        "acl": {
            "list": [
                {
                    "permissions": READ,
                    "principals": [reader_user["id"]],
                }
            ],
            "last_mod_date": "2024-01-01T00:00:00Z",
        },
    }
    resp = requests.post(
        f"{URL_GLOBAL}/groups", json=body, headers=auth_headers(root_token)
    )
    assert resp.status_code in (200, 201), f"Group creation failed: {resp.text}"
    yield gid
    requests.delete(
        f"{URL_GLOBAL}/groups/{gid}", headers=auth_headers(root_token)
    )


# ─────────────────────── GET /accesscheck/me/permissions ────────────────────


def test_me_permissions_root_has_godmode(root_token):
    """Root's super-permissions include adm_godmode."""
    resp = requests.get(
        f"{URL_AC}/me/permissions", headers=auth_headers(root_token)
    )
    assert resp.status_code == 200, resp.text
    body = resp.json()
    assert "super_permissions" in body
    assert "adm_godmode" in body["super_permissions"]


def test_me_permissions_plain_user_no_godmode(reader_user):
    """A regular user does not have adm_godmode."""
    resp = requests.get(
        f"{URL_AC}/me/permissions", headers=auth_headers(reader_user["token"])
    )
    assert resp.status_code == 200, resp.text
    assert "adm_godmode" not in resp.json()["super_permissions"]


def test_me_permissions_unauthenticated():
    """No token → 401."""
    resp = requests.get(f"{URL_AC}/me/permissions")
    assert resp.status_code == 401, f"Expected 401, got {resp.status_code}: {resp.text}"


# ─────────────────── GET /accesscheck/global/{kind}/{id} ────────────────────


def test_global_access_godmode_returns_root_bits(root_token, test_group):
    """Root (godmode) gets ROOT permission bits on any existing resource."""
    resp = requests.get(
        f"{URL_AC}/global/groups/{test_group}",
        headers=auth_headers(root_token),
    )
    assert resp.status_code == 200, resp.text
    body = resp.json()
    assert body["kind"] == "groups"
    assert body["id"] == test_group
    assert body["permission_bits"] == ROOT_BITS
    assert body["can_fetch"] is True
    assert body["can_modify"] is True
    assert body["can_create"] is True


def test_global_access_reader_gets_read_bits(reader_user, test_group):
    """User with READ ACL entry gets exactly READ bits."""
    resp = requests.get(
        f"{URL_AC}/global/groups/{test_group}",
        headers=auth_headers(reader_user["token"]),
    )
    assert resp.status_code == 200, resp.text
    body = resp.json()
    assert body["permission_bits"] == READ
    assert body["can_fetch"] is True
    assert body["can_list"] is True
    assert body["can_notify"] is True
    assert body["can_create"] is False
    assert body["can_modify"] is False


def test_global_access_no_acl_returns_404(root_token, reader_user):
    """User with no ACL entry on a resource gets 404 (existence not leaked)."""
    # root's own user document has no ACL entry for reader_user
    resp = requests.get(
        f"{URL_AC}/global/users/u_root",
        headers=auth_headers(reader_user["token"]),
    )
    assert resp.status_code == 404, f"Expected 404, got {resp.status_code}: {resp.text}"


def test_global_access_nonexistent_resource_returns_404(root_token):
    """Non-existent resource → 404 even for godmode."""
    resp = requests.get(
        f"{URL_AC}/global/groups/g_definitely_nonexistent_xyz999",
        headers=auth_headers(root_token),
    )
    assert resp.status_code == 404, f"Expected 404, got {resp.status_code}: {resp.text}"


def test_global_access_unauthenticated(test_group):
    """No token → 401."""
    resp = requests.get(f"{URL_AC}/global/groups/{test_group}")
    assert resp.status_code == 401, f"Expected 401, got {resp.status_code}: {resp.text}"


def test_global_access_response_shape(root_token, test_group):
    """Response always includes all named flag fields."""
    resp = requests.get(
        f"{URL_AC}/global/groups/{test_group}",
        headers=auth_headers(root_token),
    )
    body = resp.json()
    for key in ("kind", "id", "permission_bits", "can_fetch", "can_list",
                "can_notify", "can_create", "can_modify"):
        assert key in body, f"Missing key '{key}' in response: {body}"
