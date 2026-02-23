CONTAINER ?= $(shell (command -v podman >/dev/null 2>&1 && echo podman) || echo docker)
COMPOSE ?= $(shell (command -v podman-compose >/dev/null 2>&1 && echo podman-compose) || echo docker compose)

.PHONY: run-db stop-db reset-db logs-db

run-db:
	@echo ">>> Using container runtime: $(CONTAINER)"
	@echo ">>> Starting ArangoDB dev instance..."
	$(COMPOSE) up -d

stop-db:
	@echo ">>> Stopping ArangoDB..."
	$(COMPOSE) down

reset-db:
	@echo ">>> Resetting ArangoDB (deleting volumes)..."
	$(COMPOSE) down -v

logs-db:
	@echo ">>> Tailing ArangoDB logs"
	$(COMPOSE) logs -f arangodb

.PHONY: itests itests-seq itests-parallel itests-install

# Install PDM dependencies for integration tests
itests-install:
	@echo ">>> Installing integration test dependencies with PDM..."
	cd itests && pdm install

# Run integration tests sequentially
itests-seq:
	@echo ">>> Running integration tests (sequential)..."
	cd itests && pdm run pytest tests/ -v

# Run integration tests in parallel (uses all available CPU cores)
itests-parallel:
	@echo ">>> Running integration tests (parallel with -n auto)..."
	cd itests && pdm run pytest tests/ -n auto

# Alias for backward compatibility and default behavior
itests: itests-parallel
