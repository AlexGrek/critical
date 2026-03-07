# CLI (`cr1t`)

`cr1t` is a gitops-style CLI (similar to `kubectl`) that serves as a full alternative to the web frontend. It communicates with the Critical backend API over HTTP.

**Development guide**: See [`cli/README.md`](../cli/README.md) for local development, building, testing, and login flow details.

## Installation

```bash
curl -fsSL https://critical.dcommunity.space/install.sh | bash
```

Or build from source:

```bash
cargo build --bin cr1t
```

## Commands

### `cr1t login`

Authenticate against a Critical server. The JWT is stored in `~/.cr1tical/context.yaml`.

```bash
# Interactive (prompts for URL, username, password)
cr1t login

# Non-interactive
cr1t login --url https://critical.example.com --user alice
```

Registration is **not** supported from the CLI. Use the web frontend or API directly.

### `cr1t context list`

Show all saved contexts.

```bash
cr1t context list
```

### `cr1t context use <name>`

Switch the active context.

```bash
cr1t context use production
```

### `cr1t groups list`

List all groups (default: table format).

```bash
cr1t groups list                      # Table format (default)
cr1t groups list -o json              # JSON format
cr1t groups list -o yaml              # YAML format
```

### `cr1t groups describe <id>`

Show a specific group in full YAML.

```bash
cr1t groups describe g_engineering
```

### `cr1t users list`

List all users (default: table format).

```bash
cr1t users list                       # Table format (default)
cr1t users list -o json               # JSON format
cr1t users list -o yaml               # YAML format
```

### `cr1t users describe <id>`

Show a specific user in full YAML.

```bash
cr1t users describe u_alice
```

### `cr1t get <kind> [id]`

Generic command to list or get a resource by kind and optional ID.

```bash
cr1t get groups                       # List all groups
cr1t get groups g_engineering         # Get a specific group
cr1t get users                        # List all users
cr1t get users u_alice                # Get a specific user
cr1t get projects                     # List all projects
```

Supports all output formats:

```bash
cr1t get groups -o json
cr1t get groups -o yaml
cr1t get groups -o table
```

### `cr1t describe <kind> <id>`

Show a resource in full YAML with `kind` field prepended (useful for `cr1t apply`).

```bash
cr1t describe groups g_engineering
cr1t describe users u_alice
```

### `cr1t apply [-f <file>]`

Create or update resources from YAML. Reads from stdin if no file specified.

```bash
# From a file
cr1t apply -f group.yaml

# From stdin
cat group.yaml | cr1t apply
echo "kind: group\nid: g_ops\nname: Ops" | cr1t apply
```

YAML must have `kind` and `id` fields. Multi-document files (separated by `---`) are supported.

```yaml
kind: group
id: g_engineering
name: Engineering
---
kind: group
id: g_design
name: Design
```

### `cr1t debug events`

Show recent events from the event log (requires godmode permissions).

```bash
cr1t debug events                     # Table format (default)
cr1t debug events -o json             # JSON output
cr1t debug events -o yaml             # YAML format
```

**Table format** displays a structured view of events with the following columns:
- **ID**: Event unique identifier (UUIDv4)
- **MOMENT**: Timestamp (UTC)
- **PRIORITY**: Event priority level (Lifecycle, Important, Expected, Note, Minor)
- **KIND**: Event category (Server, Error, Background, EntityManagement, Stats, Database, ObjectStorage)
- **PRINCIPAL**: User/principal that triggered the event (or `-` if system-triggered)

Additional details (event metadata and affected resources) are shown below each event.

**Note**: This command requires `ADM_GODMODE` super-permission. Non-godmode users will receive a 403 Forbidden error.

## Context System

Contexts work like kubeconfigs — authenticate against multiple servers and switch between them.

**Contexts are created automatically by `cr1t login`.** The context name is derived from the server URL by stripping the scheme and replacing `/` and `:` with `-`. For example:

| Server URL | Context name |
|------------|--------------|
| `https://critical-dev.example.com` | `critical-dev.example.com` |
| `http://localhost:3742` | `localhost-3742` |

Logging in to the same URL again updates the existing context (upsert). Each login also sets the new context as current.

**Context file**: `~/.cr1tical/context.yaml`

```yaml
current: critical-dev.example.com
contexts:
  - name: critical-dev.example.com
    url: https://critical-dev.example.com
    token: <jwt>
  - name: critical.example.com
    url: https://critical.example.com
    token: <jwt>
```

## Key Files

| File | Purpose |
|------|---------|
| `cli/src/main.rs` | Clap-based entrypoint and command routing |
| `cli/src/context.rs` | Context file load/save |
| `cli/src/api.rs` | HTTP client calls to backend API |
| `cli/src/commands/` | Command implementations (one file per command group) |
