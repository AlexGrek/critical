# Critical CLI (`cr1t`)

`cr1t` is a gitops-style CLI tool (similar to `kubectl`) that serves as a full alternative to the web frontend. It communicates with the Critical backend API over HTTP and stores authentication tokens locally.

## Quick Start

### Build

```bash
cargo build --bin cr1t
```

### Login Flow

1. **Start the backend** (if running locally):
   ```bash
   make run  # from project root
   ```

2. **Log in to a server** (interactive or non-interactive):
   ```bash
   # Interactive (prompts for URL, username, password)
   ./target/debug/cr1t login --url http://localhost:3742 -u root

   # Non-interactive (prompts for password only)
   ./target/debug/cr1t login --url http://localhost:3742 -u root
   ```

3. **Default root user credentials**:
   - **Username**: `root`
   - **Default password**: `changeme` (set via `ROOT_PASSWORD` env var, defaults to "changeme")

### After Login

Once authenticated, your context is saved in `~/.cr1tical/context.yaml`:

```bash
# List contexts
./target/debug/cr1t context list

# Switch context
./target/debug/cr1t context use <context-name>
```

## API Authentication

The CLI authenticates by sending credentials to `/api/login`:

```
POST /api/login
{
  "user": "root",
  "password": "changeme"
}
```

The backend returns a JWT token:

```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ1X3Jvb3QiLCJleHAiOjE3NzkwNTYxNzh9..."
}
```

This token is stored in the context YAML file and used for all subsequent authenticated requests.

## Context System

Contexts work like kubeconfigs — authenticate against multiple servers and switch between them.

**Context file location**: `~/.cr1tical/context.yaml`

```yaml
current: localhost-3742
contexts:
  - name: localhost-3742
    url: http://localhost:3742
    token: <jwt>
  - name: production
    url: https://critical.example.com
    token: <jwt>
```

Context names are derived from the server URL by stripping the scheme and replacing `/` and `:` with `-`.

## Key Implementation Files

| File | Purpose |
|------|---------|
| `src/main.rs` | Clap-based entrypoint and command routing |
| `src/context.rs` | Context file load/save (`~/.cr1tical/context.yaml`) |
| `src/api.rs` | HTTP client calls to backend API (login endpoint: `/api/login`) |
| `src/commands/login.rs` | Login command implementation |
| `src/commands/` | Other command implementations (one file per command group) |

## Testing

Run CLI integration tests:

```bash
make test-cli  # from project root
```

Tests use `assert_cmd` to run the CLI binary with temporary context file isolation.
