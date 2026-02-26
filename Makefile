# Detect container runtime (docker preferred, fallback to podman)
COMPOSE ?= $(shell (command -v docker >/dev/null 2>&1 && echo "docker compose") || echo podman-compose)
BACKEND_PORT ?= 3742
BACKEND_URL ?= http://localhost:$(BACKEND_PORT)

.PHONY: dev dev-api dev-frontend run run-fresh test test-unit test-cli test-api run-db stop-db reset-db logs-db wait-db wait-backend itests itests-seq itests-parallel

dev:
	@bash -c ' \
		set -m; \
		trap "kill %1 %2 %3 2>/dev/null; wait 2>/dev/null; echo \">>> Development environment stopped.\"" EXIT INT TERM; \
		$(MAKE) run-db & \
		$(MAKE) dev-api & \
		$(MAKE) dev-frontend & \
		wait \
	'

dev-api:
	@echo ">>> Starting backend (cargo watch)..."
	cd backend && cargo watch -x run

dev-frontend:
	@echo ">>> Starting frontend dev server..."
	cd frontend && npm run dev

wait-db:
	@echo ">>> Waiting for ArangoDB to be ready..."
	@for i in $$(seq 1 30); do \
		curl -sf -u root:devpassword http://localhost:8529/_api/version >/dev/null 2>&1 && break; \
		sleep 1; \
	done
	@curl -sf -u root:devpassword http://localhost:8529/_api/version >/dev/null 2>&1 || { echo ">>> ArangoDB failed to start"; exit 1; }
	@echo ">>> ArangoDB is ready"

wait-backend:
	@echo ">>> Waiting for backend to be ready..."
	@for i in $$(seq 1 30); do \
		curl -sf $(BACKEND_URL)/health >/dev/null 2>&1 && break; \
		sleep 1; \
	done
	@curl -sf $(BACKEND_URL)/health >/dev/null 2>&1 || { echo ">>> Backend failed to start"; exit 1; }
	@echo ">>> Backend is ready"

run:
	@echo ">>> Starting ArangoDB (persistent)..."
	@$(COMPOSE) up -d
	@$(MAKE) wait-db
	@echo ">>> Building and running backend..."
	@trap '$(COMPOSE) stop; echo ">>> ArangoDB stopped."' EXIT; \
		cd backend && cargo run

run-fresh: reset-db run

# --- Test targets ---

# --- Internal: start backend in background, wait for it, set trap for cleanup ---
# Usage: $(_start_backend) inside a shell block
# After this, BACKEND_PID is set and trap ensures cleanup of both backend and DB.
# NOTE: We run the binary directly (not `cargo run`) so that BACKEND_PID points to
# the actual axum-api process. `cargo run` spawns a child, and killing cargo doesn't
# reliably kill the child — leaving orphaned backend processes that block future runs.
define _start_backend
	cargo build --bin axum-api && \
	BACKEND_LOG=$$(mktemp /tmp/axum-api-XXXXXX); \
	echo ">>> Backend log: $$BACKEND_LOG"; \
	(cd backend && ../target/debug/axum-api) > $$BACKEND_LOG 2>&1 & BACKEND_PID=$$!; \
	trap 'kill $$BACKEND_PID 2>/dev/null; wait $$BACKEND_PID 2>/dev/null; $(COMPOSE) down -v; echo ">>> Cleaned up. Backend log: $$BACKEND_LOG"' EXIT; \
	echo ">>> Waiting for backend..."; \
	for i in $$(seq 1 30); do \
		curl -sf $(BACKEND_URL)/health >/dev/null 2>&1 && break; \
		sleep 1; \
	done; \
	curl -sf $(BACKEND_URL)/health >/dev/null 2>&1 || { echo ">>> Backend failed to start"; exit 1; }; \
	echo ">>> Backend is ready"
endef

# Run everything: Rust unit/integration tests, CLI integration tests, Python API tests
test:
	@echo ">>> Starting ephemeral ArangoDB..."
	@$(COMPOSE) up -d
	@$(MAKE) wait-db
	@trap '$(COMPOSE) down -v; echo ">>> Ephemeral ArangoDB removed."' EXIT; \
		echo ">>> [1/3] Running Rust unit & backend integration tests..." && \
		cargo test -p axum-api -p crit-cli && \
		echo ">>> [2/3] Starting backend for CLI & API integration tests..." && \
		$(_start_backend) && \
		echo ">>> Running CLI integration tests..." && \
		cargo test -p crit-cli --test cli_test -- --include-ignored && \
		echo ">>> [3/3] Running Python API integration tests (parallel)..." && \
		cd backend/itests && pdm run pytest tests/ -n auto && cd ../.. && \
		echo ">>> All tests passed."

# Rust unit tests + backend integration tests (via axum-test, no backend process needed)
test-unit:
	@echo ">>> Starting ephemeral ArangoDB..."
	@$(COMPOSE) up -d
	@$(MAKE) wait-db
	@echo ">>> Running Rust tests..."
	@trap '$(COMPOSE) down -v; echo ">>> Ephemeral ArangoDB removed."' EXIT; \
		cargo test -p axum-api -p crit-cli

# CLI integration tests (requires DB + backend running)
test-cli:
	@echo ">>> Starting ephemeral ArangoDB..."
	@$(COMPOSE) up -d
	@$(MAKE) wait-db
	@echo ">>> Starting backend..."
	@trap '$(COMPOSE) down -v; echo ">>> Ephemeral ArangoDB removed."' EXIT; \
		$(_start_backend) && \
		echo ">>> Running CLI integration tests..." && \
		cargo test -p crit-cli --test cli_test -- --include-ignored --test-threads=1 && \
		echo ">>> CLI tests passed."

# Python API integration tests (requires DB + backend running)
test-api:
	@echo ">>> Starting ephemeral ArangoDB..."
	@$(COMPOSE) up -d
	@$(MAKE) wait-db
	@echo ">>> Starting backend..."
	@trap '$(COMPOSE) down -v; echo ">>> Ephemeral ArangoDB removed."' EXIT; \
		$(_start_backend) && \
		echo ">>> Running Python API tests (parallel)..." && \
		cd backend/itests && pdm run pytest tests/ -n auto && cd ../.. && \
		echo ">>> API tests passed."

# --- Integration test targets (convenience wrappers) ---

# Run Python integration tests in parallel (requires backend running on $(BACKEND_URL))
itests: itests-parallel

itests-parallel:
	@$(MAKE) -C backend itests-parallel

itests-seq:
	@$(MAKE) -C backend itests-seq

itests-install:
	@$(MAKE) -C backend itests-install

# --- DB management ---

run-db:
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
