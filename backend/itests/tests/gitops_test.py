import pytest
import requests
import random


BASE = "http://localhost:3742/api"
URL_LOGIN = f"{BASE}/login"
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
