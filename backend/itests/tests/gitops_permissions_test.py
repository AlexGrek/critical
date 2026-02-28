import pytest
import requests
import random


BASE = "http://localhost:3742/api"
URL_REGISTER = f"{BASE}/v1/register"
URL_LOGIN = f"{BASE}/v1/login"
URL_GLOBAL = f"{BASE}/v1/global"


def auth_headers(token: str) -> dict:
    return {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json",
    }


@pytest.fixture(scope="module")
def admin_token():
    """Login as root (has all super-permissions)."""
    resp = requests.post(URL_LOGIN, json={"user": "root", "password": "changeme"})
    assert resp.status_code == 200, f"Root login failed: {resp.text}"
    return resp.json()["token"]


@pytest.fixture(scope="module")
def regular_user():
    """Register a regular user with no special permissions."""
    num = random.randint(100000, 999999)
    user = f"perm_itest_{num}"
    password = f"perm_itest_{num}"

    requests.post(URL_REGISTER, json={"user": user, "password": password})
    resp = requests.post(URL_LOGIN, json={"user": user, "password": password})
    assert resp.status_code == 200, f"Login failed: {resp.text}"
    return {
        "user_id": f"u_{user}",
        "token": resp.json()["token"],
    }


# ------------------- Users & Groups: read by all, write by ADM_USER_MANAGER --------------------


def test_regular_user_can_list_users(regular_user):
    """All authenticated users can list users."""
    resp = requests.get(f"{URL_GLOBAL}/users", headers=auth_headers(regular_user["token"]))
    assert resp.status_code == 200
    assert "items" in resp.json()


def test_regular_user_can_get_user(regular_user):
    """All authenticated users can get a specific user."""
    resp = requests.get(
        f"{URL_GLOBAL}/users/{regular_user['user_id']}",
        headers=auth_headers(regular_user["token"]),
    )
    assert resp.status_code == 200
    assert resp.json()["id"] == regular_user["user_id"]


def test_regular_user_cannot_create_user(regular_user):
    """Regular users cannot create users (requires ADM_USER_MANAGER)."""
    num = random.randint(100000, 999999)
    user_id = f"u_blocked_{num}"

    resp = requests.post(
        f"{URL_GLOBAL}/users",
        json={
            "id": user_id,
            "password": "test123",
            "personal": {"name": "Blocked", "gender": "", "job_title": "", "manager": None},

        },
        headers=auth_headers(regular_user["token"]),
    )
    assert resp.status_code == 404, f"Expected 404, got {resp.status_code}: {resp.text}"


def test_admin_can_create_and_delete_user(admin_token):
    """Admin (root) can create and delete users."""
    num = random.randint(100000, 999999)
    user_id = f"u_admin_test_{num}"
    headers = auth_headers(admin_token)

    # Create
    resp = requests.post(
        f"{URL_GLOBAL}/users",
        json={
            "id": user_id,
            "password": "test123",
            "personal": {"name": "Admin Test", "gender": "", "job_title": "", "manager": None},

        },
        headers=headers,
    )
    assert resp.status_code == 201, f"Expected 201, got {resp.status_code}: {resp.text}"

    # Delete
    resp = requests.delete(f"{URL_GLOBAL}/users/{user_id}", headers=headers)
    assert resp.status_code == 204


def test_regular_user_cannot_delete_user(regular_user, admin_token):
    """Regular users cannot delete users."""
    num = random.randint(100000, 999999)
    user_id = f"u_del_test_{num}"
    admin_headers = auth_headers(admin_token)

    # Admin creates a user
    resp = requests.post(
        f"{URL_GLOBAL}/users",
        json={
            "id": user_id,
            "password": "test123",
            "personal": {"name": "Del Test", "gender": "", "job_title": "", "manager": None},

        },
        headers=admin_headers,
    )
    assert resp.status_code == 201

    # Regular user tries to delete — should get 404
    resp = requests.delete(
        f"{URL_GLOBAL}/users/{user_id}",
        headers=auth_headers(regular_user["token"]),
    )
    assert resp.status_code == 404, f"Expected 404, got {resp.status_code}: {resp.text}"

    # Cleanup by admin
    requests.delete(f"{URL_GLOBAL}/users/{user_id}", headers=admin_headers)


def test_regular_user_cannot_update_user(regular_user, admin_token):
    """Regular users cannot update users."""
    num = random.randint(100000, 999999)
    user_id = f"u_upd_test_{num}"
    admin_headers = auth_headers(admin_token)

    # Admin creates a user
    resp = requests.post(
        f"{URL_GLOBAL}/users",
        json={
            "id": user_id,
            "password": "test123",
            "personal": {"name": "Update Test", "gender": "", "job_title": "", "manager": None},

        },
        headers=admin_headers,
    )
    assert resp.status_code == 201

    # Regular user tries to update — should get 404
    resp = requests.put(
        f"{URL_GLOBAL}/users/{user_id}",
        json={
            "id": user_id,
            "personal": {"name": "Hacked", "gender": "", "job_title": "", "manager": None},

        },
        headers=auth_headers(regular_user["token"]),
    )
    assert resp.status_code == 404, f"Expected 404, got {resp.status_code}: {resp.text}"

    # Cleanup by admin
    requests.delete(f"{URL_GLOBAL}/users/{user_id}", headers=admin_headers)
