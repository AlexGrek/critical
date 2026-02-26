# Access Control

This document describes what actions users can perform, what permissions are required, and how ACLs affect each resource type.

## Authentication

All API operations (except `/register`, `/login`, `/health`) require a valid JWT token in the `Authorization: Bearer <token>` header. Unauthenticated requests are rejected before any permission check runs.

## Permission Model

Critical uses two layers of authorization:

1. **Super-permissions** — global, coarse-grained permissions stored in the `permissions` collection. Grant broad capabilities like "manage all users" or "create projects".
2. **Document-level ACLs** — fine-grained permission entries embedded in individual resources (groups, projects). Control who can read, modify, or manage a specific resource.

Both layers resolve through the **membership graph**: a user's effective identity is their own user ID plus all group IDs reachable through nested group memberships (up to 10 levels deep). If any of those principals appears in a permission grant or ACL entry, the user has that permission.

## Super-Permissions

Granted per-principal in the `permissions` collection. `u_root` has all of them by default.

| Permission            | Granted on Registration | Description                                  |
| --------------------- | ----------------------- | -------------------------------------------- |
| `adm_user_manager`    | No                      | Full control over all users, groups, memberships |
| `adm_config_editor`   | No                      | Edit global configuration and any project    |
| `usr_create_groups`   | Yes                     | Allows creating new groups                   |
| `usr_create_projects` | No (`u_root` only)      | Allows creating new projects                 |

## ACL Permission Bits

Groups and projects embed an `acl` field (`AccessControlStore`) containing a list of entries, each with a set of permission bits and a list of principals they apply to.

| Permission | Bit | Description                                 |
| ---------- | --- | ------------------------------------------- |
| FETCH      | 1   | Read a single document                      |
| LIST       | 2   | See the document in listings                |
| NOTIFY     | 4   | Receive notifications about the document    |
| CREATE     | 8   | Create child resources within this document |
| MODIFY     | 16  | Update or delete the document               |
| CUSTOM1    | 32  | Reserved for future use                     |
| CUSTOM2    | 64  | Reserved for future use                     |

Common composites:

| Name  | Value | Includes               |
| ----- | ----- | ---------------------- |
| READ  | 7     | FETCH + LIST + NOTIFY  |
| WRITE | 31    | READ + CREATE + MODIFY |
| ROOT  | 127   | All permissions        |

## Resources

### Users

Users represent people. Each user has an ID prefixed with `u_` (e.g. `u_alice`).

| Operation          | Who Can Do It           |
| ------------------ | ----------------------- |
| List all users     | Any authenticated user  |
| Read a single user | Any authenticated user  |
| Create a user      | `adm_user_manager` only |
| Update a user      | `adm_user_manager` only |
| Delete a user      | `adm_user_manager` only |

**Self-registration** via `/register` is open — no permission needed. The gitops `POST /v1/global/users` endpoint (direct user creation) requires `adm_user_manager`.

**Cascade on deletion**: when a user is deleted, all their membership edges are removed. Any group that becomes empty as a result is automatically deleted (recursively up the group hierarchy).

### Groups

Groups organize users and other groups. Each group has an ID prefixed with `g_` (e.g. `g_engineers`). Groups embed an ACL that controls per-group access.

| Operation           | Who Can Do It                                                            |
| ------------------- | ------------------------------------------------------------------------ |
| List groups         | `adm_user_manager`, OR user sees only groups where their ACL grants READ |
| Read a single group | `adm_user_manager`, OR ACL grants READ                                   |
| Create a group      | `adm_user_manager`, OR `usr_create_groups` super-permission              |
| Update a group      | `adm_user_manager`, OR ACL grants MODIFY                                 |
| Delete a group      | `adm_user_manager`, OR ACL grants MODIFY                                 |

**On creation**:
- The creator is automatically added to the group's ACL with ROOT (all permissions).
- The creator is automatically added as a member of the group (membership edge inserted).

**Empty-group auto-deletion**: if an update or membership removal leaves a group with zero members, the group is automatically deleted.

**Cascade on deletion**: when a group is deleted, all membership edges pointing into it (its members) are removed, and it is removed from all parent groups. If any parent group becomes empty, it is recursively deleted.

### Memberships

Memberships are edges connecting any principal (user, group, service account, or pipeline account) to a group. They define the group hierarchy and are stored in the `memberships` edge collection.

Membership access is controlled by the **target group's ACL**, not a separate permission:

| Operation                | Who Can Do It                                                                  |
| ------------------------ | ------------------------------------------------------------------------------ |
| List memberships         | `adm_user_manager`, OR user's principals have READ on the target group's ACL   |
| Read a single membership | `adm_user_manager`, OR user's principals have READ on the target group's ACL   |
| Create a membership      | `adm_user_manager`, OR user's principals have MODIFY on the target group's ACL |
| Delete a membership      | `adm_user_manager`, OR user's principals have MODIFY on the target group's ACL |

**On deletion**: if removing a membership leaves the target group with zero members, the group is automatically deleted (with recursive cascade).

### Projects

Projects are global resources that act as namespaces. They have a per-document ACL.

| Operation          | Who Can Do It                                             |
| ------------------ | --------------------------------------------------------- |
| List all projects  | `adm_config_editor`, OR ACL grants READ                   |
| Read a project     | `adm_config_editor`, OR ACL grants READ                   |
| Create a project   | `adm_config_editor`, OR `usr_create_projects`             |
| Update a project   | `adm_config_editor`, OR ACL grants MODIFY                 |
| Delete a project   | `adm_config_editor`, OR ACL grants MODIFY                 |

**On creation**: the creator is automatically added to the project ACL with ROOT permissions.

### Scoped Resources (Project-Namespaced)

Resources belonging to a project (e.g. tasks, pipelines) are accessed via `/v1/projects/{project}/{kind}`. These use **Hybrid ACL** resolution:

1. If the resource's own `acl.list` is non-empty → use the resource's ACL directly
2. Otherwise → fall back to the parent project's ACL, filtering by `scope` matching the resource kind

| Operation                 | Who Can Do It                                                                             |
| ------------------------- | ----------------------------------------------------------------------------------------- |
| List scoped resources     | Super-perm bypass, OR project/resource ACL grants READ for the resource kind              |
| Read a scoped resource    | Super-perm bypass, OR project/resource ACL grants READ for the resource kind              |
| Create a scoped resource  | Super-perm bypass, OR project ACL has CREATE for the resource kind                        |
| Update a scoped resource  | Super-perm bypass, OR project/resource ACL grants MODIFY for the resource kind            |
| Delete a scoped resource  | Super-perm bypass, OR project/resource ACL grants MODIFY for the resource kind            |

**ACL scope entries on projects:**

```json
{ "permissions": 31, "principals": ["g_devs"], "scope": "tasks" }
```

- `scope` absent or `"*"` → applies to all resource kinds in the project
- `scope: "tasks"` → applies only to the `tasks` collection
- This enables per-service-kind access control without separate permission documents

### Other Kinds (CRD-style)

The gitops API accepts any kind string as a collection name. Collections beyond the built-in ones are auto-created on first access. These have no permission restrictions:

| Operation     | Who Can Do It          |
| ------------- | ---------------------- |
| Any operation | Any authenticated user |

## Denial Behavior

All authorization denials return **404 Not Found** — never 401 or 403. This prevents information leakage about whether a resource exists.

## Practical Examples

### Regular user creates and manages a group

1. User registers via `/register` and receives `usr_create_groups` automatically.
2. User creates a group via `POST /v1/global/groups`. The system:
   - Injects the user into the group's ACL with ROOT permissions.
   - Adds the user as a member of the group (membership edge).
3. User can now read, update, and delete this group (ROOT includes MODIFY).
4. User can add/remove members (ROOT includes MODIFY, which gates membership operations).
5. Other users cannot see or interact with the group unless the creator adds them to the ACL.

### Admin manages all groups

A user with `adm_user_manager` bypasses all group and membership ACL checks. They can read, create, update, delete any group and manage any group's memberships.

### ACL grants read-only access

A group is created with an ACL entry: `{"permissions": 7, "principals": ["u_bob"]}`. Bob (permissions = 7 = READ) can:
- See the group in listings
- Read the group's details

Bob cannot:
- Update or delete the group (requires MODIFY = 16)
- Add or remove members (requires MODIFY)

### Nested group permissions

If `u_alice` is a member of `g_team`, and `g_team` appears in a group's ACL with MODIFY, then Alice effectively has MODIFY on that group. This resolution happens automatically via the membership graph (up to 10 levels of nesting).
