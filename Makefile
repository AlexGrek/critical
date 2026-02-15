# Detect container runtime (docker preferred, fallback to podman)
COMPOSE ?= $(shell (command -v docker >/dev/null 2>&1 && echo "docker compose") || echo podman-compose)

.PHONY: dev run run-fresh test run-db stop-db reset-db logs-db wait-db

dev:
	@echo ">>> Building workspace (dev)..."
	cargo build

wait-db:
	@echo ">>> Waiting for ArangoDB to be ready..."
	@for i in $$(seq 1 30); do \
		curl -sf -u root:devpassword http://localhost:8529/_api/version >/dev/null 2>&1 && break; \
		sleep 1; \
	done
	@curl -sf -u root:devpassword http://localhost:8529/_api/version >/dev/null 2>&1 || { echo ">>> ArangoDB failed to start"; exit 1; }
	@echo ">>> ArangoDB is ready"

run:
	@echo ">>> Starting ArangoDB (persistent)..."
	@$(COMPOSE) up -d
	@$(MAKE) wait-db
	@echo ">>> Building and running backend..."
	@trap '$(COMPOSE) stop; echo ">>> ArangoDB stopped."' EXIT; \
		cd backend && cargo run

run-fresh: reset-db run

test:
	@echo ">>> Starting ephemeral ArangoDB..."
	@$(COMPOSE) up -d
	@$(MAKE) wait-db
	@echo ">>> Running tests..."
	@trap '$(COMPOSE) down -v; echo ">>> Ephemeral ArangoDB removed."' EXIT; \
		cargo test

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
