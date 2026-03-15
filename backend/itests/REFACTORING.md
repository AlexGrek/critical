# Integration Tests Refactoring

## Overview

Refactored `backend/itests/tests/ticket_groups_test.py` to improve code reusability, maintainability, and reduce duplication.

## Changes Made

### 1. Created `_utils.py` — Shared Utilities Module

Extracted common utilities used across test files:

**URL Constants:**
- `BASE`, `URL_REGISTER`, `URL_LOGIN`, `URL_GLOBAL`, `URL_DEBUG`, `URL_WS`

**HTTP Status Code Constants:**
- `STATUS_OK`, `STATUS_CREATED`, `STATUS_NO_CONTENT`, `STATUS_BAD_REQUEST`, `STATUS_UNAUTHORIZED`, `STATUS_CONFLICT`, `STATUS_NOT_FOUND`, `STATUS_UNPROCESSABLE`

**Helper Functions:**
- `auth_headers(token)` — builds Authorization header
- `url_project(project_id)` — builds project resource URL
- `url_scoped(project_id, kind, resource_id)` — builds scoped resource URL

**DebugClient Class:**
- Centralized database inspection utilities for all tests
- Methods: `dump()`, `find()`, `find_one()`, `assert_exists()`, `assert_soft_deleted()`, `snapshot()`
- Eliminates duplicate DebugClient implementations across test files

### 2. Refactored `ticket_groups_test.py`

**Imports:**
- Replaced inline imports with centralized `_utils` imports
- Removed duplicate `DebugClient` class definition
- Removed duplicate URL constant definitions

**Test Data Constants:**
- Extracted all hardcoded test IDs and data into module-level constants:
  - `VALID_2_LETTER`, `VALID_3_LETTER`, `VALID_4_LETTER`, `VALID_5_LETTER`, `VALID_6_LETTER`
  - `INVALID_1_LETTER`, `INVALID_7_LETTER`, `INVALID_LOWERCASE`, `INVALID_WITH_NUMBERS`
  - `PREFIXED_ID`
  - `SAMPLE_TICKET_TYPE`, `SIMPLE_TICKET_TYPE` — reusable ticket type fixtures

**Fixtures:**
- Added docstrings to all fixtures
- Fixed `test_project` fixture to use correct endpoint:
  - Creation: `POST /v1/global/projects` (without ID in path, ID in body) → returns 201
  - Deletion: `DELETE /v1/global/projects/{id}` (with ID in path) → returns 204

**Test Functions:**
- Replaced hardcoded strings with constants (e.g., `"BG"` → `VALID_2_LETTER`)
- Replaced magic numbers with HTTP status constants:
  - `201` → `STATUS_CREATED`
  - `200` → `STATUS_OK`
  - `204` → `STATUS_NO_CONTENT`
  - `400` → `STATUS_BAD_REQUEST`
  - `404` → `STATUS_NOT_FOUND`
  - `401` → `STATUS_UNAUTHORIZED`
  - `422` → `STATUS_UNPROCESSABLE`
- Simplified ticket type references (use `SAMPLE_TICKET_TYPE` instead of inline dict)

**URL Helpers:**
- Added `url_tg()` wrapper around `url_scoped()` for cleaner ticket group endpoint access

## Benefits

1. **Reduced Code Duplication:** DebugClient can now be used across all test files
2. **Improved Maintainability:** URL constants and HTTP status codes defined once, used everywhere
3. **Easier Test Data Management:** Test data constants grouped at module top for easy modification
4. **Better Readability:** `STATUS_CREATED` is clearer than magic number `201`
5. **Consistent Patterns:** All tests can now follow the same utilities pattern
6. **Easier Debugging:** Centralized DebugClient means bug fixes propagate to all tests

## Future Applications

This pattern can be extended to other test files:
- Move `DebugClient` imports to existing test files via `_utils`
- Extract HTTP status constants from hardcoded values
- Create test data factories for common resource types
- Build URL helper functions for each scoped resource kind

## Example Migration

Old test:
```python
class DebugClient:
    def dump(self, collection): ...

resp = requests.post(f"{BASE}/v1/global/groups",
                    json={"id": gid, "name": "Test"},
                    headers={"Authorization": f"Bearer {token}"})
assert resp.status_code == 201
```

New test:
```python
from _utils import DebugClient, auth_headers, URL_GLOBAL, STATUS_CREATED

resp = requests.post(f"{URL_GLOBAL}/groups",
                    json={"id": gid, "name": "Test"},
                    headers=auth_headers(token))
assert resp.status_code == STATUS_CREATED
```

## Files Modified

- `backend/itests/tests/_utils.py` ✨ (NEW)
- `backend/itests/tests/ticket_groups_test.py` ♻️ (REFACTORED)

## Test Results

All 23 ticket_groups_test.py tests pass ✅
All 161 integration tests pass ✅
