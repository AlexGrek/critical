use anyhow::Result;

use crit_shared::data_models::*;
use crit_shared::util_models::*;

use super::ArangoDb;

impl ArangoDb {
    /// Ensure all super-permissions exist and that u_root has every one of them.
    pub(super) async fn seed_permissions(&self) -> Result<()> {
        use super_permissions::*;

        let all = [
            ADM_USER_MANAGER,
            ADM_CONFIG_EDITOR,
            USR_CREATE_GROUPS,
            USR_CREATE_PROJECTS,
        ];

        for perm in all {
            self.grant_permission(perm, "u_root").await?;
        }

        Ok(())
    }

    pub async fn has_permission(&self, user_id: &str, permission: &str) -> Result<bool> {
        let query = r#"
            LET perm = DOCUMENT("permissions", @permission)
            FILTER perm != null

            LET user_principals = UNION_DISTINCT(
                [@user],
                (FOR v IN 1..10 OUTBOUND CONCAT("users/", @user) memberships
                    OPTIONS { uniqueVertices: "global", order: "bfs" }
                    FILTER v.deletion == null
                    RETURN v._key)
            )

            RETURN LENGTH(INTERSECTION(user_principals, perm.principals)) > 0
        "#;

        let vars = std::collections::HashMap::from([
            ("user", serde_json::Value::String(user_id.to_string())),
            (
                "permission",
                serde_json::Value::String(permission.to_string()),
            ),
        ]);

        let result: Vec<bool> = self.aql(query, vars).await?;

        Ok(result.first().copied().unwrap_or(false))
    }

    /// Check if any of the given (pre-resolved) principals holds the named permission.
    /// Avoids redundant graph traversal when principals are already known.
    pub async fn has_permission_with_principals(
        &self,
        principals: &[String],
        permission: &str,
    ) -> Result<bool> {
        let query = r#"
            LET perm = DOCUMENT("permissions", @permission)
            FILTER perm != null
            RETURN LENGTH(INTERSECTION(@principals, perm.principals)) > 0
        "#;

        let vars = std::collections::HashMap::from([
            (
                "permission",
                serde_json::Value::String(permission.to_string()),
            ),
            ("principals", serde_json::to_value(principals)?),
        ]);

        let result: Vec<bool> = self.aql(query, vars).await?;

        Ok(result.first().copied().unwrap_or(false))
    }

    /// Get all principal IDs for a user: the user's own ID plus all group IDs
    /// reachable through the membership graph (up to 10 levels deep).
    pub async fn get_user_principals(&self, user_id: &str) -> Result<Vec<String>> {
        // TODO: cache this with 30s TTL, say explicitly in the docs that group membership changes may take up to 30s to propagate to permissions, there is no invalidation and system is vulnerable for 30s after u remove someone from a group or delete a group until the cache expires. This is a good candidate for a Redis cache if we want to optimize it later, but for now let's keep it simple and do it in-process with TTL, as group membership changes are relatively rare and this is not on the critical path of any request (only needed for permission checks which are cached separately).
        let query = r#"
            LET user_principals = UNION_DISTINCT(
                [@user],
                (FOR v IN 1..10 OUTBOUND CONCAT("users/", @user) memberships
                    OPTIONS { uniqueVertices: "global", order: "bfs" }
                    FILTER v.deletion == null
                    RETURN v._key)
            )
            RETURN user_principals
        "#;

        let vars = std::collections::HashMap::from([(
            "user",
            serde_json::Value::String(user_id.to_string()),
        )]);

        let result: Vec<Vec<String>> = self.aql(query, vars).await?;

        Ok(result.into_iter().next().unwrap_or_default())
    }

    pub async fn grant_permission(&self, permission: &str, principal: &str) -> Result<()> {
        // TODO: add "ensure permission exists" logic to add multiple permissions without worrying about
        // TODO: add "ensure permission not exists" to mass revoke permissions
        let query = r#"
            UPSERT { _key: @permission }
            INSERT { _key: @permission, principals: [@principal] }
            UPDATE { principals: UNION_DISTINCT(OLD.principals, [@principal]) }
            IN permissions
        "#;

        let vars = std::collections::HashMap::from([
            (
                "permission",
                serde_json::Value::String(permission.to_string()),
            ),
            (
                "principal",
                serde_json::Value::String(principal.to_string()),
            ),
        ]);

        // ArangoDB UPSERT is a read-then-write and is not atomic: two concurrent
        // calls on the same permission key produce a write-write conflict (error 1200).
        // This is benign — both writers carry the same intent — so we retry with
        // a short exponential backoff instead of propagating the error to the caller.
        super::upsert_with_retry(|| {
            let vars = vars.clone();
            async move {
                self.aql::<serde_json::Value>(query, vars).await
                    .map(|_| ())
            }
        })
        .await
    }

    pub async fn revoke_permission(&self, permission: &str, principal: &str) -> Result<()> {
        let query = r#"
            LET perm = DOCUMENT("permissions", @permission)
            FILTER perm != null
            UPDATE perm WITH {
                principals: REMOVE_VALUE(perm.principals, @principal)
            } IN permissions
        "#;

        let vars = std::collections::HashMap::from([
            (
                "permission",
                serde_json::Value::String(permission.to_string()),
            ),
            (
                "principal",
                serde_json::Value::String(principal.to_string()),
            ),
        ]);

        self.aql::<serde_json::Value>(query, vars).await?;

        Ok(())
    }

    pub async fn get_permission(&self, permission: &str) -> Result<Option<GlobalPermission>> {
        match self.permissions.document::<GlobalPermission>(permission).await {
            Ok(doc) => Ok(Some(doc.document)),
            Err(arangors::ClientError::Arango(ref e)) if e.code() == 404 => Ok(None),
            Err(arangors::ClientError::Arango(ref e)) if e.code() == 1202 => Ok(None),
            Err(e) => Err(anyhow::anyhow!(e.to_string())),
        }
    }

    pub async fn get_user_permissions(&self, user_id: &str) -> Result<Vec<String>> {
        // TODO: create separate godmode endpoint to check if the user X has access to Y and with what permission bits or overrides
        let query = r#"
            LET user_principals = UNION_DISTINCT(
                [@user],
                (FOR v IN 1..10 OUTBOUND CONCAT("users/", @user) memberships
                    OPTIONS { uniqueVertices: "global", order: "bfs" }
                    FILTER v.deletion == null
                    RETURN v._key)
            )

            FOR perm IN permissions
                FILTER LENGTH(INTERSECTION(user_principals, perm.principals)) > 0
                RETURN perm._key
        "#;

        let vars = std::collections::HashMap::from([(
            "user",
            serde_json::Value::String(user_id.to_string()),
        )]);

        let result: Vec<String> = self.aql(query, vars).await?;

        Ok(result)
    }
}
