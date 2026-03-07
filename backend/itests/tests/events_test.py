"""Integration tests for the system events logging feature.

Tests verify that backend actions produce the expected events in the
system_events collection, accessible via GET /v1/debug/events.

Tests run in parallel (xdist), so we fetch the last 10 events and
search for our specific event rather than assuming it's the most recent.
"""

import random
import requests
import pytest

BASE = "http://localhost:3742/api"
URL_LOGIN = f"{BASE}/v1/login"
URL_REGISTER = f"{BASE}/v1/register"
URL_DEBUG = f"{BASE}/v1/debug"
URL_GLOBAL = f"{BASE}/v1/global"

ROOT_PASSWORD = "changeme"


def auth_headers(token: str) -> dict:
    return {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json",
    }


def fetch_recent_events(root_token: str, limit: int = 10, kind: str = None, priority: str = None) -> list[dict]:
    """Fetch recent events from the debug endpoint, with optional filters."""
    params = {"limit": limit}
    if kind:
        params["kind"] = kind
    if priority:
        params["priority"] = priority
    resp = requests.get(
        f"{URL_DEBUG}/events",
        params=params,
        headers=auth_headers(root_token),
    )
    assert resp.status_code == 200, f"Failed to fetch events: {resp.text}"
    return resp.json()["events"]


def find_event(events: list[dict], **matchers) -> dict | None:
    """Find an event matching all matchers. Supports nested key paths like 'details.action'."""
    for event in events:
        match = True
        for key, val in matchers.items():
            parts = key.split(".")
            obj = event
            for part in parts:
                if not isinstance(obj, dict) or part not in obj:
                    obj = None
                    break
                obj = obj[part]
            if obj != val:
                match = False
                break
        if match:
            return event
    return None


@pytest.fixture(scope="module")
def root_token():
    resp = requests.post(URL_LOGIN, json={"user": "root", "password": ROOT_PASSWORD})
    assert resp.status_code == 200, f"Root login failed: {resp.text}"
    token = resp.json()["token"]
    print(f"\n[events_test] root_token={token}")
    return token


# ------------------- Test 1: login produces a Server event --------------------


def test_login_produces_server_event(root_token):
    """Logging in as a user should emit a Server/Minor event with action=login."""
    num = random.randint(100000, 999999)
    user = f"evt_itest_{num}"
    user_id = f"u_{user}"

    # Register and log in as a fresh user
    r = requests.post(URL_REGISTER, json={"user": user, "password": user + "!"})
    assert r.status_code == 201, f"Register failed: {r.text}"

    r = requests.post(URL_LOGIN, json={"user": user, "password": user + "!"})
    assert r.status_code == 200, f"Login failed: {r.text}"

    # Fetch last 10 Server/Minor events and look for ours
    events = fetch_recent_events(root_token, limit=10, kind="Server")
    event = find_event(events, principal=user_id, **{"details.action": "login"})

    assert event is not None, (
        f"Expected a Server event with principal={user_id} and details.action=login, "
        f"but it was not found in recent events:\n"
        + "\n".join(str(e) for e in events)
    )
    assert event["kind"] == "Server"
    assert event["priority"] == "Minor"


# ------------------- Test 2: creating a resource produces an EntityManagement event --------------------


def test_create_resource_produces_lifecycle_event(root_token):
    """Creating a group via the API should emit an EntityManagement/Lifecycle event."""
    num = random.randint(100000, 999999)
    gid = f"g_evt_itest_{num}"
    resource_path = f"groups/{gid}"

    resp = requests.post(
        f"{URL_GLOBAL}/groups",
        json={
            "id": gid,
            "name": f"Event Test Group {num}",
            "acl": {
                "list": [{"permissions": 127, "principals": ["u_root"]}],
                "last_mod_date": "2024-01-01T00:00:00Z",
            },
        },
        headers=auth_headers(root_token),
    )
    assert resp.status_code == 201, f"Group creation failed: {resp.text}"

    try:
        events = fetch_recent_events(root_token, limit=10, kind="EntityManagement", priority="Lifecycle")
        event = find_event(events, **{"details.action": "created"})
        # Find specifically for our resource in affects
        event = next(
            (e for e in events
             if e.get("details", {}).get("action") == "created"
             and resource_path in e.get("affects", [])),
            None,
        )

        assert event is not None, (
            f"Expected EntityManagement/Lifecycle event for {resource_path} with action=created, "
            f"not found in recent events:\n"
            + "\n".join(str(e) for e in events)
        )
        assert event["kind"] == "EntityManagement"
        assert event["priority"] == "Lifecycle"
        assert resource_path in event["affects"]
    finally:
        requests.delete(f"{URL_GLOBAL}/groups/{gid}", headers=auth_headers(root_token))


# ------------------- Test 3: deleting a resource produces an EntityManagement event --------------------


def test_delete_resource_produces_lifecycle_event(root_token):
    """Deleting a group via the API should emit an EntityManagement/Lifecycle event with action=deleted."""
    num = random.randint(100000, 999999)
    gid = f"g_evt_del_{num}"
    resource_path = f"groups/{gid}"

    # Create the group first
    resp = requests.post(
        f"{URL_GLOBAL}/groups",
        json={
            "id": gid,
            "name": f"Event Delete Test {num}",
            "acl": {
                "list": [{"permissions": 127, "principals": ["u_root"]}],
                "last_mod_date": "2024-01-01T00:00:00Z",
            },
        },
        headers=auth_headers(root_token),
    )
    assert resp.status_code == 201, f"Group creation failed: {resp.text}"

    # Delete it
    resp = requests.delete(f"{URL_GLOBAL}/groups/{gid}", headers=auth_headers(root_token))
    assert resp.status_code == 204, f"Group deletion failed: {resp.text}"

    # Fetch recent EntityManagement/Lifecycle events and find our delete event
    events = fetch_recent_events(root_token, limit=10, kind="EntityManagement", priority="Lifecycle")
    event = next(
        (e for e in events
         if e.get("details", {}).get("action") == "deleted"
         and resource_path in e.get("affects", [])),
        None,
    )

    assert event is not None, (
        f"Expected EntityManagement/Lifecycle event for {resource_path} with action=deleted, "
        f"not found in recent events:\n"
        + "\n".join(str(e) for e in events)
    )
    assert event["kind"] == "EntityManagement"
    assert event["priority"] == "Lifecycle"
    assert resource_path in event["affects"]


# ------------------- Test 4: events endpoint requires godmode --------------------


def test_events_endpoint_requires_godmode():
    """Regular users should not be able to access the debug events endpoint."""
    num = random.randint(100000, 999999)
    user = f"evt_noauth_{num}"
    requests.post(URL_REGISTER, json={"user": user, "password": user + "!"})
    r = requests.post(URL_LOGIN, json={"user": user, "password": user + "!"})
    assert r.status_code == 200
    token = r.json()["token"]

    resp = requests.get(f"{URL_DEBUG}/events", headers=auth_headers(token))
    assert resp.status_code == 401, f"Expected 401 for regular user, got {resp.status_code}: {resp.text}"
