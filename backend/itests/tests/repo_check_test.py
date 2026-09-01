"""Integration tests for repo_credentials CRUD and the project repository-check endpoint.

Endpoint base:
  /api/v1/global/repo_credentials
  /api/v1/global/projects/{id}/repocheck

Tests cover:
- repo_credentials CRUD: secret/passphrase are write-only (never returned), has_secret flag
- Updating a credential without resending `secret` preserves the stored value (merge semantics)
- ID auto-prefixing (rc_)
- /repocheck: 404 on unknown project, denied project access, and denied credential access
- /repocheck: 400 on a malformed/unsupported repository URL
- /repocheck: 404 for an unsupported {kind} (only "projects" is supported)

Network-dependent scenarios (actually reaching a real repository) are intentionally not
covered here to keep the suite hermetic; see test_repocheck_public_https_repo, skipped
unless REPO_CHECK_NETWORK_TESTS=1 is set in the environment.
"""

import os
import random

import pytest
import requests

from _utils import (
    DebugClient,
    STATUS_BAD_REQUEST,
    STATUS_CREATED,
    STATUS_NOT_FOUND,
    STATUS_OK,
    URL_GLOBAL,
    URL_LOGIN,
    URL_REGISTER,
    auth_headers,
    url_project,
)

# ─── Helper Functions ─────────────────────────────────────────────────────────


def _register_and_login(prefix: str) -> dict:
    num = random.randint(100000, 999999)
    username = f"{prefix}_{num}"
    requests.post(URL_REGISTER, json={"user": username, "password": username})
    resp = requests.post(URL_LOGIN, json={"user": username, "password": username})
    assert resp.status_code == STATUS_OK, f"Login failed for {username}: {resp.text}"
    return {"user_id": f"u_{username}", "token": resp.json()["token"]}


def url_credential(cred_id: str) -> str:
    return f"{URL_GLOBAL}/repo_credentials/{cred_id}"


def url_repocheck(project_id: str) -> str:
    return f"{URL_GLOBAL}/projects/{project_id}/repocheck"


# ─── Fixtures ─────────────────────────────────────────────────────────────────


@pytest.fixture(scope="module")
def admin_token():
    """Root token — has ADM_GODMODE."""
    resp = requests.post(URL_LOGIN, json={"user": "root", "password": "changeme"})
    assert resp.status_code == STATUS_OK, f"Root login failed: {resp.text}"
    return resp.json()["token"]


@pytest.fixture(scope="module")
def project_owner(admin_token):
    """Random user granted usr_create_projects, so they can create projects & credentials."""
    user = _register_and_login("repochk")
    resp = requests.post(
        f"{URL_GLOBAL}/permissions/usr_create_projects/grant",
        json={"principals": [user["user_id"]]},
        headers=auth_headers(admin_token),
    )
    assert resp.status_code == STATUS_OK, f"Grant failed: {resp.text}"
    return user


@pytest.fixture(scope="module")
def outsider(admin_token):
    """Random user with no special permissions and no ACL on anything created here."""
    return _register_and_login("repochk_out")


@pytest.fixture(scope="module")
def test_project(project_owner):
    pid = f"repochk_{random.randint(100000, 999999)}"
    headers = auth_headers(project_owner["token"])
    resp = requests.post(
        f"{URL_GLOBAL}/projects",
        json={"id": pid, "name": "Repo Check Test Project"},
        headers=headers,
    )
    assert resp.status_code == STATUS_CREATED, f"Create project failed: {resp.text}"
    yield pid
    requests.delete(url_project(pid), headers=headers)


@pytest.fixture(scope="module")
def test_credential(project_owner):
    """A credential owned by project_owner; outsider has no ACL on it."""
    headers = auth_headers(project_owner["token"])
    resp = requests.post(
        f"{URL_GLOBAL}/repo_credentials",
        json={
            "id": f"repochk_{random.randint(100000, 999999)}",
            "name": "Test SSH key",
            "method": "ssh",
            "secret": "-----BEGIN OPENSSH PRIVATE KEY-----\nFAKEKEYDATA\n-----END OPENSSH PRIVATE KEY-----\n",
        },
        headers=headers,
    )
    assert resp.status_code == STATUS_CREATED, f"Create credential failed: {resp.text}"
    cred_id = resp.json()["id"]
    yield cred_id
    requests.delete(url_credential(cred_id), headers=headers)


@pytest.fixture(scope="module")
def debug(admin_token) -> DebugClient:
    return DebugClient(admin_token)


# ─── repo_credentials CRUD ─────────────────────────────────────────────────────


def test_create_credential_id_gets_rc_prefix(project_owner):
    headers = auth_headers(project_owner["token"])
    raw_id = f"repochk_{random.randint(100000, 999999)}"
    resp = requests.post(
        f"{URL_GLOBAL}/repo_credentials",
        json={"id": raw_id, "name": "prefix test", "method": "github_token", "secret": "ghp_faketoken"},
        headers=headers,
    )
    assert resp.status_code == STATUS_CREATED, resp.text
    body = resp.json()
    assert body["id"] == f"rc_{raw_id}"
    requests.delete(url_credential(body["id"]), headers=headers)


def test_create_credential_never_returns_secret(project_owner):
    headers = auth_headers(project_owner["token"])
    resp = requests.post(
        f"{URL_GLOBAL}/repo_credentials",
        json={
            "id": f"repochk_{random.randint(100000, 999999)}",
            "name": "ephemeral",
            "method": "github_token",
            "secret": "ghp_supersecrettoken",
        },
        headers=headers,
    )
    assert resp.status_code == STATUS_CREATED, resp.text
    body = resp.json()
    assert "secret" not in body
    assert "passphrase" not in body
    assert body["has_secret"] is True
    requests.delete(url_credential(body["id"]), headers=headers)


def test_get_credential_never_returns_secret(test_credential, project_owner):
    resp = requests.get(url_credential(test_credential), headers=auth_headers(project_owner["token"]))
    assert resp.status_code == STATUS_OK, resp.text
    body = resp.json()
    assert "secret" not in body
    assert "passphrase" not in body
    assert body["has_secret"] is True


def test_update_without_secret_preserves_stored_secret(test_credential, project_owner, debug):
    """PUT is a merge (UPDATE ... WITH), so omitting `secret` on an edit must not
    erase the stored value — this is the mechanism the write-only-secret UX relies on."""
    headers = auth_headers(project_owner["token"])
    get_resp = requests.get(url_credential(test_credential), headers=headers)
    assert get_resp.status_code == STATUS_OK
    doc = get_resp.json()
    assert "secret" not in doc  # sanity: API really doesn't hand it back

    put_resp = requests.put(
        url_credential(test_credential),
        json={**doc, "description": "updated via test"},
        headers=headers,
    )
    assert put_resp.status_code == STATUS_OK, put_resp.text
    # NOTE: the PUT response is the doc_snapshot of what the caller sent (see
    # docs/gitops-controller.md), not a fresh DB read — since `secret` wasn't
    # in the request body, `has_secret` is correctly False *in this response*
    # even though the DB still has it. A follow-up GET (like the frontend's
    # revalidate) is what proves the secret actually survived.
    refetch_resp = requests.get(url_credential(test_credential), headers=headers)
    assert refetch_resp.status_code == STATUS_OK
    assert refetch_resp.json()["has_secret"] is True

    raw = debug.assert_exists(
        "repo_credentials",
        msg="credential missing from DB",
        _key=test_credential,
    )
    assert raw.get("secret"), "secret should still be present in the DB after an update that omitted it"


def test_outsider_cannot_read_credential(test_credential, outsider):
    resp = requests.get(url_credential(test_credential), headers=auth_headers(outsider["token"]))
    assert resp.status_code == STATUS_NOT_FOUND, resp.text


# ─── /repocheck access control ─────────────────────────────────────────────────


def test_repocheck_unknown_project_404(project_owner):
    resp = requests.post(
        url_repocheck(f"does-not-exist-{random.randint(100000, 999999)}"),
        json={"url": "https://github.com/octocat/Hello-World", "provider": "github"},
        headers=auth_headers(project_owner["token"]),
    )
    assert resp.status_code == STATUS_NOT_FOUND, resp.text


def test_repocheck_outsider_cannot_check_project(test_project, outsider):
    """Caller with no ACL on the project gets 404, not 403 (existence isn't leaked)."""
    resp = requests.post(
        url_repocheck(test_project),
        json={"url": "https://github.com/octocat/Hello-World", "provider": "github"},
        headers=auth_headers(outsider["token"]),
    )
    assert resp.status_code == STATUS_NOT_FOUND, resp.text


def test_repocheck_denied_credential_404(test_project, test_credential, outsider, project_owner):
    """outsider has MODIFY on nothing here, but even a caller who *can* write the
    project must not be able to probe using a credential they can't read.
    Grant outsider MODIFY on test_project, then confirm the credential (owned by
    project_owner) still blocks the request."""
    headers_owner = auth_headers(project_owner["token"])
    proj_resp = requests.get(url_project(test_project), headers=headers_owner)
    assert proj_resp.status_code == STATUS_OK
    proj = proj_resp.json()
    acl = proj.get("acl", {"list": []})
    acl["list"] = acl.get("list", []) + [{"permissions": 31, "principals": [outsider["user_id"]]}]
    put_resp = requests.put(
        url_project(test_project),
        json={**proj, "acl": acl},
        headers=headers_owner,
    )
    assert put_resp.status_code == STATUS_OK, put_resp.text

    resp = requests.post(
        url_repocheck(test_project),
        json={
            "url": "git@github.com:example/example.git",
            "provider": "git",
            "auth_method": "ssh",
            "credential": test_credential,
        },
        headers=auth_headers(outsider["token"]),
    )
    assert resp.status_code == STATUS_NOT_FOUND, resp.text


def test_repocheck_unsupported_kind_404(test_project, project_owner):
    """Only kind == 'projects' is supported at this route today."""
    resp = requests.post(
        f"{URL_GLOBAL}/groups/{test_project}/repocheck",
        json={"url": "https://github.com/octocat/Hello-World", "provider": "github"},
        headers=auth_headers(project_owner["token"]),
    )
    assert resp.status_code == STATUS_NOT_FOUND, resp.text


def test_repocheck_malformed_url_400(test_project, project_owner):
    resp = requests.post(
        url_repocheck(test_project),
        json={"url": "not a url at all", "provider": "git"},
        headers=auth_headers(project_owner["token"]),
    )
    assert resp.status_code == STATUS_BAD_REQUEST, resp.text


def test_repocheck_file_scheme_rejected_400(test_project, project_owner):
    resp = requests.post(
        url_repocheck(test_project),
        json={"url": "file:///etc/passwd", "provider": "custom"},
        headers=auth_headers(project_owner["token"]),
    )
    assert resp.status_code == STATUS_BAD_REQUEST, resp.text


def test_repocheck_ssh_without_credential_400(test_project, project_owner):
    """auth_method: ssh with no credential attached is a request-shape error, not
    something the probe should attempt and then report as a connection failure."""
    resp = requests.post(
        url_repocheck(test_project),
        json={"url": "git@github.com:example/example.git", "provider": "git", "auth_method": "ssh"},
        headers=auth_headers(project_owner["token"]),
    )
    assert resp.status_code == STATUS_BAD_REQUEST, resp.text


# ─── Opt-in network test ───────────────────────────────────────────────────────


@pytest.mark.skipif(
    os.environ.get("REPO_CHECK_NETWORK_TESTS") != "1",
    reason="hits a real public GitHub repo; set REPO_CHECK_NETWORK_TESTS=1 to enable",
)
def test_repocheck_public_https_repo(test_project, project_owner):
    resp = requests.post(
        url_repocheck(test_project),
        json={
            "url": "https://github.com/octocat/Hello-World",
            "provider": "github",
            "default_branch": "master",
            "auth_method": "none",
        },
        headers=auth_headers(project_owner["token"]),
    )
    assert resp.status_code == STATUS_OK, resp.text
    body = resp.json()
    assert body["status"] in ("found", "missing")
