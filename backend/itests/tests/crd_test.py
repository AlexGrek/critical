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
def godmode_token():
    """Login as root (has ADM_GODMODE — required for CRD writes)."""
    resp = requests.post(URL_LOGIN, json={"user": "root", "password": "changeme"})
    assert resp.status_code == 200, f"Root login failed: {resp.text}"
    return resp.json()["token"]


@pytest.fixture(scope="module")
def regular_user():
    """Register a regular user with no special permissions."""
    num = random.randint(100000, 999999)
    user = f"crd_itest_{num}"
    password = f"crd_itest_{num}"

    requests.post(URL_REGISTER, json={"user": user, "password": password})
    resp = requests.post(URL_LOGIN, json={"user": user, "password": password})
    assert resp.status_code == 200, f"Regular user login failed: {resp.text}"
    return {
        "user_id": f"u_{user}",
        "token": resp.json()["token"],
    }


def _make_crd_payload(crd_id: str) -> dict:
    return {
        "id": crd_id,
        "scope": "project",
        "acl_mode": "inherit",
        "nouns": {"singular": "deployment", "plural": "deployments"},
        "fields": {
            "image": {"type": "string", "required": True},
            "replicas": {"type": "int", "default": 1},
        },
        "description": "Container deployment",
    }


# ── Create ────────────────────────────────────────────────────────────────────


def test_godmode_can_create_crd(godmode_token):
    """Godmode user can POST a new CRD and receives 201 with the id."""
    num = random.randint(100000, 999999)
    crd_id = f"deployments_{num}"
    headers = auth_headers(godmode_token)

    resp = requests.post(
        f"{URL_GLOBAL}/crds",
        json=_make_crd_payload(crd_id),
        headers=headers,
    )
    assert resp.status_code == 201, f"Expected 201, got {resp.status_code}: {resp.text}"
    assert resp.json()["id"] == crd_id

    # Cleanup
    requests.delete(f"{URL_GLOBAL}/crds/{crd_id}", headers=headers)


# ── Read (GET single + list) ──────────────────────────────────────────────────


def test_godmode_can_get_crd(godmode_token):
    """Godmode user can GET a CRD and fields round-trip correctly."""
    num = random.randint(100000, 999999)
    crd_id = f"deployments_{num}"
    headers = auth_headers(godmode_token)

    # Create
    resp = requests.post(
        f"{URL_GLOBAL}/crds",
        json=_make_crd_payload(crd_id),
        headers=headers,
    )
    assert resp.status_code == 201, f"Create failed: {resp.text}"

    # GET and verify round-trip
    resp = requests.get(f"{URL_GLOBAL}/crds/{crd_id}", headers=headers)
    assert resp.status_code == 200, f"Expected 200, got {resp.status_code}: {resp.text}"
    data = resp.json()
    assert data["id"] == crd_id
    assert data["scope"] == "project"
    assert data["acl_mode"] == "inherit"
    assert data["description"] == "Container deployment"
    assert data["nouns"]["singular"] == "deployment"
    assert data["nouns"]["plural"] == "deployments"
    assert "image" in data["fields"]
    assert data["fields"]["image"]["type"] == "string"
    assert data["fields"]["replicas"]["default"] == 1

    # Cleanup
    requests.delete(f"{URL_GLOBAL}/crds/{crd_id}", headers=headers)


def test_godmode_can_list_crds(godmode_token):
    """Godmode user can list CRDs and the created CRD appears in the list."""
    num = random.randint(100000, 999999)
    crd_id = f"deployments_{num}"
    headers = auth_headers(godmode_token)

    # Create
    resp = requests.post(
        f"{URL_GLOBAL}/crds",
        json=_make_crd_payload(crd_id),
        headers=headers,
    )
    assert resp.status_code == 201, f"Create failed: {resp.text}"

    # List
    resp = requests.get(f"{URL_GLOBAL}/crds", headers=headers)
    assert resp.status_code == 200, f"Expected 200, got {resp.status_code}: {resp.text}"
    items = resp.json()["items"]
    ids = [item["id"] for item in items]
    assert crd_id in ids, f"Created CRD '{crd_id}' not found in list: {ids}"

    # Cleanup
    requests.delete(f"{URL_GLOBAL}/crds/{crd_id}", headers=headers)


# ── Regular user read access ──────────────────────────────────────────────────


def test_regular_user_can_list_crds(regular_user, godmode_token):
    """All authenticated users can list CRDs (read is open to any authed user)."""
    num = random.randint(100000, 999999)
    crd_id = f"deployments_{num}"
    god_headers = auth_headers(godmode_token)

    # Godmode creates the CRD
    resp = requests.post(
        f"{URL_GLOBAL}/crds",
        json=_make_crd_payload(crd_id),
        headers=god_headers,
    )
    assert resp.status_code == 201, f"Create failed: {resp.text}"

    # Regular user lists — should succeed
    resp = requests.get(
        f"{URL_GLOBAL}/crds",
        headers=auth_headers(regular_user["token"]),
    )
    assert resp.status_code == 200, f"Expected 200, got {resp.status_code}: {resp.text}"
    assert "items" in resp.json()

    # Cleanup
    requests.delete(f"{URL_GLOBAL}/crds/{crd_id}", headers=god_headers)


def test_regular_user_can_get_single_crd(regular_user, godmode_token):
    """All authenticated users can GET a single CRD by id."""
    num = random.randint(100000, 999999)
    crd_id = f"deployments_{num}"
    god_headers = auth_headers(godmode_token)

    # Godmode creates the CRD
    resp = requests.post(
        f"{URL_GLOBAL}/crds",
        json=_make_crd_payload(crd_id),
        headers=god_headers,
    )
    assert resp.status_code == 201, f"Create failed: {resp.text}"

    # Regular user fetches single — should succeed
    resp = requests.get(
        f"{URL_GLOBAL}/crds/{crd_id}",
        headers=auth_headers(regular_user["token"]),
    )
    assert resp.status_code == 200, f"Expected 200, got {resp.status_code}: {resp.text}"
    assert resp.json()["id"] == crd_id

    # Cleanup
    requests.delete(f"{URL_GLOBAL}/crds/{crd_id}", headers=god_headers)


# ── Regular user write rejection ──────────────────────────────────────────────


def test_regular_user_cannot_create_crd(regular_user):
    """Regular users cannot create CRDs (requires ADM_GODMODE)."""
    num = random.randint(100000, 999999)
    crd_id = f"deployments_{num}"

    resp = requests.post(
        f"{URL_GLOBAL}/crds",
        json=_make_crd_payload(crd_id),
        headers=auth_headers(regular_user["token"]),
    )
    assert resp.status_code == 403, f"Expected 403, got {resp.status_code}: {resp.text}"


def test_regular_user_cannot_update_crd(regular_user, godmode_token):
    """Regular users cannot PUT/update CRDs (requires ADM_GODMODE)."""
    num = random.randint(100000, 999999)
    crd_id = f"deployments_{num}"
    god_headers = auth_headers(godmode_token)

    # Godmode creates the CRD
    resp = requests.post(
        f"{URL_GLOBAL}/crds",
        json=_make_crd_payload(crd_id),
        headers=god_headers,
    )
    assert resp.status_code == 201, f"Create failed: {resp.text}"

    # Regular user tries to update — should be rejected
    payload = _make_crd_payload(crd_id)
    payload["description"] = "Unauthorized modification"
    resp = requests.put(
        f"{URL_GLOBAL}/crds/{crd_id}",
        json=payload,
        headers=auth_headers(regular_user["token"]),
    )
    assert resp.status_code in (403, 404), (
        f"Expected 403 or 404, got {resp.status_code}: {resp.text}"
    )

    # Cleanup
    requests.delete(f"{URL_GLOBAL}/crds/{crd_id}", headers=god_headers)


def test_regular_user_cannot_delete_crd(regular_user, godmode_token):
    """Regular users cannot DELETE CRDs (requires ADM_GODMODE)."""
    num = random.randint(100000, 999999)
    crd_id = f"deployments_{num}"
    god_headers = auth_headers(godmode_token)

    # Godmode creates the CRD
    resp = requests.post(
        f"{URL_GLOBAL}/crds",
        json=_make_crd_payload(crd_id),
        headers=god_headers,
    )
    assert resp.status_code == 201, f"Create failed: {resp.text}"

    # Regular user tries to delete — should be rejected
    resp = requests.delete(
        f"{URL_GLOBAL}/crds/{crd_id}",
        headers=auth_headers(regular_user["token"]),
    )
    assert resp.status_code in (403, 404), (
        f"Expected 403 or 404, got {resp.status_code}: {resp.text}"
    )

    # Cleanup
    requests.delete(f"{URL_GLOBAL}/crds/{crd_id}", headers=god_headers)


# ── Delete ────────────────────────────────────────────────────────────────────


def test_godmode_can_delete_crd(godmode_token):
    """Godmode user can DELETE a CRD; subsequent GET returns 404."""
    num = random.randint(100000, 999999)
    crd_id = f"deployments_{num}"
    headers = auth_headers(godmode_token)

    # Create
    resp = requests.post(
        f"{URL_GLOBAL}/crds",
        json=_make_crd_payload(crd_id),
        headers=headers,
    )
    assert resp.status_code == 201, f"Create failed: {resp.text}"

    # Delete
    resp = requests.delete(f"{URL_GLOBAL}/crds/{crd_id}", headers=headers)
    assert resp.status_code == 204, f"Expected 204, got {resp.status_code}: {resp.text}"

    # Verify gone
    resp = requests.get(f"{URL_GLOBAL}/crds/{crd_id}", headers=headers)
    assert resp.status_code == 404, f"Expected 404 after delete, got {resp.status_code}: {resp.text}"


# ── Input validation ──────────────────────────────────────────────────────────


def test_invalid_crd_id_with_spaces_rejected(godmode_token):
    """CRD id containing spaces must be rejected with a 4xx error."""
    headers = auth_headers(godmode_token)

    resp = requests.post(
        f"{URL_GLOBAL}/crds",
        json=_make_crd_payload("invalid id with spaces"),
        headers=headers,
    )
    assert 400 <= resp.status_code < 500, (
        f"Expected 4xx for id with spaces, got {resp.status_code}: {resp.text}"
    )


def test_invalid_crd_id_starting_with_digit_rejected(godmode_token):
    """CRD id starting with a digit must be rejected with a 4xx error."""
    headers = auth_headers(godmode_token)

    resp = requests.post(
        f"{URL_GLOBAL}/crds",
        json=_make_crd_payload("1invalid_starts_with_digit"),
        headers=headers,
    )
    assert 400 <= resp.status_code < 500, (
        f"Expected 4xx for id starting with digit, got {resp.status_code}: {resp.text}"
    )
