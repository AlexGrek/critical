COMPOSE ?= $(shell (command -v docker >/dev/null 2>&1 && echo "docker compose") || echo podman-compose)

API_IMAGE   ?= grekodocker/cr1t-api
FRONT_IMAGE ?= grekodocker/cr1t-frontend
TAG         ?= latest
PLATFORMS   ?= linux/amd64,linux/arm64

.PHONY: build build-api build-frontend build-push push-api push-frontend \
        up down logs status reset wait \
        helm-deploy helm-status helm-uninstall helm-template

# --- Build (local, current arch only) ---

build: build-api build-frontend

build-api:
	@echo ">>> Building API image..."
	docker build -t $(API_IMAGE):$(TAG) -f ../backend/Dockerfile ..

build-frontend:
	@echo ">>> Building frontend image..."
	docker build -t $(FRONT_IMAGE):$(TAG) -f ../frontend/Dockerfile ../frontend

# --- Build + Push (multi-arch via buildx) ---

build-push: push-api push-frontend

push-api:
	@echo ">>> Building + pushing API image ($(PLATFORMS))..."
	docker buildx build \
		--platform $(PLATFORMS) \
		-t $(API_IMAGE):$(TAG) \
		-f ../backend/Dockerfile \
		--push ..

push-frontend:
	@echo ">>> Building + pushing frontend image ($(PLATFORMS))..."
	docker buildx build \
		--platform $(PLATFORMS) \
		-t $(FRONT_IMAGE):$(TAG) \
		-f ../frontend/Dockerfile \
		--push ../frontend

# --- Compose lifecycle ---

up:
	@echo ">>> Starting stack..."
	$(COMPOSE) up -d
	@$(MAKE) wait
	@echo ">>> Stack is ready at http://localhost:$${GATEWAY_PORT:-8080}"

down:
	@echo ">>> Stopping stack..."
	$(COMPOSE) down

logs:
	$(COMPOSE) logs -f

status:
	$(COMPOSE) ps

reset:
	@echo ">>> Tearing down stack and removing volumes..."
	$(COMPOSE) down -v

# --- Health checks ---

wait:
	@echo ">>> Waiting for services..."
	@for i in $$(seq 1 30); do \
		curl -sf http://localhost:$${GATEWAY_PORT:-8080}/health >/dev/null 2>&1 && break; \
		sleep 2; \
	done
	@curl -sf http://localhost:$${GATEWAY_PORT:-8080}/health >/dev/null 2>&1 \
		|| { echo ">>> Services failed to start"; exit 1; }
	@echo ">>> All services healthy"

# --- Helm ---

HELM_RELEASE ?= critical
HELM_NS      ?= critical-dev
HELM_CHART   ?= helm/critical
HELM_VALUES  ?= helm/values-dev.yaml

helm-deploy:
	@echo ">>> Deploying $(HELM_RELEASE) to namespace $(HELM_NS)..."
	helm upgrade --install $(HELM_RELEASE) $(HELM_CHART) \
		--namespace $(HELM_NS) --create-namespace \
		-f $(HELM_VALUES)

helm-status:
	helm status $(HELM_RELEASE) -n $(HELM_NS)

helm-uninstall:
	helm uninstall $(HELM_RELEASE) -n $(HELM_NS)

helm-template:
	helm template $(HELM_RELEASE) $(HELM_CHART) -f $(HELM_VALUES)
