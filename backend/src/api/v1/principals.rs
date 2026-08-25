//! `POST /v1/principals/resolve` — batch-resolve principal IDs to identity cards.
//!
//! ## Contract
//!
//! **Request** (JSON body):
//! ```json
//! { "ids": ["u_alice", "g_engineering", "sa_github", "nonexistent"] }
//! ```
//!
//! **Query params:**
//! - `no-cache=true` — bypass cache, force DB fetch and update cache entries.
//!
//! **Response** (200, always — partial failures are inline):
//! ```json
//! {
//!   "u_alice":        { "type": "user",  "name": "Alice",       "avatar_ulid": "01jz..." },
//!   "g_engineering":  { "type": "group", "name": "Engineering" },
//!   "sa_github":      { "type": "service_account", "name": "GitHub Actions" },
//!   "nonexistent":    { "error": "not_found" }
//! }
//! ```
//!
//! Every requested ID appears as a key in the output map. If the principal
//! exists, the value is a `PrincipalInfo` object. If not found (or soft-deleted),
//! the value is `{"error": "not_found"}`.
//!
//! **Cache**: Per-principal TTL cache (10 min, max 2048 entries). The endpoint
//! serves from cache when possible and only queries the DB for cache misses.
//! `?no-cache=true` forces a full DB fetch and refreshes the cache.

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::cache;
use crate::error::AppError;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ResolveBody {
    ids: Vec<String>,
}

#[derive(Deserialize)]
pub struct ResolveQuery {
    #[serde(default, rename = "no-cache")]
    no_cache: Option<bool>,
}

pub async fn resolve_principals(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ResolveQuery>,
    Json(body): Json<ResolveBody>,
) -> Result<Json<Value>, AppError> {
    let no_cache = query.no_cache.unwrap_or(false);
    let mut requested_ids = body.ids;

    // Cap at a reasonable limit to prevent abuse
    if requested_ids.len() > 500 {
        return Err(AppError::bad_request("Maximum 500 IDs per request"));
    }

    // Deduplicate requested IDs to prevent cache poisoning
    requested_ids.sort_unstable();
    requested_ids.dedup();

    if requested_ids.is_empty() {
        return Ok(Json(json!({})));
    }

    let mut result_map = serde_json::Map::with_capacity(requested_ids.len());
    let mut cache_misses: Vec<String> = Vec::new();

    // Phase 1: Serve from cache (unless no-cache)
    if !no_cache {
        for id in &requested_ids {
            if let Some(cached) = state
                .cache
                .get(cache::PRINCIPAL_INFO_CACHE, id)
                .await
            {
                result_map.insert(id.clone(), cached);
            } else {
                cache_misses.push(id.clone());
            }
        }
    } else {
        cache_misses = requested_ids.clone();
    }

    // Phase 2: Batch-fetch cache misses from DB
    if !cache_misses.is_empty() {
        log::debug!(
            "[PRINCIPALS] resolve: {} cache hits, {} misses (no_cache={})",
            requested_ids.len() - cache_misses.len(),
            cache_misses.len(),
            no_cache,
        );

        let found = state
            .db
            .batch_resolve_principals(&cache_misses)
            .await
            .map_err(AppError::Internal)?;

        // Index found results by ID
        let mut found_map: std::collections::HashMap<String, Value> =
            std::collections::HashMap::with_capacity(found.len());
        for doc in found {
            if let Some(id) = doc.get("id").and_then(|v| v.as_str()) {
                // Build the response value (without the "id" field — it's the map key)
                let mut info = doc.clone();
                if let Some(obj) = info.as_object_mut() {
                    obj.remove("id");
                }
                found_map.insert(id.to_string(), info);
            }
        }

        // Populate result map and cache
        for id in &cache_misses {
            let value = match found_map.remove(id) {
                Some(info) => info,
                None => json!({"error": "not_found"}),
            };

            // Cache both found and not-found results (negative caching)
            state
                .cache
                .set(
                    cache::PRINCIPAL_INFO_CACHE,
                    id.clone(),
                    value.clone(),
                )
                .await;

            result_map.insert(id.clone(), value);
        }
    }

    Ok(Json(Value::Object(result_map)))
}
