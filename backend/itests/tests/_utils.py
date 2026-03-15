"""Shared utilities for integration tests.

This module provides reusable helpers across all test files:
- HTTP URL constants and builders
- HTTP status code constants
- auth_headers() helper
- DebugClient for DB inspection
"""

import requests

# ─── HTTP Constants ───────────────────────────────────────────────────────────

BASE = "http://localhost:3742/api"
URL_REGISTER = f"{BASE}/v1/register"
URL_LOGIN = f"{BASE}/v1/login"
URL_GLOBAL = f"{BASE}/v1/global"
URL_DEBUG = f"{BASE}/v1/debug"
URL_WS = "ws://localhost:3742/api/v1/ws"

# HTTP Status Codes
STATUS_OK = 200
STATUS_CREATED = 201
STATUS_NO_CONTENT = 204
STATUS_BAD_REQUEST = 400
STATUS_UNAUTHORIZED = 401
STATUS_CONFLICT = 409
STATUS_NOT_FOUND = 404
STATUS_UNPROCESSABLE = 422

# ─── Helper Functions ─────────────────────────────────────────────────────────


def auth_headers(token: str) -> dict:
    """Build Authorization header for API requests."""
    return {"Authorization": f"Bearer {token}", "Content-Type": "application/json"}


def url_project(project_id: str) -> str:
    """Build URL to a project resource."""
    return f"{URL_GLOBAL}/projects/{project_id}"


def url_scoped(project_id: str, kind: str, resource_id: str = "") -> str:
    """Build URL to a scoped resource under a project.

    Args:
        project_id: Parent project ID
        kind: Resource kind (e.g. 'ticketgroups')
        resource_id: Optional resource ID; omit for list/create endpoints

    Returns:
        Full URL path for the scoped resource
    """
    base = f"{BASE}/v1/projects/{project_id}/{kind}"
    return f"{base}/{resource_id}" if resource_id else base


# ─── DebugClient ──────────────────────────────────────────────────────────────


class DebugClient:
    """HTTP wrapper around /v1/debug endpoints for test diagnostics.

    Requires a token with ADM_GODMODE (root token).
    Use dump() in pytest.fail() messages to show raw DB state on failures.

    Example:
        debug = DebugClient(admin_token)
        doc = debug.assert_exists("ticketgroups", _key="myproj__BUG")
        debug.assert_soft_deleted("ticketgroups", "myproj__BUG")
    """

    def __init__(self, root_token: str, base: str = BASE):
        self._headers = auth_headers(root_token)
        self._base = base

    def list_collections(self) -> list[str]:
        """Return names of all non-system collections."""
        resp = requests.get(f"{self._base}/v1/debug/collections", headers=self._headers)
        assert (
            resp.status_code == STATUS_OK
        ), f"debug/collections failed: {resp.text}"
        return [c["name"] for c in resp.json()["collections"]]

    def dump(self, collection: str) -> list[dict]:
        """Return all raw documents in a collection (as stored in ArangoDB)."""
        resp = requests.get(
            f"{self._base}/v1/debug/collections/{collection}",
            headers=self._headers,
        )
        assert (
            resp.status_code == STATUS_OK
        ), f"debug dump of '{collection}' failed: {resp.text}"
        body = resp.json()
        return body["documents"]

    def count(self, collection: str) -> int:
        """Return document count for a collection."""
        resp = requests.get(
            f"{self._base}/v1/debug/collections/{collection}",
            headers=self._headers,
        )
        assert (
            resp.status_code == STATUS_OK
        ), f"debug count of '{collection}' failed: {resp.text}"
        return resp.json()["count"]

    def find(self, collection: str, **field_matchers) -> list[dict]:
        """Return documents where all field_matchers match (equality).

        Example:
            debug.find("ticketgroups", project="myproj")
            debug.find("users", _key="u_alice")
        """
        docs = self.dump(collection)
        result = []
        for doc in docs:
            if all(doc.get(k) == v for k, v in field_matchers.items()):
                result.append(doc)
        return result

    def find_one(self, collection: str, **field_matchers) -> dict | None:
        """Like find() but returns first match or None."""
        matches = self.find(collection, **field_matchers)
        return matches[0] if matches else None

    def assert_exists(self, collection: str, msg: str = "", **field_matchers) -> dict:
        """Assert a document matching field_matchers exists; return it."""
        doc = self.find_one(collection, **field_matchers)
        if doc is None:
            all_docs = self.dump(collection)
            raise AssertionError(
                f"{msg or 'Expected document not found'}\n"
                f"Filters: {field_matchers}\n"
                f"Collection '{collection}' contents ({len(all_docs)} docs):\n"
                + "\n".join(f"  {d}" for d in all_docs)
            )
        return doc

    def assert_not_exists(
        self, collection: str, msg: str = "", **field_matchers
    ) -> None:
        """Assert no document matching field_matchers exists."""
        doc = self.find_one(collection, **field_matchers)
        if doc is not None:
            raise AssertionError(
                f"{msg or 'Document unexpectedly found'}\n"
                f"Filters: {field_matchers}\n"
                f"Found: {doc}"
            )

    def assert_soft_deleted(self, collection: str, key: str) -> dict:
        """Assert document exists AND has a deletion marker (soft-deleted).

        Use this instead of asserting 404 from the API when you need to
        verify that soft-delete actually wrote the deletion field to the DB.
        """
        docs = self.dump(collection)
        for doc in docs:
            if doc.get("_key") == key:
                assert (
                    "deletion" in doc and doc["deletion"] is not None
                ), (
                    f"Document {key} in '{collection}' exists but is NOT soft-deleted.\n"
                    f"Document: {doc}"
                )
                return doc
        raise AssertionError(
            f"Document {key} not found in '{collection}' at all (not even soft-deleted).\n"
            f"Keys present: {[d.get('_key') for d in docs]}"
        )

    def snapshot(self, *collections: str) -> dict[str, list[dict]]:
        """Dump multiple collections at once; useful for before/after comparisons."""
        return {col: self.dump(col) for col in collections}
