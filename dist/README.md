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
task docker:build

# Start the stack
task stack:up

# Open http://localhost:8080
```

## Tasks

Run from the repo root — every target below lives in [`../Taskfile.yml`](../Taskfile.yml).

| Task                         | Description                                           |
|------------------------------|-------------------------------------------------------|
| `task docker:build`          | Build both images locally (current arch)              |
| `task docker:build:api`      | Build API image only                                  |
| `task docker:build:frontend` | Build frontend image only                             |
| `task docker:push`           | Build multi-arch (amd64+arm64) and push to Docker Hub |
| `task stack:up`              | Start the stack, wait for health checks               |
| `task stack:down`            | Stop the stack                                        |
| `task stack:logs`            | Tail logs from all services                           |
| `task stack:status`          | Show running containers                               |
| `task stack:reset`           | Stop and remove all volumes (clean slate)             |

Override the image tag with `task docker:build TAG=v1.2.3`.

## Environment Variables

Set in shell or `.env` file next to `docker-compose.yml`:

| Variable         | Default                    | Description              |
|------------------|----------------------------|--------------------------|
| `TAG`            | `latest`                   | Image tag                |
| `GATEWAY_PORT`   | `8080`                     | Exposed port             |
| `DB_PASSWORD`    | `changeme`                 | ArangoDB root password   |
| `DB_NAME`        | `critical`                 | Database name            |
| `JWT_SECRET`     | `change-me-in-production`  | JWT signing secret       |
| `ROOT_PASSWORD`  | `changeme`                 | Default root user password |

## Images

| Image | Registry |
|-------|----------|
| `grekodocker/cr1t-api` | [Docker Hub](https://hub.docker.com/r/grekodocker/cr1t-api) |
| `grekodocker/cr1t-frontend` | [Docker Hub](https://hub.docker.com/r/grekodocker/cr1t-frontend) |
