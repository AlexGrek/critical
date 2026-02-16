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
