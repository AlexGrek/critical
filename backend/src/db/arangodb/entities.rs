use anyhow::{Result, anyhow};
use arangors::document::Document;
use serde_json::json;

use crit_shared::data_models::*;

use super::{ArangoDb, ArangoTx, collection_for_principal};

impl ArangoDb {
    pub async fn create_user(&self, user: User, tx: Option<&mut ArangoTx>) -> Result<()> {
        let doc = Document::new(user);

        if let Some(tr) = tx {
            let col = tr
                .inner
                .collection("users")
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
            col.create_document(doc, Default::default())
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
        } else {
            self.users
                .create_document(doc, Default::default())
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
        }

        Ok(())
    }

    pub async fn create_group(&self, group: Group, tx: Option<&mut ArangoTx>) -> Result<()> {
        let doc = Document::new(group);

        if let Some(tr) = tx {
            let col = tr
                .inner
                .collection("groups")
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
            col.create_document(doc, Default::default())
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
        } else {
            self.groups
                .create_document(doc, Default::default())
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
        }

        Ok(())
    }

    pub async fn add_principal_to_group(
        &self,
        principal_id: &str,
        group_id: &str,
        tx: Option<&mut ArangoTx>,
    ) -> Result<()> {
        let key = format!("{}::{}", principal_id, group_id);
        let from_collection = collection_for_principal(principal_id);
        let from = format!("{}/{}", from_collection, principal_id);
        let to = format!("groups/{}", group_id);
        let body = json!({
            "_key": key,
            "_from": from,
            "_to": to,
            "principal": principal_id,
            "group": group_id,
        });

        if let Some(tr) = tx {
            let col = tr
                .inner
                .collection("memberships")
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
            col.create_document(Document::new(body), Default::default())
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
        } else {
            self.memberships
                .create_document(Document::new(body), Default::default())
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
        }

        Ok(())
    }

    pub async fn get_users_list(&self) -> Result<Vec<User>> {
        let query = "FOR u IN users RETURN u";
        let users: Vec<User> = self.aql_str_query(query).await?;
        Ok(users)
    }

    pub async fn get_groups_list(&self) -> Result<Vec<Group>> {
        let query = "FOR g IN groups RETURN g";
        let groups: Vec<Group> = self.aql_str_query(query).await?;
        Ok(groups)
    }

    pub async fn get_users_in_group(&self, group_id: &str) -> Result<Vec<String>> {
        let query = r#"
            FOR m IN memberships
                FILTER m.group == @group
                FILTER LIKE(m.principal, "u_%")
                RETURN m.principal
        "#;

        let vars = std::collections::HashMap::from([(
            "group",
            serde_json::Value::String(group_id.to_string()),
        )]);

        let res: Vec<String> = self.aql(query, vars).await?;
        Ok(res)
    }

    pub async fn get_groups_in_group(&self, group_id: &str) -> Result<Vec<String>> {
        let query = r#"
            FOR m IN memberships
                FILTER m.group == @group
                FILTER LIKE(m.principal, "g:%")
                RETURN m.principal
        "#;

        let vars = std::collections::HashMap::from([(
            "group",
            serde_json::Value::String(group_id.to_string()),
        )]);

        let res: Vec<String> = self.aql(query, vars).await?;
        Ok(res)
    }

    /// Remove a principal from all groups it belongs to.
    /// Returns the list of group IDs that became empty after removal.
    pub async fn remove_principal_from_all_groups(&self, principal_id: &str) -> Result<Vec<String>> {
        // Step 1: Find all groups this principal belongs to (active memberships only)
        let find_query = r#"
            FOR m IN memberships
                FILTER m.principal == @principal
                FILTER m.deletion == null
                RETURN m.group
        "#;
        let vars = std::collections::HashMap::from([(
            "principal",
            serde_json::Value::String(principal_id.to_string()),
        )]);
        let affected_groups: Vec<String> = self.aql(find_query, vars).await?;

        // Step 2: Remove all membership edges for this principal
        let remove_query = r#"
            FOR m IN memberships
                FILTER m.principal == @principal
                REMOVE m IN memberships
        "#;
        let vars = std::collections::HashMap::from([(
            "principal",
            serde_json::Value::String(principal_id.to_string()),
        )]);
        self.aql::<serde_json::Value>(remove_query, vars).await?;

        // Step 3: Check which of the affected groups are now empty
        let mut empty_groups = Vec::new();
        for group_id in &affected_groups {
            let count = self.count_group_members(group_id).await?;
            if count == 0 {
                empty_groups.push(group_id.clone());
            }
        }

        Ok(empty_groups)
    }

    /// Count the number of members in a group.
    pub async fn count_group_members(&self, group_id: &str) -> Result<u64> {
        let query = r#"
            RETURN LENGTH(
                FOR m IN memberships
                    FILTER m.group == @group
                    FILTER m.deletion == null
                    RETURN 1
            )
        "#;
        let vars = std::collections::HashMap::from([(
            "group",
            serde_json::Value::String(group_id.to_string()),
        )]);
        let result: Vec<u64> = self.aql(query, vars).await?;
        Ok(result.into_iter().next().unwrap_or(0))
    }

    /// Add a principal to a group document's ACL with the given permissions.
    /// If the principal already appears in any ACL entry, this is a no-op.
    /// Uses an AQL UPDATE to atomically append to the ACL list.
    pub async fn add_principal_to_group_acl(
        &self,
        group_id: &str,
        principal_id: &str,
        permissions_bits: u8,
    ) -> Result<()> {
        let query = r#"
            LET doc = DOCUMENT("groups", @group)
            FILTER doc != null
            FILTER doc.deletion == null
            LET already = (
                FOR entry IN (doc.acl.list || [])
                    FILTER @principal IN entry.principals
                    RETURN 1
            )
            FILTER LENGTH(already) == 0
            UPDATE doc WITH {
                acl: {
                    list: APPEND(doc.acl.list || [], [{
                        permissions: @permissions,
                        principals: [@principal]
                    }]),
                    last_mod_date: DATE_ISO8601(DATE_NOW())
                }
            } IN groups
        "#;

        let vars = std::collections::HashMap::from([
            (
                "group",
                serde_json::Value::String(group_id.to_string()),
            ),
            (
                "principal",
                serde_json::Value::String(principal_id.to_string()),
            ),
            (
                "permissions",
                serde_json::Value::Number(serde_json::Number::from(permissions_bits)),
            ),
        ]);

        self.aql::<serde_json::Value>(query, vars).await?;
        Ok(())
    }

    /// Get all principals that are members of a group, including transitive members
    /// (members of sub-groups, up to 10 levels deep).
    /// Returns a flat set of all user and group IDs that are direct or indirect members.
    pub async fn get_all_group_members_transitive(&self, group_id: &str) -> Result<Vec<String>> {
        let query = r#"
            LET members = UNION_DISTINCT(
                (FOR m IN memberships
                    FILTER m.group == @group
                    FILTER m.deletion == null
                    RETURN m.principal),
                (FOR v IN 1..10 INBOUND CONCAT("groups/", @group) memberships
                    OPTIONS { uniqueVertices: "global", order: "bfs" }
                    FILTER v.deletion == null
                    RETURN v._key)
            )
            RETURN members
        "#;

        let vars = std::collections::HashMap::from([(
            "group",
            serde_json::Value::String(group_id.to_string()),
        )]);

        let result: Vec<Vec<String>> = self.aql(query, vars).await?;
        Ok(result.into_iter().next().unwrap_or_default())
    }

    /// Remove all membership edges where this group is the target (members OF this group).
    pub async fn remove_all_members_of_group(&self, group_id: &str) -> Result<()> {
        let query = r#"
            FOR m IN memberships
                FILTER m.group == @group
                REMOVE m IN memberships
        "#;
        let vars = std::collections::HashMap::from([(
            "group",
            serde_json::Value::String(group_id.to_string()),
        )]);
        self.aql::<serde_json::Value>(query, vars).await?;
        Ok(())
    }

    pub async fn modify_user(&self, user: User, tx: Option<&mut ArangoTx>) -> Result<()> {
        let key = user.id.clone();
        let doc = Document::new(user);
        if let Some(tr) = tx {
            let col = tr.inner.collection("users").await?;
            col.replace_document(&key, doc, Default::default(), None)
                .await?;
        } else {
            self.users
                .replace_document(&key, doc, Default::default(), None)
                .await?;
        }
        Ok(())
    }

    pub async fn get_user_by_id(&self, user_id: &str) -> Result<Option<User>> {
        let id = if user_id.starts_with("u_") {
            user_id.to_string()
        } else {
            format!("u_{}", user_id)
        };
        match self.users.document::<User>(&id).await {
            Ok(doc) => Ok(Some(doc.document)),
            Err(arangors::ClientError::Arango(ref e)) if e.code() == 404 => Ok(None),
            Err(e) => Err(anyhow!(e.to_string())),
        }
    }

    pub async fn get_group_by_id(&self, group_id: &str) -> Result<Option<Group>> {
        match self.groups.document::<Group>(group_id).await {
            Ok(doc) => Ok(Some(doc.document)),
            Err(arangors::ClientError::Arango(it)) => {
                let error = it;
                let message = error.message().to_string();
                if error.code() == 1202 {
                    return Ok(None);
                }
                Err(anyhow!(message))
            }
            Err(e) => Err(anyhow!(e.to_string())),
        }
    }
}
