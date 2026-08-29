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

### Tasks (run from the repo root)

| Task | Description |
|------|-------------|
| `task docker:build` | Build both images locally (current arch) |
| `task docker:build:api` | Build API image only |
| `task docker:build:frontend` | Build frontend image only |
| `task docker:push` | Build multi-arch (amd64+arm64) and push to Docker Hub |
| `task stack:up` | Start the stack, wait for health checks |
| `task stack:down` | Stop the stack |
| `task stack:logs` | Tail logs from all services |
| `task stack:status` | Show running containers |
| `task stack:reset` | Stop and remove all volumes (clean slate) |

Override the image tag with `task docker:build TAG=v1.2.3`.

### Environment Variables

Set in shell or `.env` file next to `docker-compose.yml`:

| Variable | Default | Description |
|----------|---------|-------------|
| `TAG` | `latest` | Image tag |
| `GATEWAY_PORT` | `8080` | Exposed port |
| `DB_PASSWORD` | `changeme` | ArangoDB root password |
| `DB_NAME` | `critical` | Database name |
| `JWT_SECRET` | `change-me-in-production` | JWT signing secret |
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
task helm:deploy          # Deploy to critical-dev namespace
task helm:status          # Check deployment status
task helm:uninstall       # Remove deployment
task helm:template        # Render templates locally (dry run)
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
  rootPassword: "changeme"

# Or reference an existing Secret (must contain DB_PASSWORD, JWT_SECRET, ROOT_PASSWORD)
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
task cli:build:all        # Build for all 9 platforms
task cli:release          # Full release with archives
task cli:verify           # Show which platform binaries are present
task cli:info             # Show version, commit and the target list
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
