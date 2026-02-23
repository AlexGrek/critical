# App overview

This is a document for a Critical project: developer-friendly project management app with web and cli interfaces.
Goal: replace jira and confluence, use same app too control pipelines, deployents and product lifecycle, as well as bug tracking, sprint
and canban boards, project wiki.

This app implements gitops-like approach to project management: PaC - Project as Code.

Uses http(s) with JWT auth

Uses graph database to store hierarchical data: ArangoDB

Uses react-router frontend with SSR/SPA

### Critical = GitOps for product lifecycle

Every entity in the company is a resource.

Every resource is declarative.

Every change is auditable.

Every action can be automated.

Critical is not a tracker — it is an operating system for development teams.

# Cr1t

`cr1t` is a gitops kubectl-like CLI tool for comfortable Critical interaction from CLI and automation. Written in Rust, uses same
models as the backend (also written in rust).
Allows full control, same as UI.

Uses public API, as well as frontend, making Critical even more hackable (in a good way) and developer-friendly.

---

Description of the product in `cr1t` CLI client perspective.

# Storage

Instead of foreign keys, Critical uses relations.

Examples:

```
task -> belongs_to -> sprint
task -> implements -> feature
bug -> caused_by -> release
deployment -> deploys -> artifact
artifact -> built_from -> pipeline_run
pipeline -> triggered_by -> repo
user -> member_of -> group
page -> references -> task
```

All relations are first-class and queryable.

# Kind (of resource)

Kind is like the "kind" in kubernetes: a resource class. It has single and plural forms: "user"/"users", "group"/"groups".

Resources are stored in ArangoDB as a graph:

vertices → objects

edges → relations

history → immutable revisions

# Project

Projects act like namespaces. Namespaced resources should always specify projects.

# CLI (cr1t) commands

Cr1t should look like kubectl as much as we can.

Projects: -p or --project, save last used project in context and assume this is about it. Also, all projects should be available with flag -A.

## Get resources (brief, not full data is fetched from db)

```sh
cr1t get users

cr1t get groups

cr1t get projects --limit 4  # get 4 projects per page
```

Flags: 
    -o - output format [json,yaml,table,list]

Requested features:

### Filtering & Selectors

Like Kubernetes label selectors:

```sh
cr1t get tasks -l priority=high
cr1t get bugs --field-selector state=Open
cr1t get deployments -l env=prod
```

More output modes:

```
-o table
-o json
-o yaml
-o wide
-o graph
-o name
```

## Describe: get full info about some object

crit describe [users/groups/...etc]

Should return full info, including special props:

1. kind
2. state (if resource is stateful)
3. children (if resource has children)'

Flags: 
    -o - output format [json,yaml]

## Apply (GitOps Mode)

Create/update resources from files:

```sh
cr1t apply -f task.yaml
cr1t apply -k ./sprint/
```

Supports:

1. directory trees
2. kustomize-like overlays
3. stdin

## Diff

`cr1t diff -f task.yaml`

Shows intent vs actual state.

## Delete

```sh
cr1t delete task login-page
cr1t delete -f old_sprint/
```

Soft delete → archived revision preserved. Cleanup configurable and deferred, implemented as async backend job.

## Edit

Opens **$EDITOR** and patches resource.

```sh
cr1t edit task login-page
```

## Patch

```sh
cr1t patch task login-page --type merge -p '{"spec":{"priority":"high"}}'
```

Patch types:

- merge
- json
- strategic

## Logs (for stateful automation)

```sh
cr1t logs run build-frontend-123
cr1t logs deployment api-prod
```

## Watch

Live updates via websocket:

```sh
cr1t get tasks -w
cr1t watch deployments
```

## PolicyШы ершы

Example:

```yaml
kind: Policy
spec:
  match:
    kind: deployment
    env: prod
  require:
    approval: release-managers
```

## Offline Mode (Least Important Feature)

You can clone project state locally:

```sh
cr1t snapshot export myproject
cr1t snapshot import myproject
```

Allows:

- airgapped work
- code review of project state
- pull-request-like workflows

# Overall philosophy

Frontend Philosophy

UI is not special.
UI = API client.

Everything UI can do:

CLI can do
API can do
automation can do

No hidden operations.

## Short Tagline

**Critical is Kubernetes for software development lifecycle.**

