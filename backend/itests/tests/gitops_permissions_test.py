import pytest
import requests
import random
from datetime import datetime, timezone


BASE = "http://localhost:3742/api"
URL_REGISTER = f"{BASE}/register"
URL_LOGIN = f"{BASE}/login"
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
            "meta": {},
            "state": "active",
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
            "meta": {},
            "state": "active",
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
            "meta": {},
            "state": "active",
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
            "meta": {},
            "state": "active",
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
            "meta": {},
            "state": "active",
        },
        headers=auth_headers(regular_user["token"]),
    )
    assert resp.status_code == 404, f"Expected 404, got {resp.status_code}: {resp.text}"

    # Cleanup by admin
    requests.delete(f"{URL_GLOBAL}/users/{user_id}", headers=admin_headers)


# ------------------- Projects: ADM_PROJECT_MANAGER or ACL --------------------


def _make_project_body(project_id, acl_list=None):
    """Helper to build a project JSON body."""
    return {
        "id": project_id,
        "name": "Test Project",
        "acl": {
            "list": acl_list or [],
            "last_mod_date": datetime.now(timezone.utc).isoformat(),
        },
    }


def test_regular_user_can_create_project_and_gets_root_acl(regular_user):
    """Regular users can create projects; backend auto-grants ROOT ACL to creator."""
    project_id = f"p_proj_{random.randint(100000, 999999)}"
    headers = auth_headers(regular_user["token"])

    # Create with empty ACL — backend should inject creator with ROOT
    resp = requests.post(
        f"{URL_GLOBAL}/projects",
        json=_make_project_body(project_id),
        headers=headers,
    )
    assert resp.status_code == 201, f"Expected 201, got {resp.status_code}: {resp.text}"

    # Creator can read their own project
    resp = requests.get(f"{URL_GLOBAL}/projects/{project_id}", headers=headers)
    assert resp.status_code == 200
    data = resp.json()
    assert data["id"] == project_id

    # Verify creator is in ACL
    acl_principals = []
    for entry in data.get("acl", {}).get("list", []):
        acl_principals.extend(entry.get("principals", []))
    assert regular_user["user_id"] in acl_principals, (
        f"Creator {regular_user['user_id']} should be in project ACL, got {acl_principals}"
    )

    # Creator can update their own project (has ROOT → includes WRITE)
    updated = _make_project_body(project_id, data["acl"]["list"])
    updated["name"] = "Updated By Creator"
    resp = requests.put(f"{URL_GLOBAL}/projects/{project_id}", json=updated, headers=headers)
    assert resp.status_code == 200

    # Creator can delete their own project
    resp = requests.delete(f"{URL_GLOBAL}/projects/{project_id}", headers=headers)
    assert resp.status_code == 204


def test_project_no_acl_hidden_from_regular_user(regular_user, admin_token):
    """Projects with empty ACL are invisible to regular users."""
    project_id = f"p_proj_{random.randint(100000, 999999)}"
    admin_headers = auth_headers(admin_token)

    # Admin creates project with empty ACL
    resp = requests.post(
        f"{URL_GLOBAL}/projects",
        json=_make_project_body(project_id),
        headers=admin_headers,
    )
    assert resp.status_code == 201

    # Regular user cannot get it
    resp = requests.get(
        f"{URL_GLOBAL}/projects/{project_id}",
        headers=auth_headers(regular_user["token"]),
    )
    assert resp.status_code == 404

    # Regular user cannot see it in list
    resp = requests.get(f"{URL_GLOBAL}/projects", headers=auth_headers(regular_user["token"]))
    assert resp.status_code == 200
    ids = [item["id"] for item in resp.json()["items"]]
    assert project_id not in ids

    # Admin can still see it
    resp = requests.get(f"{URL_GLOBAL}/projects/{project_id}", headers=admin_headers)
    assert resp.status_code == 200

    # Cleanup
    requests.delete(f"{URL_GLOBAL}/projects/{project_id}", headers=admin_headers)


def test_project_acl_read_grants_access(regular_user, admin_token):
    """Projects with ACL READ grant read access to the user."""
    project_id = f"p_proj_{random.randint(100000, 999999)}"
    admin_headers = auth_headers(admin_token)

    # Admin creates project with ACL granting READ to regular user
    # READ = FETCH | LIST | NOTIFY = 1 | 2 | 4 = 7
    acl_list = [{"permissions": 7, "principals": [regular_user["user_id"]]}]
    resp = requests.post(
        f"{URL_GLOBAL}/projects",
        json=_make_project_body(project_id, acl_list),
        headers=admin_headers,
    )
    assert resp.status_code == 201

    # Regular user can get it
    resp = requests.get(
        f"{URL_GLOBAL}/projects/{project_id}",
        headers=auth_headers(regular_user["token"]),
    )
    assert resp.status_code == 200
    assert resp.json()["id"] == project_id

    # Regular user can see it in list
    resp = requests.get(f"{URL_GLOBAL}/projects", headers=auth_headers(regular_user["token"]))
    assert resp.status_code == 200
    ids = [item["id"] for item in resp.json()["items"]]
    assert project_id in ids

    # Regular user cannot update it (only has READ, not WRITE)
    resp = requests.put(
        f"{URL_GLOBAL}/projects/{project_id}",
        json=_make_project_body(project_id, acl_list),
        headers=auth_headers(regular_user["token"]),
    )
    assert resp.status_code == 404, f"Expected 404, got {resp.status_code}: {resp.text}"

    # Cleanup
    requests.delete(f"{URL_GLOBAL}/projects/{project_id}", headers=admin_headers)


def test_project_acl_write_grants_full_access(regular_user, admin_token):
    """Projects with ACL WRITE grant read+write access to the user."""
    project_id = f"p_proj_{random.randint(100000, 999999)}"
    admin_headers = auth_headers(admin_token)

    # WRITE = CREATE | MODIFY | READ = 8 | 16 | 7 = 31
    acl_list = [{"permissions": 31, "principals": [regular_user["user_id"]]}]
    resp = requests.post(
        f"{URL_GLOBAL}/projects",
        json=_make_project_body(project_id, acl_list),
        headers=admin_headers,
    )
    assert resp.status_code == 201

    # Regular user can read it
    resp = requests.get(
        f"{URL_GLOBAL}/projects/{project_id}",
        headers=auth_headers(regular_user["token"]),
    )
    assert resp.status_code == 200

    # Regular user can update it
    updated = _make_project_body(project_id, acl_list)
    updated["name"] = "Updated By Regular User"
    resp = requests.put(
        f"{URL_GLOBAL}/projects/{project_id}",
        json=updated,
        headers=auth_headers(regular_user["token"]),
    )
    assert resp.status_code == 200

    # Regular user can delete it
    resp = requests.delete(
        f"{URL_GLOBAL}/projects/{project_id}",
        headers=auth_headers(regular_user["token"]),
    )
    assert resp.status_code == 204
