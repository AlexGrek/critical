"""Integration tests for project-scoped TicketGroup resource.

Endpoint base: /api/v1/projects/{project}/ticketgroups

Tests cover:
- CRUD lifecycle (create, read, list, update, delete)
- ID validation (2–6 capital A-Z letters, auto-prefixed with tg_)
- project field injected by scoped handler
- List returns brief fields only (ticket_types excluded)
- Single fetch returns full document (ticket_types included)
- Scoped ACL: resource ACL checked; falls back to project ACL when empty
- Admin (root) bypass
- Non-existent project → 404
"""

import random
import pytest
import requests
from datetime import datetime, timezone

from _utils import (
    auth_headers,
    url_scoped,
    url_project,
    DebugClient,
    URL_LOGIN,
    URL_REGISTER,
    URL_GLOBAL,
    STATUS_OK,
    STATUS_CREATED,
    STATUS_NO_CONTENT,
    STATUS_BAD_REQUEST,
    STATUS_UNAUTHORIZED,
    STATUS_CONFLICT,
    STATUS_NOT_FOUND,
    STATUS_UNPROCESSABLE,
)

# ─── Test Data Constants ───────────────────────────────────────────────────────

# Valid ticket group IDs (2-6 capital letters A-Z)
VALID_2_LETTER = "BG"
VALID_3_LETTER = "BUG"
VALID_4_LETTER = "BUGS"
VALID_5_LETTER = "BUGSS"
VALID_6_LETTER = "SIXSIX"
PREFIXED_ID = "tg_PRE"

# Invalid ticket group IDs
INVALID_1_LETTER = "X"
INVALID_7_LETTER = "TOOLONG"
INVALID_LOWERCASE = "bug"
INVALID_WITH_NUMBERS = "BUG1"

# Sample ticket type structure
SAMPLE_TICKET_TYPE = {
    "name": "Bug",
    "description": "A defect",
    "statuses": [
        {"name": "Open", "category": "todo"},
        {"name": "In Progress", "category": "in_progress"},
        {"name": "Resolved", "category": "done"},
    ],
    "fields": [
        {
            "name": "severity",
            "field_type": "single_select",
            "required": True,
            "options": ["low", "medium", "high"],
        },
    ],
}

SIMPLE_TICKET_TYPE = {
    "name": "Task",
    "statuses": [{"name": "Todo", "category": "todo"}],
    "fields": [],
}


# ─── Helper Functions ─────────────────────────────────────────────────────────


def url_tg(project_id: str, tg_id: str = "") -> str:
    """Build URL to a ticket group resource."""
    return url_scoped(project_id, "ticketgroups", tg_id)


# ─── Fixtures ─────────────────────────────────────────────────────────────────


@pytest.fixture(scope="module")
def admin_token():
    """Root token — has ADM_GODMODE."""
    resp = requests.post(URL_LOGIN, json={"user": "root", "password": "changeme"})
    assert resp.status_code == STATUS_OK, f"Root login failed: {resp.text}"
    return resp.json()["token"]


@pytest.fixture(scope="module")
def regular_user(admin_token):
    """Register a fresh random user and login."""
    num = random.randint(100000, 999999)
    username = f"tgtest_{num}"
    requests.post(
        URL_REGISTER,
        json={"user": username, "password": username},
    )
    resp = requests.post(URL_LOGIN, json={"user": username, "password": username})
    assert resp.status_code == STATUS_OK, f"Regular user login failed: {resp.text}"
    return {"user_id": f"u_{username}", "token": resp.json()["token"]}


@pytest.fixture(scope="module")
def test_project(admin_token):
    """Create a test project scoped to this module; cleanup after."""
    num = random.randint(100000, 999999)
    pid = f"tgtest_{num}"
    headers = auth_headers(admin_token)
    # POST to /v1/global/projects (without ID in path) with ID in body for create
    resp = requests.post(
        f"{URL_GLOBAL}/projects",
        json={"id": pid, "name": "TicketGroup Test Project"},
        headers=headers,
    )
    assert resp.status_code == STATUS_CREATED, f"Create project failed: {resp.text}"
    yield pid
    # DELETE from /v1/global/projects/{id}
    requests.delete(url_project(pid), headers=headers)


@pytest.fixture(scope="module")
def debug(admin_token) -> DebugClient:
    """DebugClient backed by root's ADM_GODMODE token."""
    return DebugClient(admin_token)


# ─── create ───────────────────────────────────────────────────────────────────


def test_create_ticketgroup_minimal(admin_token, test_project):
    """POST with a valid 2-letter code creates a ticketgroup (201)."""
    headers = auth_headers(admin_token)
    resp = requests.post(
        url_tg(test_project),
        json={"id": VALID_2_LETTER, "name": "Bugs"},
        headers=headers,
    )
    assert (
        resp.status_code == STATUS_CREATED
    ), f"Expected {STATUS_CREATED}: {resp.text}"
    body = resp.json()
    assert body["id"] == f"tg_{VALID_2_LETTER}"
    assert body["name"] == "Bugs"
    assert body["project"] == test_project
    assert "hash_code" in body

    # cleanup
    requests.delete(url_tg(test_project, f"tg_{VALID_2_LETTER}"), headers=headers)


def test_create_ticketgroup_with_types(admin_token, test_project):
    """POST with full ticket_types creates correctly and returns them."""
    headers = auth_headers(admin_token)
    resp = requests.post(
        url_tg(test_project),
        json={"id": VALID_4_LETTER, "name": "Bug Tracking", "ticket_types": [SAMPLE_TICKET_TYPE]},
        headers=headers,
    )
    assert (
        resp.status_code == STATUS_CREATED
    ), f"Expected {STATUS_CREATED}: {resp.text}"
    body = resp.json()
    assert body["id"] == f"tg_{VALID_4_LETTER}"
    assert len(body["ticket_types"]) == 1
    assert body["ticket_types"][0]["name"] == "Bug"
    assert len(body["ticket_types"][0]["statuses"]) == 3

    # cleanup
    requests.delete(url_tg(test_project, f"tg_{VALID_4_LETTER}"), headers=headers)


def test_create_ticketgroup_six_letter_code(admin_token, test_project):
    """6-letter code is the maximum — should succeed."""
    headers = auth_headers(admin_token)
    resp = requests.post(
        url_tg(test_project),
        json={"id": VALID_6_LETTER, "name": "Six Letters"},
        headers=headers,
    )
    assert (
        resp.status_code == STATUS_CREATED
    ), f"Expected {STATUS_CREATED}: {resp.text}"
    requests.delete(url_tg(test_project, f"tg_{VALID_6_LETTER}"), headers=headers)


# ─── ID validation ────────────────────────────────────────────────────────────


def test_create_ticketgroup_id_too_short(admin_token, test_project):
    """Single-letter code is too short — should be rejected."""
    resp = requests.post(
        url_tg(test_project),
        json={"id": INVALID_1_LETTER, "name": "Too Short"},
        headers=auth_headers(admin_token),
    )
    assert resp.status_code in (
        STATUS_BAD_REQUEST,
        STATUS_UNPROCESSABLE,
    ), f"Expected 400/422, got {resp.status_code}: {resp.text}"


def test_create_ticketgroup_id_too_long(admin_token, test_project):
    """Seven-letter code is too long — should be rejected."""
    resp = requests.post(
        url_tg(test_project),
        json={"id": INVALID_7_LETTER, "name": "Too Long"},
        headers=auth_headers(admin_token),
    )
    assert resp.status_code in (
        STATUS_BAD_REQUEST,
        STATUS_UNPROCESSABLE,
    ), f"Expected 400/422, got {resp.status_code}: {resp.text}"


def test_create_ticketgroup_id_lowercase_rejected(admin_token, test_project):
    """Lowercase letters in code should be rejected."""
    resp = requests.post(
        url_tg(test_project),
        json={"id": INVALID_LOWERCASE, "name": "lowercase"},
        headers=auth_headers(admin_token),
    )
    assert resp.status_code in (
        STATUS_BAD_REQUEST,
        STATUS_UNPROCESSABLE,
    ), f"Expected 400/422, got {resp.status_code}: {resp.text}"


def test_create_ticketgroup_id_with_numbers_rejected(admin_token, test_project):
    """Digits in code should be rejected (only A-Z allowed)."""
    resp = requests.post(
        url_tg(test_project),
        json={"id": INVALID_WITH_NUMBERS, "name": "with number"},
        headers=auth_headers(admin_token),
    )
    assert resp.status_code in (
        STATUS_BAD_REQUEST,
        STATUS_UNPROCESSABLE,
    ), f"Expected 400/422, got {resp.status_code}: {resp.text}"


def test_create_ticketgroup_prefixed_id_accepted(admin_token, test_project):
    """Sending id as tg_BUG (already prefixed) should work."""
    headers = auth_headers(admin_token)
    resp = requests.post(
        url_tg(test_project),
        json={"id": PREFIXED_ID, "name": "Prefixed"},
        headers=headers,
    )
    assert (
        resp.status_code == STATUS_CREATED
    ), f"Expected {STATUS_CREATED}: {resp.text}"
    assert resp.json()["id"] == PREFIXED_ID
    requests.delete(url_tg(test_project, PREFIXED_ID), headers=headers)


# ─── read ─────────────────────────────────────────────────────────────────────


def test_get_ticketgroup_returns_full_doc(admin_token, test_project):
    """Single GET returns ticket_types (full doc), not just brief fields."""
    headers = auth_headers(admin_token)
    # Create with types
    requests.post(
        url_tg(test_project),
        json={
            "id": "FULL",
            "name": "Full Fetch",
            "ticket_types": [SIMPLE_TICKET_TYPE],
        },
        headers=headers,
    )

    resp = requests.get(url_tg(test_project, "tg_FULL"), headers=headers)
    assert resp.status_code == STATUS_OK, f"Expected {STATUS_OK}: {resp.text}"
    body = resp.json()
    assert body["id"] == "tg_FULL"
    assert body["project"] == test_project
    assert "ticket_types" in body, "ticket_types should be present in full fetch"
    assert len(body["ticket_types"]) == 1
    assert "hash_code" in body

    requests.delete(url_tg(test_project, "tg_FULL"), headers=headers)


def test_get_ticketgroup_not_found(admin_token, test_project):
    """GET non-existent ticketgroup returns 404."""
    resp = requests.get(
        url_tg(test_project, "tg_NOPE"),
        headers=auth_headers(admin_token),
    )
    assert resp.status_code == STATUS_NOT_FOUND


# ─── list ─────────────────────────────────────────────────────────────────────


def test_list_ticketgroups_brief_fields(admin_token, test_project):
    """List returns brief fields; ticket_types must NOT appear."""
    headers = auth_headers(admin_token)
    tg_id = "LST"
    requests.post(
        url_tg(test_project),
        json={
            "id": tg_id,
            "name": "List Test",
            "ticket_types": [SIMPLE_TICKET_TYPE],
        },
        headers=headers,
    )

    resp = requests.get(url_tg(test_project), headers=headers)
    assert resp.status_code == STATUS_OK
    items = resp.json()["items"]
    match = next((i for i in items if i["id"] == f"tg_{tg_id}"), None)
    assert match is not None, f"tg_{tg_id} not in list: {items}"
    assert (
        "ticket_types" not in match
    ), f"ticket_types should be excluded from brief list: {match}"
    assert "name" in match
    assert "project" in match

    requests.delete(url_tg(test_project, f"tg_{tg_id}"), headers=headers)


def test_list_ticketgroups_empty_project(admin_token):
    """List on a project with no ticketgroups returns empty items list."""
    num = random.randint(100000, 999999)
    pid = f"empty_tg_{num}"
    headers = auth_headers(admin_token)
    requests.post(
        f"{URL_GLOBAL}/projects",
        json={"id": pid, "name": "Empty"},
        headers=headers,
    )

    resp = requests.get(url_tg(pid), headers=headers)
    assert resp.status_code == STATUS_OK
    assert resp.json()["items"] == []

    requests.delete(url_project(pid), headers=headers)


# ─── update ───────────────────────────────────────────────────────────────────


def test_update_ticketgroup_name(admin_token, test_project):
    """PUT updates the name and returns the full updated document."""
    headers = auth_headers(admin_token)
    # Create
    create_resp = requests.post(
        url_tg(test_project),
        json={"id": "UPD", "name": "Original Name"},
        headers=headers,
    )
    assert create_resp.status_code == STATUS_CREATED
    hash_code = create_resp.json()["hash_code"]

    # Update
    resp = requests.put(
        url_tg(test_project, "tg_UPD"),
        json={
            "id": "tg_UPD",
            "name": "Updated Name",
            "project": test_project,
            "hash_code": hash_code,
        },
        headers=headers,
    )
    assert resp.status_code == STATUS_OK, f"Expected {STATUS_OK}: {resp.text}"
    assert resp.json()["name"] == "Updated Name"
    assert resp.json()["id"] == "tg_UPD"

    # Verify via GET
    get_resp = requests.get(url_tg(test_project, "tg_UPD"), headers=headers)
    assert get_resp.json()["name"] == "Updated Name"

    requests.delete(url_tg(test_project, "tg_UPD"), headers=headers)


def test_update_ticketgroup_add_ticket_type(admin_token, test_project):
    """PUT can add ticket_types to an existing ticketgroup."""
    headers = auth_headers(admin_token)
    create_resp = requests.post(
        url_tg(test_project),
        json={"id": "ADDTP", "name": "Add Types", "ticket_types": []},
        headers=headers,
    )
    assert create_resp.status_code == STATUS_CREATED
    hash_code = create_resp.json()["hash_code"]

    resp = requests.put(
        url_tg(test_project, "tg_ADDTP"),
        json={
            "id": "tg_ADDTP",
            "name": "Add Types",
            "project": test_project,
            "hash_code": hash_code,
            "ticket_types": [SIMPLE_TICKET_TYPE],
        },
        headers=headers,
    )
    assert resp.status_code == STATUS_OK, f"Expected {STATUS_OK}: {resp.text}"
    assert len(resp.json()["ticket_types"]) == 1

    requests.delete(url_tg(test_project, "tg_ADDTP"), headers=headers)


# ─── delete ───────────────────────────────────────────────────────────────────


def test_delete_ticketgroup(admin_token, test_project, debug):
    """DELETE soft-deletes the ticketgroup (204); subsequent GET returns 404."""
    headers = auth_headers(admin_token)
    requests.post(
        url_tg(test_project),
        json={"id": "DEL", "name": "To Delete"},
        headers=headers,
    )

    resp = requests.delete(url_tg(test_project, "tg_DEL"), headers=headers)
    assert (
        resp.status_code == STATUS_NO_CONTENT
    ), f"Expected {STATUS_NO_CONTENT}: {resp.text}"

    # API 404 after soft-delete
    assert (
        requests.get(url_tg(test_project, "tg_DEL"), headers=headers).status_code
        == STATUS_NOT_FOUND
    )

    # DB has soft-delete marker — key is composite: {project}__{code}
    debug.assert_soft_deleted("ticketgroups", f"{test_project}__DEL")


def test_delete_ticketgroup_not_found(admin_token, test_project):
    """DELETE non-existent ticketgroup returns 404."""
    resp = requests.delete(
        url_tg(test_project, "tg_GONE"),
        headers=auth_headers(admin_token),
    )
    assert resp.status_code == STATUS_NOT_FOUND


# ─── non-existent project ─────────────────────────────────────────────────────


def test_list_ticketgroups_nonexistent_project(admin_token):
    """List on a non-existent project returns 404."""
    resp = requests.get(
        url_tg("no_such_project_xyz"),
        headers=auth_headers(admin_token),
    )
    assert resp.status_code == STATUS_NOT_FOUND


def test_create_ticketgroup_nonexistent_project(admin_token):
    """Create on a non-existent project returns 404."""
    resp = requests.post(
        url_tg("no_such_project_xyz"),
        json={"id": VALID_3_LETTER, "name": "Orphan"},
        headers=auth_headers(admin_token),
    )
    assert resp.status_code == STATUS_NOT_FOUND


# ─── ACL / access control ─────────────────────────────────────────────────────


def test_regular_user_can_read_public_ticketgroup(admin_token, regular_user, test_project):
    """A regular user with project-level READ access can fetch a ticketgroup."""
    headers_admin = auth_headers(admin_token)
    headers_user = auth_headers(regular_user["token"])

    # Grant the regular user READ on the project
    project_resp = requests.get(
        url_project(test_project),
        headers=headers_admin,
    )
    assert project_resp.status_code == STATUS_OK
    project_hash = project_resp.json()["hash_code"]

    acl = {
        "list": [
            {"permissions": 127, "principals": ["u_root"]},
            {"permissions": 7, "principals": [regular_user["user_id"]]},
        ],
        "last_mod_date": datetime.now(timezone.utc).isoformat(),
    }
    requests.put(
        url_project(test_project),
        json={
            "id": test_project,
            "name": "TicketGroup Test Project",
            "hash_code": project_hash,
            "acl": acl,
        },
        headers=headers_admin,
    )

    # Create a ticketgroup with empty ACL (inherits project ACL)
    requests.post(
        url_tg(test_project),
        json={"id": "ACR", "name": "ACL Read Test"},
        headers=headers_admin,
    )

    # Regular user should be able to list and fetch
    resp = requests.get(url_tg(test_project), headers=headers_user)
    assert resp.status_code == STATUS_OK, f"Regular user list failed: {resp.text}"

    resp = requests.get(url_tg(test_project, "tg_ACR"), headers=headers_user)
    assert resp.status_code == STATUS_OK, f"Regular user fetch failed: {resp.text}"

    requests.delete(url_tg(test_project, "tg_ACR"), headers=headers_admin)


def test_unauthenticated_request_rejected(test_project):
    """Requests without a JWT are rejected with 401."""
    resp = requests.get(url_tg(test_project))
    assert resp.status_code == STATUS_UNAUTHORIZED


# ─── project field injection ──────────────────────────────────────────────────


def test_project_field_injected_on_create(admin_token, test_project):
    """The project field in the response matches the URL project parameter."""
    headers = auth_headers(admin_token)
    resp = requests.post(
        url_tg(test_project),
        json={"id": "INJ", "name": "Project Injection"},
        headers=headers,
    )
    assert resp.status_code == STATUS_CREATED
    assert resp.json()["project"] == test_project
    requests.delete(url_tg(test_project, "tg_INJ"), headers=headers)


def test_project_field_in_list_items(admin_token, test_project):
    """Brief list items include the project field."""
    headers = auth_headers(admin_token)
    requests.post(
        url_tg(test_project),
        json={"id": "PJL", "name": "Project In List"},
        headers=headers,
    )

    resp = requests.get(url_tg(test_project), headers=headers)
    items = resp.json()["items"]
    match = next((i for i in items if i["id"] == "tg_PJL"), None)
    assert match is not None
    assert match["project"] == test_project

    requests.delete(url_tg(test_project, "tg_PJL"), headers=headers)


# ─── db-level verification ────────────────────────────────────────────────────


def test_ticketgroup_stored_in_correct_collection(admin_token, test_project, debug):
    """Verify the ticketgroup document is stored in ticketgroups collection."""
    headers = auth_headers(admin_token)
    requests.post(
        url_tg(test_project),
        json={"id": "DB", "name": "DB Check"},
        headers=headers,
    )

    # _key in DB is composite: {project}__{code}
    doc = debug.find_one("ticketgroups", _key=f"{test_project}__DB")
    assert (
        doc is not None
    ), f"{test_project}__DB not found in ticketgroups: {debug.dump('ticketgroups')}"
    assert doc["project"] == test_project
    assert doc["name"] == "DB Check"

    requests.delete(url_tg(test_project, "tg_DB"), headers=headers)
