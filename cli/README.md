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

## Commands

### Groups

List all groups:

```bash
cr1t groups list
```

Show a specific group (outputs as YAML):

```bash
cr1t groups describe <group-id>
```

Example:

```bash
# List groups
$ cr1t groups list
Groups:

  Engineering (g_engineering)
  Design (g_design)
  Marketing (g_marketing)

# Describe a group
$ cr1t groups describe g_engineering
id: g_engineering
name: Engineering
acl:
  owner:
    - u_alice
  member:
    - u_bob
    - u_charlie
```

### Users

List all users:

```bash
cr1t users list
```

Show a specific user (outputs as YAML):

```bash
cr1t users describe <user-id>
```

Example:

```bash
# List users
$ cr1t users list
Users:

  Alice Smith (u_alice)
  Bob Johnson (u_bob)
  Charlie Brown (u_charlie)

# Describe a user
$ cr1t users describe u_alice
id: u_alice
personal:
  name: Alice Smith
  job_title: Engineering Lead
  gender: ""
  manager: ""
deactivated: false
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

| File                      | Purpose                                                         |
| ------------------------- | --------------------------------------------------------------- |
| `src/main.rs`             | Clap-based entrypoint and command routing                       |
| `src/context.rs`          | Context file load/save (`~/.cr1tical/context.yaml`)             |
| `src/api.rs`              | HTTP client calls to backend API (login, groups, users)         |
| `src/commands/login.rs`   | Login command implementation                                    |
| `src/commands/gitops.rs`  | Groups and Users list/describe commands                        |
| `src/commands/`           | Other command implementations (one file per command group)      |

## Testing

Run CLI integration tests:

```bash
make test-cli  # from project root
```

This automatically:
- Starts an ephemeral ArangoDB container
- Starts the backend server
- Runs all CLI integration tests with isolated context files
- Cleans up containers after completion

Tests use `assert_cmd` to run the CLI binary with temporary context file isolation.

### Test Coverage

Current tests include:

- **Context management**: List, switch, error cases
- **Groups**: List (empty/with data), describe, 404 handling
- **Users**: List (with users), describe, 404 handling

To run tests manually:

```bash
# Start backend and database separately
make run-fresh  # Terminal 1

# Run tests in another terminal
cargo test -p crit-cli --test cli_test -- --include-ignored --test-threads=1
```

## Architecture Notes

The CLI uses a **gitops-style API** (`/api/v1/global/{kind}`) where:

- `kind` is a resource type (e.g., `groups`, `users`)
- List and describe commands fetch from `/api/v1/global/{kind}` and `/api/v1/global/{kind}/{id}`
- All requests include a Bearer token in the Authorization header
- Responses are authenticated and ACL-filtered by the backend

This design allows the CLI to be extensible — new resource kinds can be added without CLI changes.
