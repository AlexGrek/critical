use crate::validation::*;

/// Validate a username (without the u_ prefix).
/// Returns the lowercased, validated username.
pub fn validate_username(username: &str) -> Result<String, String> {
    let lowercased = force_lowercase()(username);
    let validators: Vec<ValidatorFn> = vec![
            limit_length(63),
            limit_min_length(2),
            allow_only_alphanumerics_and_specials(Some("_")),
            not_start_with_digit(),
        ];
    run_validators(&lowercased, &validators)?;
    Ok(lowercased)
}

/// Validate a group ID (without the g_ prefix).
/// Strips the g_ prefix if present, validates, and returns the validated ID without prefix.
/// The caller is responsible for adding the g_ prefix to the final ID.
pub fn validate_group_id(group_id: &str) -> Result<String, String> {
    // Strip g_ prefix if present for backward compatibility
    let id_without_prefix = group_id.strip_prefix("g_").unwrap_or(group_id);

    let lowercased = force_lowercase()(id_without_prefix);
    let validators: Vec<ValidatorFn> = vec![
            limit_length(63),
            limit_min_length(2),
            allow_only_alphanumerics_and_specials(Some("_-")),
            not_start_with_digit(),
            not_start_with_char('-'),
        ];
    run_validators(&lowercased, &validators)?;
    Ok(lowercased)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ok_username() {
        let r = validate_username("johN_doe99").unwrap();
        assert_eq!(r, "john_doe99");
    }

    #[test]
    fn too_long() {
        // 64 chars (exceeds 63 limit)
        let name = "abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijkl";
        let err = validate_username(name).unwrap_err();
        assert!(err.contains("Length limit exceeded"));
    }

    #[test]
    fn invalid_characters() {
        let err = validate_username("john*doe").unwrap_err();
        assert!(err.contains("Invalid character"));
    }

    #[test]
    fn starts_with_digit() {
        let err = validate_username("1abc").unwrap_err();
        assert!(err.contains("cannot start with a digit"));
    }

    #[test]
    fn case_conversion_happens_first() {
        let r = validate_username("abcXYZ").unwrap();
        assert_eq!(r, "abcxyz");
    }

    // Group ID tests
    #[test]
    fn ok_group_id_without_prefix() {
        let r = validate_group_id("mygroup").unwrap();
        assert_eq!(r, "mygroup");
    }

    #[test]
    fn ok_group_id_with_prefix() {
        // Should strip g_ prefix and validate the rest
        let r = validate_group_id("g_mygroup").unwrap();
        assert_eq!(r, "mygroup");
    }

    #[test]
    fn group_id_case_conversion() {
        let r = validate_group_id("MyGroup_123").unwrap();
        assert_eq!(r, "mygroup_123");
    }

    #[test]
    fn group_id_with_prefix_case_conversion() {
        let r = validate_group_id("g_MyGroup_123").unwrap();
        assert_eq!(r, "mygroup_123");
    }

    #[test]
    fn group_id_too_long() {
        let name = "abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijkl";
        let err = validate_group_id(name).unwrap_err();
        assert!(err.contains("Length limit exceeded"));
    }

    #[test]
    fn group_id_starts_with_digit() {
        let err = validate_group_id("1group").unwrap_err();
        assert!(err.contains("cannot start with a digit"));
    }

    #[test]
    fn group_id_with_hyphens() {
        let r = validate_group_id("my-group-name").unwrap();
        assert_eq!(r, "my-group-name");
    }

    #[test]
    fn group_id_with_hyphens_and_underscores() {
        let r = validate_group_id("my-group_name").unwrap();
        assert_eq!(r, "my-group_name");
    }

    #[test]
    fn group_id_starts_with_hyphen() {
        let err = validate_group_id("-group").unwrap_err();
        assert!(err.contains("cannot start with '-'"));
    }
}
