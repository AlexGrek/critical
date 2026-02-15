# Critical — Deployment

Prod-like Docker Compose stack for the Critical platform.

## Architecture

```
            :8080
              |
          [ gateway ]  (nginx:alpine)
           /        \
     /api/*          /*
        |              |
    [ api ]      [ frontend ]
   (cr1t-api)   (cr1t-frontend)
        |
   [ arangodb ]
```

All traffic enters through the **gateway** on port 8080.
`/api/*` routes to the Rust backend, everything else to the React SSR frontend.

## Prerequisites

- Docker with BuildKit enabled (Docker Desktop 4.x+ or `DOCKER_BUILDKIT=1`)
- `docker buildx` for multi-arch builds (included in Docker Desktop)

## Quick Start

```bash
# Build images locally (current arch)
make build

# Start the stack
make up

# Open http://localhost:8080
```

## Make Targets

| Target           | Description                                       |
|------------------|---------------------------------------------------|
| `make build`     | Build both images locally (current arch)          |
| `make build-api` | Build API image only                              |
| `make build-frontend` | Build frontend image only                    |
| `make build-push`| Build multi-arch (amd64+arm64) and push to Docker Hub |
| `make up`        | Start the stack, wait for health checks           |
| `make down`      | Stop the stack                                    |
| `make logs`      | Tail logs from all services                       |
| `make status`    | Show running containers                           |
| `make reset`     | Stop and remove all volumes (clean slate)         |

## Environment Variables

Set in shell or `.env` file next to `docker-compose.yml`:

| Variable         | Default                    | Description              |
|------------------|----------------------------|--------------------------|
| `TAG`            | `latest`                   | Image tag                |
| `GATEWAY_PORT`   | `8080`                     | Exposed port             |
| `DB_PASSWORD`    | `changeme`                 | ArangoDB root password   |
| `DB_NAME`        | `critical`                 | Database name            |
| `JWT_SECRET`     | `change-me-in-production`  | JWT signing secret       |
| `MGMT_TOKEN`     | `change-me-in-production`  | Management API token     |
| `ROOT_PASSWORD`  | `changeme`                 | Default root user password |

## Images

| Image | Registry |
|-------|----------|
| `grekodocker/cr1t-api` | [Docker Hub](https://hub.docker.com/r/grekodocker/cr1t-api) |
| `grekodocker/cr1t-frontend` | [Docker Hub](https://hub.docker.com/r/grekodocker/cr1t-frontend) |
