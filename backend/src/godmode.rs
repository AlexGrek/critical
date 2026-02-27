use std::time::Duration;

/// Well-known cache names.
pub const SPECIAL_ACCESS_CACHE: &str = "special_access_cache";

/// TTL for the special access cache (5 minutes).
pub const SPECIAL_ACCESS_TTL: Duration = Duration::from_secs(5 * 60);

/// Build the cache key for a user's godmode status.
pub fn godmode_cache_key(user_id: &str) -> String {
    format!("{}//adm_godmode", user_id)
}
