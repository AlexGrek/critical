import pytest
import requests
import random


BASE = "http://localhost:3742/api"
URL_REGISTER = f"{BASE}/register"
URL_LOGIN = f"{BASE}/login"
URL_GLOBAL = f"{BASE}/v1/global"


def auth_headers(token: str) -> dict:
    return {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json",
    }


def _make_task_body(task_id: str, title: str, *, state="backlog", priority="medium"):
    return {
        "id": task_id,
        "title": title,
        "description": "Test task description",
        "state": state,
        "priority": priority,
        "severity": None,
        "assigned_to": None,
        "mentioned": [],
        "meta": {},
    }


@pytest.fixture(scope="module")
def project_user():
    """Register a user who gets USR_CREATE_PROJECTS by default."""
    num = random.randint(100000, 999999)
    user = f"task_test_{num}"
    password = f"task_test_{num}"

    requests.post(URL_REGISTER, json={"user": user, "password": password})
    resp = requests.post(URL_LOGIN, json={"user": user, "password": password})
    assert resp.status_code == 200, f"Login failed: {resp.text}"
    return {
        "user_id": f"u_{user}",
        "token": resp.json()["token"],
    }


@pytest.fixture(scope="module")
def other_user():
    """Register a second user for ACL testing."""
    num = random.randint(100000, 999999)
    user = f"task_other_{num}"
    password = f"task_other_{num}"

    requests.post(URL_REGISTER, json={"user": user, "password": password})
    resp = requests.post(URL_LOGIN, json={"user": user, "password": password})
    assert resp.status_code == 200, f"Login failed: {resp.text}"
    return {
        "user_id": f"u_{user}",
        "token": resp.json()["token"],
    }


# ------------------- CRUD --------------------


def test_create_and_get_task(project_user):
    """Create a task via POST /global/tasks, verify via GET."""
    num = random.randint(100000, 999999)
    task_id = f"t_task_{num}"
    headers = auth_headers(project_user["token"])

    # Create
    resp = requests.post(
        f"{URL_GLOBAL}/tasks",
        json=_make_task_body(task_id, "My First Task"),
        headers=headers,
    )
    assert resp.status_code == 201, f"Expected 201, got {resp.status_code}: {resp.text}"
    assert resp.json()["id"] == task_id

    # Get
    resp = requests.get(f"{URL_GLOBAL}/tasks/{task_id}", headers=headers)
    assert resp.status_code == 200
    data = resp.json()
    assert data["id"] == task_id
    assert data["title"] == "My First Task"
    assert data["state"] == "backlog"
    assert data["priority"] == "medium"

    # Cleanup
    resp = requests.delete(f"{URL_GLOBAL}/tasks/{task_id}", headers=headers)
    assert resp.status_code == 204


def test_list_tasks(project_user):
    """Tasks appear in list responses with brief fields only."""
    num = random.randint(100000, 999999)
    task_id = f"t_list_{num}"
    headers = auth_headers(project_user["token"])

    resp = requests.post(
        f"{URL_GLOBAL}/tasks",
        json=_make_task_body(task_id, "List Test Task"),
        headers=headers,
    )
    assert resp.status_code == 201

    resp = requests.get(f"{URL_GLOBAL}/tasks", headers=headers)
    assert resp.status_code == 200
    ids = [item["id"] for item in resp.json()["items"]]
    assert task_id in ids

    # Cleanup
    requests.delete(f"{URL_GLOBAL}/tasks/{task_id}", headers=headers)


def test_update_task_state(project_user):
    """Creator can update a task's state."""
    num = random.randint(100000, 999999)
    task_id = f"t_upd_{num}"
    headers = auth_headers(project_user["token"])

    resp = requests.post(
        f"{URL_GLOBAL}/tasks",
        json=_make_task_body(task_id, "State Update Task"),
        headers=headers,
    )
    assert resp.status_code == 201

    # Fetch current doc and update state
    resp = requests.get(f"{URL_GLOBAL}/tasks/{task_id}", headers=headers)
    data = resp.json()
    data["state"] = "in_progress"

    resp = requests.put(f"{URL_GLOBAL}/tasks/{task_id}", json=data, headers=headers)
    assert resp.status_code == 200

    resp = requests.get(f"{URL_GLOBAL}/tasks/{task_id}", headers=headers)
    assert resp.json()["state"] == "in_progress"

    # Cleanup
    requests.delete(f"{URL_GLOBAL}/tasks/{task_id}", headers=headers)


def test_task_acl_enforcement(project_user, other_user):
    """Users not in a task's ACL cannot read it."""
    num = random.randint(100000, 999999)
    task_id = f"t_acl_{num}"
    creator_headers = auth_headers(project_user["token"])
    other_headers = auth_headers(other_user["token"])

    # Creator creates the task (gets ROOT ACL injected)
    resp = requests.post(
        f"{URL_GLOBAL}/tasks",
        json=_make_task_body(task_id, "Private Task"),
        headers=creator_headers,
    )
    assert resp.status_code == 201

    # Other user cannot read it
    resp = requests.get(f"{URL_GLOBAL}/tasks/{task_id}", headers=other_headers)
    assert resp.status_code == 404, f"Expected 404 (denied), got {resp.status_code}: {resp.text}"

    # Other user cannot see it in the list
    resp = requests.get(f"{URL_GLOBAL}/tasks", headers=other_headers)
    ids = [item["id"] for item in resp.json().get("items", [])]
    assert task_id not in ids

    # Cleanup
    requests.delete(f"{URL_GLOBAL}/tasks/{task_id}", headers=creator_headers)


def test_task_id_prefixed(project_user):
    """Tasks auto-receive the t_ prefix even when created without it."""
    num = random.randint(100000, 999999)
    raw_id = f"nopfx_{num}"
    expected_id = f"t_{raw_id}"
    headers = auth_headers(project_user["token"])

    resp = requests.post(
        f"{URL_GLOBAL}/tasks",
        json=_make_task_body(raw_id, "Prefix Test"),
        headers=headers,
    )
    assert resp.status_code == 201, f"Expected 201, got {resp.status_code}: {resp.text}"
    assert resp.json()["id"] == expected_id

    # Cleanup
    requests.delete(f"{URL_GLOBAL}/tasks/{expected_id}", headers=headers)
