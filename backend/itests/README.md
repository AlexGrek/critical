# Critical Backend Integration Tests

Python integration tests for the Critical backend API using pytest.

## Prerequisites

- Backend running on `localhost:3742`
- ArangoDB running and accessible to the backend
- PDM installed (`pip install pdm`)

## Installation

Install test dependencies with PDM:

```bash
cd backend/itests
pdm install
```

## Running Tests

### Sequential Execution
Run all tests serially:

```bash
cd backend/itests
pdm run pytest tests/ -v
```

### Parallel Execution with pytest-xdist
Run tests in parallel using all available CPU cores:

```bash
cd backend/itests
pdm run pytest tests/ -n auto
```

Specify a fixed number of workers instead of auto-detection:

```bash
pdm run pytest tests/ -n 4
```

### Other Useful Options

Run tests with output from failing tests only:
```bash
pdm run pytest tests/ -n auto -x
```

Run a specific test file in parallel:
```bash
pdm run pytest tests/auth_test.py -n auto
```

Run a specific test in parallel:
```bash
pdm run pytest tests/auth_test.py::test_1_register_user -n auto
```

## Test Organization

Test files follow the naming convention `*_test.py`:
- `auth_test.py` - Authentication and login flows
- `gitops_test.py` - Gitops API operations
- `group_test.py` - Group management
- `gitops_permissions_test.py` - Permission-based gitops operations
- `pagination_test.py` - Pagination in list endpoints

## Writing Parallelizable Tests

Tests are designed to run in parallel by using unique identifiers:

1. **Use random IDs**: Each test creates resources with unique IDs using `random.randint()`
   ```python
   num = random.randint(100000, 999999)
   user_id = f"u_gitops_{num}"
   ```

2. **Fixture scope**: Fixtures use `scope="module"` so each test module has its own isolated setup

3. **Cleanup**: Always clean up created resources at the end of tests to avoid DB bloat:
   ```python
   resp = requests.delete(f"{URL_GLOBAL}/users/{user_id}", headers=auth_headers(token))
   assert resp.status_code == 204
   ```

4. **No shared state**: Avoid depending on state created by other tests; each test should be independent

## Troubleshooting

### Tests fail with "Connection refused"
Ensure the backend is running on `localhost:3742`:
```bash
make run  # From project root
```

### Tests fail with database errors
Check that ArangoDB is running and the backend is connected properly

### Intermittent failures in parallel mode
- Verify test isolation (each test should create unique resources)
- Check for race conditions or timing issues
- Review test logs to identify patterns

## Configuration

The pytest configuration is defined in `pyproject.toml`:

```toml
[tool.pytest.ini_options]
addopts = "-v --tb=short"
testpaths = ["tests"]
python_files = "*_test.py"

[tool.pytest.xdist]
worker_restart_threshold = 100
```

Modify these settings to customize test behavior. For example, change `addopts` to adjust verbosity or traceback formatting.
