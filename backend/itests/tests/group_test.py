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


def _make_group_body(group_id, name, acl_list=None):
    """Helper to build a group JSON body."""
    return {
        "id": group_id,
        "name": name,
        "acl": {
            "list": acl_list or [],
            "last_mod_date": datetime.now(timezone.utc).isoformat(),
        },
    }


@pytest.fixture(scope="module")
def admin_token():
    """Login as root (has all super-permissions including ADM_USER_MANAGER)."""
    resp = requests.post(URL_LOGIN, json={"user": "root", "password": "changeme"})
    assert resp.status_code == 200, f"Root login failed: {resp.text}"
    return resp.json()["token"]


@pytest.fixture(scope="module")
def regular_user():
    """Register a regular user (gets usr_create_groups by default)."""
    num = random.randint(100000, 999999)
    user = f"grp_test_{num}"
    password = f"grp_test_{num}"

    requests.post(URL_REGISTER, json={"user": user, "password": password})
    resp = requests.post(URL_LOGIN, json={"user": user, "password": password})
    assert resp.status_code == 200, f"Login failed: {resp.text}"
    return {
        "user_id": f"u_{user}",
        "token": resp.json()["token"],
    }


@pytest.fixture(scope="module")
def second_user():
    """Register a second regular user for ACL testing."""
    num = random.randint(100000, 999999)
    user = f"grp_test2_{num}"
    password = f"grp_test2_{num}"

    requests.post(URL_REGISTER, json={"user": user, "password": password})
    resp = requests.post(URL_LOGIN, json={"user": user, "password": password})
    assert resp.status_code == 200, f"Login failed: {resp.text}"
    return {
        "user_id": f"u_{user}",
        "token": resp.json()["token"],
    }


# ------------------- Group creation with ACL --------------------


def test_regular_user_can_create_group(regular_user):
    """Regular users with usr_create_groups can create groups."""
    num = random.randint(100000, 999999)
    group_id = f"g_test_{num}"
    headers = auth_headers(regular_user["token"])

    resp = requests.post(
        f"{URL_GLOBAL}/groups",
        json=_make_group_body(group_id, "Test Group"),
        headers=headers,
    )
    assert resp.status_code == 201, f"Expected 201, got {resp.status_code}: {resp.text}"

    # Verify creator can read the group
    resp = requests.get(f"{URL_GLOBAL}/groups/{group_id}", headers=headers)
    assert resp.status_code == 200
    data = resp.json()
    assert data["id"] == group_id
    assert data["name"] == "Test Group"

    # Verify creator is in ACL with ROOT permissions
    acl_principals = []
    for entry in data.get("acl", {}).get("list", []):
        acl_principals.extend(entry.get("principals", []))
    assert regular_user["user_id"] in acl_principals, (
        f"Creator {regular_user['user_id']} should be in group ACL, got {acl_principals}"
    )

    # Cleanup
    resp = requests.delete(f"{URL_GLOBAL}/groups/{group_id}", headers=headers)
    assert resp.status_code == 204


def test_creator_has_root_acl(regular_user):
    """Creator gets ROOT permissions in group ACL, can update and delete."""
    num = random.randint(100000, 999999)
    group_id = f"g_root_acl_{num}"
    headers = auth_headers(regular_user["token"])

    # Create
    resp = requests.post(
        f"{URL_GLOBAL}/groups",
        json=_make_group_body(group_id, "Root ACL Test"),
        headers=headers,
    )
    assert resp.status_code == 201

    # Update (requires MODIFY, which is part of ROOT)
    resp = requests.get(f"{URL_GLOBAL}/groups/{group_id}", headers=headers)
    data = resp.json()
    updated = _make_group_body(group_id, "Updated Name", data["acl"]["list"])
    resp = requests.put(f"{URL_GLOBAL}/groups/{group_id}", json=updated, headers=headers)
    assert resp.status_code == 200

    # Verify name changed
    resp = requests.get(f"{URL_GLOBAL}/groups/{group_id}", headers=headers)
    assert resp.json()["name"] == "Updated Name"

    # Delete
    resp = requests.delete(f"{URL_GLOBAL}/groups/{group_id}", headers=headers)
    assert resp.status_code == 204


# ------------------- ACL enforcement --------------------


def test_user_without_acl_cannot_read_group(regular_user, second_user):
    """Users not in group ACL cannot read the group."""
    num = random.randint(100000, 999999)
    group_id = f"g_no_acl_{num}"
    creator_headers = auth_headers(regular_user["token"])

    # Creator creates the group
    resp = requests.post(
        f"{URL_GLOBAL}/groups",
        json=_make_group_body(group_id, "Private Group"),
        headers=creator_headers,
    )
    assert resp.status_code == 201

    # Second user cannot read it
    resp = requests.get(
        f"{URL_GLOBAL}/groups/{group_id}",
        headers=auth_headers(second_user["token"]),
    )
    assert resp.status_code == 404, f"Expected 404, got {resp.status_code}: {resp.text}"

    # Second user cannot see it in list
    resp = requests.get(f"{URL_GLOBAL}/groups", headers=auth_headers(second_user["token"]))
    assert resp.status_code == 200
    ids = [item["id"] for item in resp.json()["items"]]
    assert group_id not in ids

    # Cleanup
    requests.delete(f"{URL_GLOBAL}/groups/{group_id}", headers=creator_headers)


def test_user_without_modify_cannot_update_group(regular_user, second_user):
    """Users with only READ ACL cannot update the group."""
    num = random.randint(100000, 999999)
    group_id = f"g_read_only_{num}"
    creator_headers = auth_headers(regular_user["token"])

    # Create with READ (7) for second user
    acl_list = [
        {"permissions": 127, "principals": [regular_user["user_id"]]},  # ROOT for creator
        {"permissions": 7, "principals": [second_user["user_id"]]},  # READ only
    ]
    resp = requests.post(
        f"{URL_GLOBAL}/groups",
        json=_make_group_body(group_id, "Read Only Test", acl_list),
        headers=creator_headers,
    )
    assert resp.status_code == 201

    # Second user can read it
    resp = requests.get(
        f"{URL_GLOBAL}/groups/{group_id}",
        headers=auth_headers(second_user["token"]),
    )
    assert resp.status_code == 200

    # Second user cannot update it (only has READ, not MODIFY)
    resp = requests.put(
        f"{URL_GLOBAL}/groups/{group_id}",
        json=_make_group_body(group_id, "Hacked Name", acl_list),
        headers=auth_headers(second_user["token"]),
    )
    assert resp.status_code == 404, f"Expected 404, got {resp.status_code}: {resp.text}"

    # Cleanup
    requests.delete(f"{URL_GLOBAL}/groups/{group_id}", headers=creator_headers)


def test_admin_bypasses_group_acl(admin_token, regular_user):
    """ADM_USER_MANAGER can read/write any group regardless of ACL."""
    num = random.randint(100000, 999999)
    group_id = f"g_admin_bypass_{num}"
    creator_headers = auth_headers(regular_user["token"])
    admin_headers = auth_headers(admin_token)

    # Regular user creates the group
    resp = requests.post(
        f"{URL_GLOBAL}/groups",
        json=_make_group_body(group_id, "Admin Bypass Test"),
        headers=creator_headers,
    )
    assert resp.status_code == 201

    # Admin can read it (not in ACL, but has ADM_USER_MANAGER)
    resp = requests.get(f"{URL_GLOBAL}/groups/{group_id}", headers=admin_headers)
    assert resp.status_code == 200

    # Admin can update it
    resp = requests.get(f"{URL_GLOBAL}/groups/{group_id}", headers=admin_headers)
    data = resp.json()
    updated = _make_group_body(group_id, "Admin Updated", data["acl"]["list"])
    resp = requests.put(f"{URL_GLOBAL}/groups/{group_id}", json=updated, headers=admin_headers)
    assert resp.status_code == 200

    # Admin can delete it
    resp = requests.delete(f"{URL_GLOBAL}/groups/{group_id}", headers=admin_headers)
    assert resp.status_code == 204


# ------------------- Cascade: user deletion removes from groups --------------------


def test_cascade_user_deletion_removes_from_groups(admin_token):
    """Deleting a user removes it from all groups via cascade."""
    num = random.randint(100000, 999999)
    admin_headers = auth_headers(admin_token)

    # Create a user
    user_id = f"u_cascade_{num}"
    resp = requests.post(
        f"{URL_GLOBAL}/users",
        json={
            "id": user_id,
            "password": "test123",
            "personal": {"name": "Cascade Test", "gender": "", "job_title": "", "manager": None},
            "state": "active",
            "meta": {},
        },
        headers=admin_headers,
    )
    assert resp.status_code == 201

    # Create a group with the user and root in ACL
    group_id = f"g_cascade_{num}"
    acl_list = [
        {"permissions": 127, "principals": ["u_root", user_id]},
    ]
    resp = requests.post(
        f"{URL_GLOBAL}/groups",
        json=_make_group_body(group_id, "Cascade Group", acl_list),
        headers=admin_headers,
    )
    assert resp.status_code == 201
    # after_create auto-adds u_root as member — that's expected

    # Add user as member via memberships
    membership_key = f"{user_id}::{group_id}"
    resp = requests.post(
        f"{URL_GLOBAL}/memberships",
        json={
            "id": membership_key,
            "_from": f"users/{user_id}",
            "_to": f"groups/{group_id}",
            "principal": user_id,
            "group": group_id,
        },
        headers=admin_headers,
    )
    assert resp.status_code == 201, f"Expected 201, got {resp.status_code}: {resp.text}"

    # Verify membership exists
    resp = requests.get(f"{URL_GLOBAL}/memberships/{membership_key}", headers=admin_headers)
    assert resp.status_code == 200

    # Delete the user
    resp = requests.delete(f"{URL_GLOBAL}/users/{user_id}", headers=admin_headers)
    assert resp.status_code == 204

    # Membership should be gone
    resp = requests.get(f"{URL_GLOBAL}/memberships/{membership_key}", headers=admin_headers)
    assert resp.status_code == 404

    # Group should still exist (u_root is still a member via after_create)
    resp = requests.get(f"{URL_GLOBAL}/groups/{group_id}", headers=admin_headers)
    assert resp.status_code == 200

    # Cleanup: delete the group
    requests.delete(f"{URL_GLOBAL}/groups/{group_id}", headers=admin_headers)


def test_cascade_user_deletion_empty_group_deleted(admin_token):
    """When deleting a user empties a group, that group is auto-deleted."""
    num = random.randint(100000, 999999)
    admin_headers = auth_headers(admin_token)

    # Create a user
    user_id = f"u_empty_{num}"
    resp = requests.post(
        f"{URL_GLOBAL}/users",
        json={
            "id": user_id,
            "password": "test123",
            "personal": {"name": "Empty Group Test", "gender": "", "job_title": "", "manager": None},
            "state": "active",
            "meta": {},
        },
        headers=admin_headers,
    )
    assert resp.status_code == 201

    # Create a group (root in ACL for management only)
    group_id = f"g_empty_{num}"
    acl_list = [
        {"permissions": 127, "principals": ["u_root"]},
    ]
    resp = requests.post(
        f"{URL_GLOBAL}/groups",
        json=_make_group_body(group_id, "Empty Group", acl_list),
        headers=admin_headers,
    )
    assert resp.status_code == 201

    # Add user as member first (group currently has [u_root] from after_create)
    membership_key = f"{user_id}::{group_id}"
    resp = requests.post(
        f"{URL_GLOBAL}/memberships",
        json={
            "id": membership_key,
            "_from": f"users/{user_id}",
            "_to": f"groups/{group_id}",
            "principal": user_id,
            "group": group_id,
        },
        headers=admin_headers,
    )
    assert resp.status_code == 201

    # Now remove u_root so user is the sole member
    # (must be after adding user, otherwise group becomes empty and gets cascade-deleted)
    requests.delete(f"{URL_GLOBAL}/memberships/u_root::{group_id}", headers=admin_headers)

    # Delete the user — the group should be auto-deleted since it's now empty
    resp = requests.delete(f"{URL_GLOBAL}/users/{user_id}", headers=admin_headers)
    assert resp.status_code == 204

    # Group should be gone
    resp = requests.get(f"{URL_GLOBAL}/groups/{group_id}", headers=admin_headers)
    assert resp.status_code == 404, (
        f"Expected group {group_id} to be deleted (empty), got {resp.status_code}"
    )


# ------------------- Cascade: group deletion --------------------


def test_cascade_group_deletion_removes_from_parent(admin_token):
    """Deleting a child group removes it from parent groups."""
    num = random.randint(100000, 999999)
    admin_headers = auth_headers(admin_token)

    # Create parent and child groups
    parent_id = f"g_parent_{num}"
    child_id = f"g_child_{num}"

    acl_list = [{"permissions": 127, "principals": ["u_root"]}]

    resp = requests.post(
        f"{URL_GLOBAL}/groups",
        json=_make_group_body(parent_id, "Parent", acl_list),
        headers=admin_headers,
    )
    assert resp.status_code == 201

    resp = requests.post(
        f"{URL_GLOBAL}/groups",
        json=_make_group_body(child_id, "Child", acl_list),
        headers=admin_headers,
    )
    assert resp.status_code == 201

    # Add a user to parent so it won't be empty when child is removed
    # (parent already has u_root from after_create, plus we add this user)
    user_id = f"u_parent_member_{num}"
    resp = requests.post(
        f"{URL_GLOBAL}/users",
        json={
            "id": user_id,
            "password": "test123",
            "personal": {"name": "Parent Member", "gender": "", "job_title": "", "manager": None},
            "state": "active",
            "meta": {},
        },
        headers=admin_headers,
    )
    assert resp.status_code == 201

    # Add user as member of parent
    resp = requests.post(
        f"{URL_GLOBAL}/memberships",
        json={
            "id": f"{user_id}::{parent_id}",
            "_from": f"users/{user_id}",
            "_to": f"groups/{parent_id}",
            "principal": user_id,
            "group": parent_id,
        },
        headers=admin_headers,
    )
    assert resp.status_code == 201

    # Add child group as member of parent
    child_membership_key = f"{child_id}::{parent_id}"
    resp = requests.post(
        f"{URL_GLOBAL}/memberships",
        json={
            "id": child_membership_key,
            "_from": f"groups/{child_id}",
            "_to": f"groups/{parent_id}",
            "principal": child_id,
            "group": parent_id,
        },
        headers=admin_headers,
    )
    assert resp.status_code == 201

    # Delete child group
    resp = requests.delete(f"{URL_GLOBAL}/groups/{child_id}", headers=admin_headers)
    assert resp.status_code == 204

    # Child membership edge should be gone
    resp = requests.get(
        f"{URL_GLOBAL}/memberships/{child_membership_key}",
        headers=admin_headers,
    )
    assert resp.status_code == 404

    # Parent should still exist (still has the user member)
    resp = requests.get(f"{URL_GLOBAL}/groups/{parent_id}", headers=admin_headers)
    assert resp.status_code == 200

    # Cleanup
    requests.delete(f"{URL_GLOBAL}/memberships/{user_id}::{parent_id}", headers=admin_headers)
    requests.delete(f"{URL_GLOBAL}/users/{user_id}", headers=admin_headers)
    requests.delete(f"{URL_GLOBAL}/groups/{parent_id}", headers=admin_headers)


def test_cascade_recursive_empty_group_deletion(admin_token):
    """Deleting a group that's the only member of another group cascades deletion."""
    num = random.randint(100000, 999999)
    admin_headers = auth_headers(admin_token)

    # Create grandparent -> parent -> child hierarchy
    grandparent_id = f"g_gp_{num}"
    parent_id = f"g_pp_{num}"
    child_id = f"g_cc_{num}"

    acl_list = [{"permissions": 127, "principals": ["u_root"]}]

    for gid, name in [(grandparent_id, "GP"), (parent_id, "Parent"), (child_id, "Child")]:
        resp = requests.post(
            f"{URL_GLOBAL}/groups",
            json=_make_group_body(gid, name, acl_list),
            headers=admin_headers,
        )
        assert resp.status_code == 201, f"Failed to create {gid}: {resp.text}"

    # Set up the chain: child → parent → grandparent
    resp = requests.post(
        f"{URL_GLOBAL}/memberships",
        json={
            "id": f"{child_id}::{parent_id}",
            "_from": f"groups/{child_id}",
            "_to": f"groups/{parent_id}",
            "principal": child_id,
            "group": parent_id,
        },
        headers=admin_headers,
    )
    assert resp.status_code == 201

    resp = requests.post(
        f"{URL_GLOBAL}/memberships",
        json={
            "id": f"{parent_id}::{grandparent_id}",
            "_from": f"groups/{parent_id}",
            "_to": f"groups/{grandparent_id}",
            "principal": parent_id,
            "group": grandparent_id,
        },
        headers=admin_headers,
    )
    assert resp.status_code == 201

    # after_create auto-added u_root as member of each group.
    # Remove u_root from parent and grandparent so child is their sole member.
    # Don't remove from child (that would trigger empty-group cascade on child).
    requests.delete(f"{URL_GLOBAL}/memberships/u_root::{parent_id}", headers=admin_headers)
    requests.delete(f"{URL_GLOBAL}/memberships/u_root::{grandparent_id}", headers=admin_headers)

    # State: child=[u_root], parent=[child], grandparent=[parent]
    # Delete child → parent becomes empty → cascade deletes parent →
    #   grandparent becomes empty → cascade deletes grandparent
    resp = requests.delete(f"{URL_GLOBAL}/groups/{child_id}", headers=admin_headers)
    assert resp.status_code == 204

    # Parent should be gone (its only member, child, was deleted)
    resp = requests.get(f"{URL_GLOBAL}/groups/{parent_id}", headers=admin_headers)
    assert resp.status_code == 404, (
        f"Expected parent {parent_id} to be deleted (empty), got {resp.status_code}"
    )

    # Grandparent should be gone (its only member, parent, was deleted)
    resp = requests.get(f"{URL_GLOBAL}/groups/{grandparent_id}", headers=admin_headers)
    assert resp.status_code == 404, (
        f"Expected grandparent {grandparent_id} to be deleted (empty), got {resp.status_code}"
    )


# ------------------- Upsert flow --------------------


def test_upsert_creates_group_without_immediate_deletion(regular_user):
    """Groups created via upsert (POST /global/groups/{id}) must NOT be
    immediately deleted by after_update — the after_create hook should fire
    instead, inserting the creator as a member."""
    num = random.randint(100000, 999999)
    group_id = f"g_upsert_{num}"
    headers = auth_headers(regular_user["token"])

    body = _make_group_body(group_id, "Upsert Group")
    # Use the upsert endpoint: POST /global/groups/{id}
    resp = requests.post(
        f"{URL_GLOBAL}/groups/{group_id}",
        json=body,
        headers=headers,
    )
    assert resp.status_code == 200, f"Expected 200, got {resp.status_code}: {resp.text}"

    # Group must still exist (not deleted by after_update)
    resp = requests.get(f"{URL_GLOBAL}/groups/{group_id}", headers=headers)
    assert resp.status_code == 200, (
        f"Group {group_id} should exist after upsert-create, got {resp.status_code}"
    )
    assert resp.json()["name"] == "Upsert Group"

    # Creator should be a member (after_create inserted membership)
    creator_id = regular_user["user_id"]
    membership_key = f"{creator_id}::{group_id}"
    resp = requests.get(f"{URL_GLOBAL}/memberships/{membership_key}", headers=headers)
    assert resp.status_code == 200, (
        f"Creator membership {membership_key} should exist after upsert-create"
    )

    # Cleanup
    requests.delete(f"{URL_GLOBAL}/groups/{group_id}", headers=headers)


def test_upsert_update_does_not_delete_group(regular_user):
    """Upsert-updating an existing group with members must not delete it."""
    num = random.randint(100000, 999999)
    group_id = f"g_upsert_upd_{num}"
    headers = auth_headers(regular_user["token"])

    # Create via normal POST
    resp = requests.post(
        f"{URL_GLOBAL}/groups",
        json=_make_group_body(group_id, "Original Name"),
        headers=headers,
    )
    assert resp.status_code == 201

    # Fetch to get full ACL (with creator injected)
    resp = requests.get(f"{URL_GLOBAL}/groups/{group_id}", headers=headers)
    data = resp.json()

    # Upsert-update with new name
    updated = _make_group_body(group_id, "Upsert Updated", data["acl"]["list"])
    resp = requests.post(
        f"{URL_GLOBAL}/groups/{group_id}",
        json=updated,
        headers=headers,
    )
    assert resp.status_code == 200

    # Group must still exist with updated name
    resp = requests.get(f"{URL_GLOBAL}/groups/{group_id}", headers=headers)
    assert resp.status_code == 200
    assert resp.json()["name"] == "Upsert Updated"

    # Cleanup
    requests.delete(f"{URL_GLOBAL}/groups/{group_id}", headers=headers)


# ------------------- Creator auto-membership --------------------


def test_creator_auto_added_as_member(regular_user):
    """When a group is created, the creator is automatically added as a member."""
    num = random.randint(100000, 999999)
    group_id = f"g_auto_member_{num}"
    headers = auth_headers(regular_user["token"])

    resp = requests.post(
        f"{URL_GLOBAL}/groups",
        json=_make_group_body(group_id, "Auto Member Test"),
        headers=headers,
    )
    assert resp.status_code == 201

    # Verify creator membership edge exists
    creator_id = regular_user["user_id"]
    membership_key = f"{creator_id}::{group_id}"
    resp = requests.get(f"{URL_GLOBAL}/memberships/{membership_key}", headers=headers)
    assert resp.status_code == 200, (
        f"Creator membership {membership_key} should exist, got {resp.status_code}"
    )
    data = resp.json()
    assert data["principal"] == creator_id
    assert data["group"] == group_id

    # Cleanup
    requests.delete(f"{URL_GLOBAL}/groups/{group_id}", headers=headers)


# ------------------- Membership ACL enforcement --------------------


def test_membership_create_requires_group_modify(regular_user, second_user):
    """Users without MODIFY on a group cannot add members to it."""
    num = random.randint(100000, 999999)
    group_id = f"g_memb_acl_{num}"
    creator_headers = auth_headers(regular_user["token"])
    second_headers = auth_headers(second_user["token"])

    # Creator creates the group (gets ROOT ACL)
    resp = requests.post(
        f"{URL_GLOBAL}/groups",
        json=_make_group_body(group_id, "Membership ACL Test"),
        headers=creator_headers,
    )
    assert resp.status_code == 201

    # Second user (not in group ACL) cannot add a membership
    membership_key = f"{second_user['user_id']}::{group_id}"
    resp = requests.post(
        f"{URL_GLOBAL}/memberships",
        json={
            "id": membership_key,
            "_from": f"users/{second_user['user_id']}",
            "_to": f"groups/{group_id}",
            "principal": second_user["user_id"],
            "group": group_id,
        },
        headers=second_headers,
    )
    assert resp.status_code == 404, (
        f"Expected 404 (denied), got {resp.status_code}: {resp.text}"
    )

    # Creator (has MODIFY via ROOT) CAN add a membership
    resp = requests.post(
        f"{URL_GLOBAL}/memberships",
        json={
            "id": membership_key,
            "_from": f"users/{second_user['user_id']}",
            "_to": f"groups/{group_id}",
            "principal": second_user["user_id"],
            "group": group_id,
        },
        headers=creator_headers,
    )
    assert resp.status_code == 201, (
        f"Expected 201 (creator has MODIFY), got {resp.status_code}: {resp.text}"
    )

    # Cleanup
    requests.delete(f"{URL_GLOBAL}/memberships/{membership_key}", headers=creator_headers)
    requests.delete(f"{URL_GLOBAL}/groups/{group_id}", headers=creator_headers)


def test_membership_delete_requires_group_modify(regular_user, second_user):
    """Users without MODIFY on a group cannot remove members from it."""
    num = random.randint(100000, 999999)
    group_id = f"g_memb_del_{num}"
    creator_headers = auth_headers(regular_user["token"])
    second_headers = auth_headers(second_user["token"])

    # Creator creates the group
    resp = requests.post(
        f"{URL_GLOBAL}/groups",
        json=_make_group_body(group_id, "Membership Delete ACL"),
        headers=creator_headers,
    )
    assert resp.status_code == 201

    # Creator adds second user as member
    membership_key = f"{second_user['user_id']}::{group_id}"
    resp = requests.post(
        f"{URL_GLOBAL}/memberships",
        json={
            "id": membership_key,
            "_from": f"users/{second_user['user_id']}",
            "_to": f"groups/{group_id}",
            "principal": second_user["user_id"],
            "group": group_id,
        },
        headers=creator_headers,
    )
    assert resp.status_code == 201

    # Second user (no MODIFY) cannot delete the membership
    resp = requests.delete(
        f"{URL_GLOBAL}/memberships/{membership_key}",
        headers=second_headers,
    )
    assert resp.status_code == 404, (
        f"Expected 404 (denied), got {resp.status_code}: {resp.text}"
    )

    # Creator (has MODIFY) can delete the membership
    resp = requests.delete(
        f"{URL_GLOBAL}/memberships/{membership_key}",
        headers=creator_headers,
    )
    assert resp.status_code == 204

    # Cleanup
    requests.delete(f"{URL_GLOBAL}/groups/{group_id}", headers=creator_headers)


def test_admin_can_manage_memberships_for_any_group(admin_token, regular_user):
    """ADM_USER_MANAGER can add/remove memberships for any group."""
    num = random.randint(100000, 999999)
    group_id = f"g_admin_memb_{num}"
    creator_headers = auth_headers(regular_user["token"])
    admin_headers = auth_headers(admin_token)

    # Regular user creates the group
    resp = requests.post(
        f"{URL_GLOBAL}/groups",
        json=_make_group_body(group_id, "Admin Membership Test"),
        headers=creator_headers,
    )
    assert resp.status_code == 201

    # Admin can add a membership (even though not in group ACL)
    membership_key = f"u_root::{group_id}"
    resp = requests.post(
        f"{URL_GLOBAL}/memberships",
        json={
            "id": membership_key,
            "_from": "users/u_root",
            "_to": f"groups/{group_id}",
            "principal": "u_root",
            "group": group_id,
        },
        headers=admin_headers,
    )
    assert resp.status_code == 201, (
        f"Expected 201 (admin), got {resp.status_code}: {resp.text}"
    )

    # Admin can delete the membership
    resp = requests.delete(
        f"{URL_GLOBAL}/memberships/{membership_key}",
        headers=admin_headers,
    )
    assert resp.status_code == 204

    # Cleanup
    requests.delete(f"{URL_GLOBAL}/groups/{group_id}", headers=creator_headers)
