# Deployment

## Docker Compose

The `dist/` directory contains a prod-like Docker Compose stack.

### Architecture

```
        :8080
          |
      [ gateway ]  (nginx:alpine)
       /        \
  /api/*         /*
    |              |
 [ api ]     [ frontend ]
(cr1t-api)  (cr1t-frontend)
    |
[ arangodb ]
```

### Make Targets (from `dist/`)

| Target | Description |
|--------|-------------|
| `make build` | Build both images locally (current arch) |
| `make build-api` | Build API image only |
| `make build-frontend` | Build frontend image only |
| `make build-push` | Build multi-arch (amd64+arm64) and push to Docker Hub |
| `make up` | Start the stack, wait for health checks |
| `make down` | Stop the stack |
| `make logs` | Tail logs from all services |
| `make status` | Show running containers |
| `make reset` | Stop and remove all volumes (clean slate) |

### Environment Variables

Set in shell or `.env` file next to `docker-compose.yml`:

| Variable | Default | Description |
|----------|---------|-------------|
| `TAG` | `latest` | Image tag |
| `GATEWAY_PORT` | `8080` | Exposed port |
| `DB_PASSWORD` | `changeme` | ArangoDB root password |
| `DB_NAME` | `critical` | Database name |
| `JWT_SECRET` | `change-me-in-production` | JWT signing secret |
| `MGMT_TOKEN` | `change-me-in-production` | Management API token |
| `ROOT_PASSWORD` | `changeme` | Default root user password |

### Images

| Image | Registry |
|-------|----------|
| `grekodocker/cr1t-api` | Docker Hub (multi-arch: amd64, arm64) |
| `grekodocker/cr1t-frontend` | Docker Hub (multi-arch: amd64, arm64) |

## Helm / Kubernetes

Helm chart at `dist/helm/critical/`.

### Quick Start

```bash
cd dist
make helm-deploy          # Deploy to critical-dev namespace
make helm-status          # Check deployment status
make helm-uninstall       # Remove deployment
make helm-template        # Render templates locally (dry run)
```

### Chart Details

| Field | Value |
|-------|-------|
| Chart name | `critical` |
| Chart version | `0.1.0` |
| Default namespace | `critical-dev` |
| Release name | `critical` |

### Components

The chart deploys:

- **API** — Deployment running `grekodocker/cr1t-api` (port 3069)
- **Frontend** — Deployment running `grekodocker/cr1t-frontend` (port 3000)
- **ArangoDB** — StatefulSet with persistent storage (optional, `arangodb.enabled: true`)
- **Ingress** — Traefik with cert-manager TLS
- **Secret** — Chart-created or externally managed (`existingSecret`)

### Key Values

```yaml
# ArangoDB — disable for external DB
arangodb:
  enabled: true             # Set false to use external ArangoDB
  persistence:
    size: 10Gi

# Secrets — chart creates a Secret by default
secrets:
  create: true
  dbPassword: "changeme"
  jwtSecret: "change-me-in-production"
  mgmtToken: "change-me-in-production"
  rootPassword: "changeme"

# Or reference an existing Secret (must contain DB_PASSWORD, JWT_SECRET, MGMT_TOKEN, ROOT_PASSWORD)
existingSecret: ""

# Non-secret config
config:
  dbName: "critical"
  dbUser: "root"
  dbConnectionString: ""    # Auto-generated when arangodb.enabled=true

# Ingress
ingress:
  enabled: true
  className: traefik
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
```

### Environment Overlays

Per-environment values files at `dist/helm/values-<env>.yaml`:

| File | Environment |
|------|-------------|
| `values-dev.yaml` | `critical-dev` — development/staging |

Usage:
```bash
helm upgrade --install critical helm/critical -f helm/values-dev.yaml -n critical-dev
```

## Cross-Compilation (CLI)

Build `cr1t` for all supported platforms:

```bash
make -f Makefile.xplatform build-all    # Build for all 9 platforms
make -f Makefile.xplatform release      # Full release with archives
```

### Supported Platforms

| OS | Architectures |
|----|---------------|
| Linux | amd64, 386, arm64, arm |
| macOS | amd64, arm64 |
| Windows | amd64, 386, arm64 |

Uses `cross` for cross-compilation. macOS targets build natively when on macOS.

### CLI Installer

```bash
curl -fsSL https://critical.dcommunity.space/install.sh | bash
```

Installer script: `dist/cli/crit-cli-installer.sh`
