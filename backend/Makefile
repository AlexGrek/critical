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

.PHONY: itests

itests:
	@if [ ! -d ".venv" ]; then \
		python3 -m venv .venv; \
	fi
	@. .venv/bin/activate && \
		pip install --upgrade pip && \
		pip install requests pytest && \
		pip install websockets && \
		pytest itests/tests
