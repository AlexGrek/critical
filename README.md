# crit-cli

A full-stack project management and ticketing system with a Rust backend, React TypeScript frontend, and ArangoDB database.

Full documentation: [`docs/`](docs/README.md)

Whitepaper for architectural constraints: [`WHITEPAPER`](WHITEPAPER.md)

## 🚀 Quick Start

### Installation

Install crit-cli with a single command:

```bash
curl -fsSL https://critical.dcommunity.space/install.sh | bash
```

Or using wget:

```bash
wget -qO- https://critical.dcommunity.space/install.sh | bash
```

### Usage

```bash
cr1t [command] [options]
```

## CLI (`cr1t`)

`cr1t` is a gitops-style CLI (similar to `kubectl`) that serves as a full alternative to the web frontend. It communicates with the Critical backend API over HTTP.

### Authentication

Login with username and password. The JWT is stored in `~/.cr1tical/context.yaml` and reused for subsequent commands.

```bash
# Interactive login (prompts for URL, username, password)
cr1t login

# Non-interactive
cr1t login --url https://critical.example.com --user alice
```

Registration is not supported from the CLI. Use the web frontend or API directly.

> **Note:** Additional login methods (API keys, SSO, etc.) are planned for future releases.

### Context Management

Contexts work like kubeconfigs — you can authenticate against multiple servers and switch between them.

```bash
cr1t context list           # Show all contexts
cr1t context use <name>     # Switch active context
```

Context file location: `~/.cr1tical/context.yaml`

### Building the CLI

```bash
cargo build --bin cr1t      # Development build
```

## Testing

All test targets start an ephemeral ArangoDB container and clean up on exit.

```bash
task test                   # Run ALL tests (Rust + CLI + Python API + E2E)
task test:unit              # Rust unit & backend integration tests only
task test:cli               # CLI integration tests (starts backend automatically)
task test:api               # Python API integration tests (starts backend automatically)
task test:e2e               # Playwright E2E tests (starts backend automatically)
```

| Type | Location | What it tests |
|------|----------|---------------|
| Rust unit + backend | `backend/src/test/`, CLI `#[cfg(test)]` | Backend API via axum-test, context management |
| CLI integration | `cli/tests/cli_test.rs` | `cr1t` binary end-to-end against real backend |
| Python API | `backend/itests/` | HTTP API + WebSocket via pytest |

## 🛠️ Development

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- [Task](https://taskfile.dev) (`brew install go-task` — runs everything in `Taskfile.yml`)
- [Docker](https://www.docker.com/) (for the dev database and cross-compilation)
- [Git](https://git-scm.com/) (for version information)

### Project Structure

```
├── shared/            # Shared Rust library (crit-shared) — domain models
├── backend/           # Axum API server (axum-api)
├── cli/               # CLI tool (cr1t)
├── frontend/          # React Router 7 + Vite frontend (SSR)
├── dist/              # Docker deployment (compose, nginx, helm chart)
│   └── cli/           # CLI installer script
├── build/             # Cross-compiled CLI binaries (gitignored)
├── Taskfile.yml       # Dev/test/build/deploy orchestration
└── docker-compose.yml # Dev-only ArangoDB
```

### Development Build

```bash
task                        # List every available task
task build                  # Quick development build (all crates)
task dev                    # ArangoDB + backend (cargo watch) + frontend dev server
task run                    # Start ArangoDB + run backend only
```

## Docker Deployment

The `dist/` directory contains a prod-like Docker Compose stack.

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

```bash
task docker:build           # Build images locally
task stack:up               # Start full stack at http://localhost:8080
task stack:down             # Stop
task docker:push            # Build multi-arch (amd64+arm64) + push to Docker Hub
```

Images: `grekodocker/cr1t-api`, `grekodocker/cr1t-frontend`

See [`dist/README.md`](dist/README.md) for environment variables and configuration.

## Cross-Compilation (CLI)

```bash
task cli:build:all          # Build cr1t for all 9 platforms
task cli:release            # Full release with archives
```

CLI installer: `dist/cli/crit-cli-installer.sh`