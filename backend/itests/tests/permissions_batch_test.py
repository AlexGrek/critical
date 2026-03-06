"""
Integration tests for:
  - POST /v1/global/permissions/{key}/grant  — grant to one or multiple principals
  - POST /v1/global/permissions/{key}/revoke — revoke from one or multiple principals
  - GET  /v1/debug/access                    — permission inspection endpoint
"""

import random

import pytest
import requests

BASE = "http://localhost:3742/api"
URL_LOGIN = f"{BASE}/v1/login"
URL_REGISTER = f"{BASE}/v1/register"
URL_GLOBAL = f"{BASE}/v1/global"
URL_DEBUG = f"{BASE}/v1/debug"

PERM_CREATE_GROUPS = "usr_create_groups"
PERM_CREATE_PROJECTS = "usr_create_projects"
PERM_USER_MANAGER = "adm_user_manager"

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
def test_user(root_token):
    """Create a fresh user for permission tests; clean up afterwards."""
    num = random.randint(100000, 999999)
    username = f"perm_batch_{num}"

    resp = requests.post(URL_REGISTER, json={"user": username, "password": username})
    assert resp.status_code in (200, 201), f"Register failed: {resp.text}"

    resp = requests.post(URL_LOGIN, json={"user": username, "password": username})
    assert resp.status_code == 200, f"Login failed: {resp.text}"

    user = {
        "user_id": f"u_{username}",
        "token": resp.json()["token"],
    }
    yield user

    # Cleanup
    requests.delete(
        f"{URL_GLOBAL}/users/{user['user_id']}",
        headers=auth_headers(root_token),
    )


# ───────────────────────────── Grant endpoint ─────────────────────────────


def test_grant_single_principal_shorthand(root_token, test_user):
    """Grant using the `principal` (singular) convenience field."""
    resp = requests.post(
        f"{URL_GLOBAL}/permissions/{PERM_CREATE_GROUPS}/grant",
        json={"principal": test_user["user_id"]},
        headers=auth_headers(root_token),
    )
    assert resp.status_code == 200, resp.text
    body = resp.json()
    assert body["permission"] == PERM_CREATE_GROUPS
    assert test_user["user_id"] in body["granted_to"]


def test_grant_multiple_principals(root_token, test_user):
    """Grant using `principals` array — grants two permissions, verifies both."""
    resp = requests.post(
        f"{URL_GLOBAL}/permissions/{PERM_CREATE_PROJECTS}/grant",
        json={"principals": [test_user["user_id"]]},
        headers=auth_headers(root_token),
    )
    assert resp.status_code == 200, resp.text

    # Verify via debug endpoint
    debug = requests.get(
        f"{URL_DEBUG}/access",
        params={"user": test_user["user_id"]},
        headers=auth_headers(root_token),
    )
    assert debug.status_code == 200, debug.text
    super_perms = debug.json()["super_permissions"]
    assert PERM_CREATE_GROUPS in super_perms, f"Expected {PERM_CREATE_GROUPS}, got {super_perms}"
    assert PERM_CREATE_PROJECTS in super_perms, f"Expected {PERM_CREATE_PROJECTS}, got {super_perms}"


def test_grant_is_idempotent(root_token, test_user):
    """Granting the same permission twice is a no-op (200, no error)."""
    for _ in range(2):
        resp = requests.post(
            f"{URL_GLOBAL}/permissions/{PERM_CREATE_GROUPS}/grant",
            json={"principals": [test_user["user_id"]]},
            headers=auth_headers(root_token),
        )
        assert resp.status_code == 200, resp.text


def test_grant_requires_permission_manager(test_user):
    """Regular user without ADM_USER_MANAGER cannot grant permissions (403)."""
    resp = requests.post(
        f"{URL_GLOBAL}/permissions/{PERM_USER_MANAGER}/grant",
        json={"principals": [test_user["user_id"]]},
        headers=auth_headers(test_user["token"]),
    )
    assert resp.status_code == 403, f"Expected 403, got {resp.status_code}: {resp.text}"


def test_grant_unauthenticated():
    """Unauthenticated request is rejected."""
    resp = requests.post(
        f"{URL_GLOBAL}/permissions/{PERM_CREATE_GROUPS}/grant",
        json={"principals": ["u_root"]},
    )
    assert resp.status_code == 401, f"Expected 401, got {resp.status_code}: {resp.text}"


def test_grant_empty_principals_rejected(root_token):
    """Empty principals list returns 400."""
    resp = requests.post(
        f"{URL_GLOBAL}/permissions/{PERM_CREATE_GROUPS}/grant",
        json={"principals": []},
        headers=auth_headers(root_token),
    )
    assert resp.status_code == 400, f"Expected 400, got {resp.status_code}: {resp.text}"


# ───────────────────────────── Revoke endpoint ────────────────────────────


def test_revoke_single_permission(root_token, test_user):
    """Revoke one permission; the other granted one must remain."""
    resp = requests.post(
        f"{URL_GLOBAL}/permissions/{PERM_CREATE_PROJECTS}/revoke",
        json={"principal": test_user["user_id"]},
        headers=auth_headers(root_token),
    )
    assert resp.status_code == 200, resp.text
    body = resp.json()
    assert body["permission"] == PERM_CREATE_PROJECTS
    assert test_user["user_id"] in body["revoked_from"]

    # Verify state via debug
    debug = requests.get(
        f"{URL_DEBUG}/access",
        params={"user": test_user["user_id"]},
        headers=auth_headers(root_token),
    )
    super_perms = debug.json()["super_permissions"]
    assert PERM_CREATE_PROJECTS not in super_perms, f"{PERM_CREATE_PROJECTS} should be revoked"
    assert PERM_CREATE_GROUPS in super_perms, f"{PERM_CREATE_GROUPS} should still be present"


def test_revoke_nonexistent_permission_is_silent(root_token, test_user):
    """Revoking a permission the user doesn't have is a no-op (200)."""
    resp = requests.post(
        f"{URL_GLOBAL}/permissions/{PERM_USER_MANAGER}/revoke",
        json={"principals": [test_user["user_id"]]},
        headers=auth_headers(root_token),
    )
    assert resp.status_code == 200, resp.text


def test_revoke_requires_permission_manager(test_user):
    """Regular user cannot revoke permissions (403)."""
    resp = requests.post(
        f"{URL_GLOBAL}/permissions/{PERM_CREATE_GROUPS}/revoke",
        json={"principals": [test_user["user_id"]]},
        headers=auth_headers(test_user["token"]),
    )
    assert resp.status_code == 403, f"Expected 403, got {resp.status_code}: {resp.text}"


def test_revoke_empty_principals_rejected(root_token):
    """Empty principals list returns 400."""
    resp = requests.post(
        f"{URL_GLOBAL}/permissions/{PERM_CREATE_GROUPS}/revoke",
        json={},
        headers=auth_headers(root_token),
    )
    assert resp.status_code == 400, f"Expected 400, got {resp.status_code}: {resp.text}"


# ───────────────────────────── Debug /access endpoint ─────────────────────


def test_debug_access_returns_principals_and_super_permissions(root_token):
    """Root has adm_godmode and appears in its own principals list."""
    resp = requests.get(
        f"{URL_DEBUG}/access",
        params={"user": "u_root"},
        headers=auth_headers(root_token),
    )
    assert resp.status_code == 200, resp.text
    body = resp.json()
    assert body["user"] == "u_root"
    assert "u_root" in body["principals"]
    assert "adm_godmode" in body["super_permissions"]


def test_debug_access_missing_user_param(root_token):
    """Omitting 'user' returns 400."""
    resp = requests.get(f"{URL_DEBUG}/access", headers=auth_headers(root_token))
    assert resp.status_code == 400, f"Expected 400, got {resp.status_code}: {resp.text}"


def test_debug_access_with_valid_resource(root_token):
    """resource=users/u_root shows resource_found and resource_acl."""
    resp = requests.get(
        f"{URL_DEBUG}/access",
        params={"user": "u_root", "resource": "users/u_root"},
        headers=auth_headers(root_token),
    )
    assert resp.status_code == 200, resp.text
    body = resp.json()
    assert body["resource_found"] is True
    assert "resource_acl" in body
    assert "matching_acl_entries" in body


def test_debug_access_with_missing_resource(root_token):
    """Non-existent resource key returns resource_found=false."""
    resp = requests.get(
        f"{URL_DEBUG}/access",
        params={"user": "u_root", "resource": "users/u_definitely_nonexistent_xyz"},
        headers=auth_headers(root_token),
    )
    assert resp.status_code == 200, resp.text
    assert resp.json()["resource_found"] is False


def test_debug_access_godmode_bypasses_acl(root_token):
    """Root (godmode) is always granted even for FETCH (bit 1)."""
    resp = requests.get(
        f"{URL_DEBUG}/access",
        params={"user": "u_root", "resource": "users/u_root", "permission": "1"},
        headers=auth_headers(root_token),
    )
    assert resp.status_code == 200, resp.text
    ar = resp.json()["access_result"]
    assert ar["granted"] is True
    assert ar["reason"] == "godmode bypass"


def test_debug_access_invalid_permission_bits(root_token):
    """Non-numeric 'permission' returns 400."""
    resp = requests.get(
        f"{URL_DEBUG}/access",
        params={"user": "u_root", "resource": "users/u_root", "permission": "not_a_number"},
        headers=auth_headers(root_token),
    )
    assert resp.status_code == 400, f"Expected 400, got {resp.status_code}: {resp.text}"


def test_debug_access_invalid_resource_format(root_token):
    """resource without '/' separator returns 400."""
    resp = requests.get(
        f"{URL_DEBUG}/access",
        params={"user": "u_root", "resource": "justakeynoformat"},
        headers=auth_headers(root_token),
    )
    assert resp.status_code == 400, f"Expected 400, got {resp.status_code}: {resp.text}"


def test_debug_access_requires_godmode(test_user):
    """Regular user gets 401 from godmode_middleware."""
    resp = requests.get(
        f"{URL_DEBUG}/access",
        params={"user": test_user["user_id"]},
        headers=auth_headers(test_user["token"]),
    )
    assert resp.status_code == 401, f"Expected 401, got {resp.status_code}: {resp.text}"


def test_debug_access_unauthenticated():
    """No token returns 401."""
    resp = requests.get(f"{URL_DEBUG}/access", params={"user": "u_root"})
    assert resp.status_code == 401, f"Expected 401, got {resp.status_code}: {resp.text}"
