# CLI (`cr1t`)

`cr1t` is a gitops-style CLI (similar to `kubectl`) that serves as a full alternative to the web frontend. It communicates with the Critical backend API over HTTP.

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

**Context file**: `~/.cr1tical/context.yaml`

```yaml
contexts:
  dev:
    url: https://critical-dev.example.com
    token: <jwt>
  production:
    url: https://critical.example.com
    token: <jwt>
current: dev
```

## Key Files

| File | Purpose |
|------|---------|
| `cli/src/main.rs` | Clap-based entrypoint and command routing |
| `cli/src/context.rs` | Context file load/save |
| `cli/src/api.rs` | HTTP client calls to backend API |
| `cli/src/commands/` | Command implementations (one file per command group) |
