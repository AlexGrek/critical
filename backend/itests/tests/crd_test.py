import pytest
import random
import requests


BASE = "http://localhost:3742/api"
URL_REGISTER = f"{BASE}/v1/register"
URL_LOGIN = f"{BASE}/v1/login"
URL_GLOBAL = f"{BASE}/v1/global"
URL_DEBUG = f"{BASE}/v1/debug"


def auth_headers(token: str) -> dict:
    return {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json",
    }


# ── Debug client ───────────────────────────────────────────────────────────────


class DebugClient:
    def __init__(self, root_token: str):
        self._headers = auth_headers(root_token)

    def dump(self, collection: str) -> list[dict]:
        resp = requests.get(f"{URL_DEBUG}/collections/{collection}", headers=self._headers)
        assert resp.status_code == 200, f"debug dump of '{collection}' failed: {resp.text}"
        return resp.json()["documents"]

    def find_one(self, collection: str, **matchers) -> dict | None:
        for doc in self.dump(collection):
            if all(doc.get(k) == v for k, v in matchers.items()):
                return doc
        return None

    def assert_soft_deleted(self, collection: str, key: str) -> dict:
        docs = self.dump(collection)
        for doc in docs:
            if doc.get("_key") == key:
                assert "deletion" in doc and doc["deletion"] is not None, (
                    f"Document {key} in '{collection}' exists but is NOT soft-deleted.\n"
                    f"Document: {doc}"
                )
                return doc
        raise AssertionError(
            f"Document {key} not found in '{collection}' at all.\n"
            f"Keys present: {[d.get('_key') for d in docs]}"
        )


# ── Fixtures ───────────────────────────────────────────────────────────────────


@pytest.fixture(scope="module")
def godmode_token():
    """Login as root (has ADM_GODMODE — required for CRD writes)."""
    resp = requests.post(URL_LOGIN, json={"user": "root", "password": "changeme"})
    assert resp.status_code == 200, f"Root login failed: {resp.text}"
    token = resp.json()["token"]
    print(f"\n[DEBUG] godmode_token: {token}")
    return token


@pytest.fixture(scope="module")
def debug(godmode_token):
    return DebugClient(godmode_token)


@pytest.fixture(scope="module")
def regular_user():
    """Register a fresh user with no special permissions."""
    num = random.randint(100000, 999999)
    user = f"crd_itest_{num}"
    requests.post(URL_REGISTER, json={"user": user, "password": user})
    resp = requests.post(URL_LOGIN, json={"user": user, "password": user})
    assert resp.status_code == 200, f"Regular user login failed: {resp.text}"
    return {"user_id": f"u_{user}", "token": resp.json()["token"]}


@pytest.fixture(scope="module")
def shared_crd(godmode_token):
    """Create a single CRD shared across read-only tests; deleted after module."""
    num = random.randint(100000, 999999)
    crd_id = f"shared-crd-{num}"
    headers = auth_headers(godmode_token)
    resp = requests.post(
        f"{URL_GLOBAL}/crds",
        json=_make_crd_payload(crd_id),
        headers=headers,
    )
    assert resp.status_code == 201, f"Shared CRD creation failed: {resp.text}"
    yield crd_id
    requests.delete(f"{URL_GLOBAL}/crds/{crd_id}", headers=headers)


# ── Helpers ────────────────────────────────────────────────────────────────────


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


# ── Create ─────────────────────────────────────────────────────────────────────


def test_godmode_can_create_crd(godmode_token):
    """Godmode user can POST a new CRD and receives 201 with the correct id."""
    num = random.randint(100000, 999999)
    crd_id = f"deployments-{num}"
    headers = auth_headers(godmode_token)

    resp = requests.post(f"{URL_GLOBAL}/crds", json=_make_crd_payload(crd_id), headers=headers)
    assert resp.status_code == 201, f"Expected 201, got {resp.status_code}: {resp.text}"

    data = resp.json()
    assert data["id"] == crd_id
    assert "state" in data, f"Create response missing 'state': {data}"
    assert "created_at" in data["state"], f"Missing state.created_at: {data['state']}"

    requests.delete(f"{URL_GLOBAL}/crds/{crd_id}", headers=headers)


def test_create_crd_response_shape(godmode_token):
    """Create response includes all expected top-level fields."""
    num = random.randint(100000, 999999)
    crd_id = f"shape-test-{num}"
    headers = auth_headers(godmode_token)

    resp = requests.post(f"{URL_GLOBAL}/crds", json=_make_crd_payload(crd_id), headers=headers)
    assert resp.status_code == 201, resp.text

    data = resp.json()
    for field in ("id", "scope", "acl_mode", "nouns", "fields", "description", "state", "hash_code"):
        assert field in data, f"Response missing expected field '{field}': {data}"

    requests.delete(f"{URL_GLOBAL}/crds/{crd_id}", headers=headers)


def test_regular_user_cannot_create_crd(regular_user):
    """Create is forbidden (403) for users without ADM_GODMODE."""
    num = random.randint(100000, 999999)
    resp = requests.post(
        f"{URL_GLOBAL}/crds",
        json=_make_crd_payload(f"deployments-{num}"),
        headers=auth_headers(regular_user["token"]),
    )
    assert resp.status_code == 403, f"Expected 403, got {resp.status_code}: {resp.text}"


def test_unauthenticated_cannot_create_crd():
    """Unauthenticated request to create returns 401."""
    num = random.randint(100000, 999999)
    resp = requests.post(
        f"{URL_GLOBAL}/crds",
        json=_make_crd_payload(f"deployments-{num}"),
    )
    assert resp.status_code == 401, f"Expected 401, got {resp.status_code}: {resp.text}"


# ── Read (GET single + list) ───────────────────────────────────────────────────


def test_godmode_can_get_crd(shared_crd, godmode_token):
    """Godmode user can GET a CRD; fields round-trip correctly."""
    resp = requests.get(f"{URL_GLOBAL}/crds/{shared_crd}", headers=auth_headers(godmode_token))
    assert resp.status_code == 200, f"Expected 200, got {resp.status_code}: {resp.text}"

    data = resp.json()
    assert data["id"] == shared_crd
    assert data["scope"] == "project"
    assert data["acl_mode"] == "inherit"
    assert data["description"] == "Container deployment"
    assert data["nouns"]["singular"] == "deployment"
    assert data["nouns"]["plural"] == "deployments"
    assert "image" in data["fields"]
    assert data["fields"]["image"]["type"] == "string"
    assert data["fields"]["image"]["required"] is True
    assert data["fields"]["replicas"]["default"] == 1


def test_godmode_can_list_crds(shared_crd, godmode_token):
    """Godmode user can list CRDs; the shared CRD appears in the list."""
    resp = requests.get(f"{URL_GLOBAL}/crds", headers=auth_headers(godmode_token))
    assert resp.status_code == 200, f"Expected 200, got {resp.status_code}: {resp.text}"

    body = resp.json()
    assert "items" in body, f"Response missing 'items': {body}"
    ids = [item["id"] for item in body["items"]]
    assert shared_crd in ids, f"Shared CRD '{shared_crd}' not found in list: {ids}"


def test_list_returns_brief_items(shared_crd, godmode_token):
    """List items contain id and labels but NOT full fields like hash_code or annotations."""
    resp = requests.get(f"{URL_GLOBAL}/crds", headers=auth_headers(godmode_token))
    assert resp.status_code == 200, resp.text

    items = resp.json()["items"]
    assert len(items) > 0, "Expected at least one item in list"

    for item in items:
        assert "id" in item, f"Brief item missing 'id': {item}"
        # Full-only fields must not appear in brief list
        assert "hash_code" not in item, f"Brief item should not have 'hash_code': {item}"
        assert "annotations" not in item, f"Brief item should not have 'annotations': {item}"


def test_regular_user_can_list_crds(shared_crd, regular_user):
    """All authenticated users can list CRDs."""
    resp = requests.get(f"{URL_GLOBAL}/crds", headers=auth_headers(regular_user["token"]))
    assert resp.status_code == 200, f"Expected 200, got {resp.status_code}: {resp.text}"
    assert "items" in resp.json()


def test_regular_user_can_get_single_crd(shared_crd, regular_user):
    """All authenticated users can GET a single CRD by id."""
    resp = requests.get(
        f"{URL_GLOBAL}/crds/{shared_crd}", headers=auth_headers(regular_user["token"])
    )
    assert resp.status_code == 200, f"Expected 200, got {resp.status_code}: {resp.text}"
    assert resp.json()["id"] == shared_crd


def test_unauthenticated_cannot_list_crds():
    """Unauthenticated list request returns 401."""
    resp = requests.get(f"{URL_GLOBAL}/crds")
    assert resp.status_code == 401, f"Expected 401, got {resp.status_code}: {resp.text}"


def test_get_nonexistent_crd_returns_404(godmode_token):
    """GET for a CRD that does not exist returns 404."""
    resp = requests.get(
        f"{URL_GLOBAL}/crds/definitely-does-not-exist-99999",
        headers=auth_headers(godmode_token),
    )
    assert resp.status_code == 404, f"Expected 404, got {resp.status_code}: {resp.text}"


# ── Update ─────────────────────────────────────────────────────────────────────


def test_godmode_can_update_crd(godmode_token):
    """Godmode user can PUT to update a CRD; changes are persisted."""
    num = random.randint(100000, 999999)
    crd_id = f"updatable-{num}"
    headers = auth_headers(godmode_token)

    # Create
    resp = requests.post(f"{URL_GLOBAL}/crds", json=_make_crd_payload(crd_id), headers=headers)
    assert resp.status_code == 201, f"Create failed: {resp.text}"
    original_hash = resp.json()["hash_code"]

    # Update — change description and add a field
    payload = _make_crd_payload(crd_id)
    payload["description"] = "Updated description"
    payload["fields"]["env"] = {"type": "string", "required": False}

    resp = requests.put(f"{URL_GLOBAL}/crds/{crd_id}", json=payload, headers=headers)
    assert resp.status_code == 200, f"Expected 200 on update, got {resp.status_code}: {resp.text}"

    data = resp.json()
    assert data["description"] == "Updated description"
    assert "env" in data["fields"]
    # Hash should change when content changes
    assert data["hash_code"] != original_hash, "hash_code should change after update"

    requests.delete(f"{URL_GLOBAL}/crds/{crd_id}", headers=headers)


def test_regular_user_cannot_update_crd(regular_user, godmode_token, shared_crd):
    """Regular user PUT returns 404 (ACL denial) not 403."""
    payload = _make_crd_payload(shared_crd)
    payload["description"] = "Unauthorized modification"
    resp = requests.put(
        f"{URL_GLOBAL}/crds/{shared_crd}",
        json=payload,
        headers=auth_headers(regular_user["token"]),
    )
    assert resp.status_code == 404, f"Expected 404 (ACL denial), got {resp.status_code}: {resp.text}"


# ── Delete ─────────────────────────────────────────────────────────────────────


def test_godmode_can_delete_crd(godmode_token, debug):
    """Godmode user can DELETE a CRD; API returns 404 after; DB has soft-delete marker."""
    num = random.randint(100000, 999999)
    crd_id = f"deletable-{num}"
    headers = auth_headers(godmode_token)

    resp = requests.post(f"{URL_GLOBAL}/crds", json=_make_crd_payload(crd_id), headers=headers)
    assert resp.status_code == 201, f"Create failed: {resp.text}"

    resp = requests.delete(f"{URL_GLOBAL}/crds/{crd_id}", headers=headers)
    assert resp.status_code == 204, f"Expected 204, got {resp.status_code}: {resp.text}"

    # API returns 404 after deletion
    resp = requests.get(f"{URL_GLOBAL}/crds/{crd_id}", headers=headers)
    assert resp.status_code == 404, f"Expected 404 after delete, got {resp.status_code}: {resp.text}"

    # DB should have soft-delete marker (not hard-deleted)
    debug.assert_soft_deleted("crds", crd_id)


def test_regular_user_cannot_delete_crd(regular_user, shared_crd):
    """Regular user DELETE returns 404 (ACL denial) not 403."""
    resp = requests.delete(
        f"{URL_GLOBAL}/crds/{shared_crd}",
        headers=auth_headers(regular_user["token"]),
    )
    assert resp.status_code == 404, f"Expected 404 (ACL denial), got {resp.status_code}: {resp.text}"


# ── Name validation ────────────────────────────────────────────────────────────


@pytest.mark.parametrize("bad_id, reason", [
    ("invalid id with spaces", "contains spaces"),
    ("1starts-with-digit", "starts with digit"),
    ("-starts-with-hyphen", "starts with hyphen"),
    ("x", "too short (1 char)"),
    ("a" * 64, "too long (64 chars)"),
    ("123456", "only digits"),
    ("my.task", "contains dot"),
    ("task@home", "contains at-sign"),
])
def test_invalid_crd_id_rejected(bad_id, reason, godmode_token):
    """CRD ids that violate naming rules are rejected with 4xx."""
    resp = requests.post(
        f"{URL_GLOBAL}/crds",
        json=_make_crd_payload(bad_id),
        headers=auth_headers(godmode_token),
    )
    assert 400 <= resp.status_code < 500, (
        f"Expected 4xx for id '{bad_id}' ({reason}), got {resp.status_code}: {resp.text}"
    )


def test_valid_crd_id_with_hyphens_and_underscores(godmode_token):
    """CRD ids with hyphens and underscores are accepted."""
    num = random.randint(100000, 999999)
    crd_id = f"valid-crd_name-{num}"
    headers = auth_headers(godmode_token)

    resp = requests.post(f"{URL_GLOBAL}/crds", json=_make_crd_payload(crd_id), headers=headers)
    assert resp.status_code == 201, f"Expected 201 for valid id '{crd_id}': {resp.text}"

    requests.delete(f"{URL_GLOBAL}/crds/{crd_id}", headers=headers)


def test_crd_id_is_lowercased(godmode_token):
    """CRD id is stored lowercased even if submitted with uppercase."""
    num = random.randint(100000, 999999)
    # Submit with uppercase — backend should lowercase and accept
    upper_id = f"MyDeployment{num}"
    lower_id = upper_id.lower()
    headers = auth_headers(godmode_token)

    resp = requests.post(f"{URL_GLOBAL}/crds", json=_make_crd_payload(upper_id), headers=headers)
    # Either the backend lowercases and returns 201, or rejects with 4xx (both are valid)
    if resp.status_code == 201:
        assert resp.json()["id"] == lower_id, (
            f"Expected id to be lowercased to '{lower_id}', got '{resp.json()['id']}'"
        )
        requests.delete(f"{URL_GLOBAL}/crds/{lower_id}", headers=headers)
    else:
        assert 400 <= resp.status_code < 500, (
            f"Unexpected status {resp.status_code} for uppercase id: {resp.text}"
        )
