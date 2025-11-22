pub mod naming;

use std::collections::HashSet;

// --- Type Aliases for Pipeline Functions ---

// ValidatorFn: Takes an immutable string slice and returns Ok(()) on success,
// or an Err(String) containing the error message.
type ValidatorFn = Box<dyn Fn(&str) -> Result<(), String>>;

// TransformerFn: Takes an immutable string slice and returns a transformed String.
type TransformerFn = Box<dyn Fn(&str) -> String>;

// --- Validator Generator Functions ---

/// 1. Limits the length of the string to `n` characters (Unicode-aware).
pub fn limit_length(n: usize) -> ValidatorFn {
    Box::new(move |s: &str| {
        let actual_len = s.chars().count();
        if actual_len > n {
            Err(format!(
                "Length limit exceeded: {} characters found, maximum is {}",
                actual_len, n
            ))
        } else {
            Ok(())
        }
    })
}

/// Limits the length of the string to  min `n` characters (Unicode-aware).
pub fn limit_min_length(n: usize) -> ValidatorFn {
    Box::new(move |s: &str| {
        let actual_len = s.chars().count();
        if actual_len < n {
            Err(format!(
                "Length limit exceeded: {} characters found, maximum is {}",
                actual_len, n
            ))
        } else {
            Ok(())
        }
    })
}

/// Allows only ASCII alphanumerics (a-z, A-Z, 0-9) and an optional list of specific characters.
/// Note: The input `allowed_specials` is expected to be a string of characters to allow.
pub fn allow_only_alphanumerics_and_specials(allowed_specials: Option<&str>) -> ValidatorFn {
    // Convert the allowed specials into a HashSet for O(1) lookup
    let allowed_set: HashSet<char> = allowed_specials.unwrap_or("").chars().collect();

    Box::new(move |s: &str| {
        for c in s.chars() {
            // Check if it's a standard ASCII alphanumeric
            if c.is_ascii_alphanumeric() {
                continue;
            }

            // Check if it's one of the explicitly allowed special characters
            if allowed_set.contains(&c) {
                continue;
            }

            // If it's neither, validation fails
            return Err(format!(
                "Invalid character '{}' found. Only alphanumerics and allowed specials are permitted.",
                c
            ));
        }
        Ok(())
    })
}

/// Ensures the string does not start with a digit.
pub fn not_start_with_digit() -> ValidatorFn {
    Box::new(|s: &str| {
        if let Some(first_char) = s.chars().next()
            && first_char.is_ascii_digit() {
                return Err("String cannot start with a digit.".to_string());
            }
        Ok(())
    })
}

// --- Transformer Generator Functions (for case change) ---

/// Forces the input string to lowercase.
pub fn force_lowercase() -> TransformerFn {
    Box::new(|s: &str| s.to_lowercase())
}

/// Forces the input string to uppercase.
pub fn force_uppercase() -> TransformerFn {
    Box::new(|s: &str| s.to_uppercase())
}

pub fn validate_email() -> ValidatorFn {
    Box::new(|s: &str| {
        // must contain exactly one '@'
        let parts: Vec<&str> = s.split('@').collect();
        if parts.len() != 2 {
            return Err("Email must contain exactly one '@'".into());
        }

        let (left, right) = (parts[0].trim(), parts[1].trim());

        // both sides must not be empty or whitespace
        if left.is_empty() {
            return Err("Email local-part (before '@') is empty".into());
        }
        if right.is_empty() {
            return Err("Email domain (after '@') is empty".into());
        }

        // right half must contain at least one dot, and not at edges
        if !right.contains('.') {
            return Err("Email domain must contain at least one '.'".into());
        }

        // no leading '.' or trailing '.'
        if right.starts_with('.') || right.ends_with('.') {
            return Err("Email domain cannot start or end with '.'".into());
        }

        // ensure each domain segment is nonempty
        if right.split('.').any(|seg| seg.trim().is_empty()) {
            return Err("Email domain contains empty segments".into());
        }

        Ok(())
    })
}

// --- Pipeline Execution ---

/// Executes a sequence of validators against a string slice.
pub fn run_validators(s: &str, validators: &[ValidatorFn]) -> Result<(), String> {
    for validator in validators {
        validator(s)?; // The '?' operator returns early on the first Err(String)
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test suite for Transformer functions
    #[test]
    fn test_force_lowercase() {
        let transformer = force_lowercase();
        assert_eq!(transformer("HeLlO"), "hello");
        assert_eq!(transformer("123"), "123");
        assert_eq!(transformer(""), "");
    }

    #[test]
    fn test_force_uppercase() {
        let transformer = force_uppercase();
        assert_eq!(transformer("HeLlO"), "HELLO");
        assert_eq!(transformer("123"), "123");
        assert_eq!(transformer(""), "");
    }

    // Test suite for Validator Generator functions
    #[test]
    fn test_limit_length_pass() {
        let validator = limit_length(10);
        assert!(validator("abc").is_ok());
        assert!(validator("0123456789").is_ok());
        assert!(validator("").is_ok()); // Edge case: empty string
        assert!(validator("Hëllo").is_ok()); // Unicode: 5 chars, max 10
    }

    #[test]
    fn test_limit_length_fail() {
        let validator = limit_length(5);
        assert!(validator("abcdef").is_err());
        assert!(validator("HëlloWörld").is_err()); // Unicode: 10 chars, max 5
    }

    #[test]
    fn test_allow_only_alphanumerics_and_specials_pass() {
        let validator_no_specials = allow_only_alphanumerics_and_specials(None);
        assert!(validator_no_specials("Abc123Z").is_ok());

        let validator_with_specials = allow_only_alphanumerics_and_specials(Some("-_@"));
        assert!(validator_with_specials("User-Name_1@").is_ok());
    }

    #[test]
    fn test_allow_only_alphanumerics_and_specials_fail() {
        let validator_no_specials = allow_only_alphanumerics_and_specials(None);
        assert!(validator_no_specials("abc!").is_err());
        assert!(validator_no_specials("123$").is_err());
        assert!(validator_no_specials("Hëllo").is_err()); // Non-ASCII alphanumeric fails here

        let validator_with_specials = allow_only_alphanumerics_and_specials(Some("-"));
        assert!(validator_with_specials("user_name").is_err());
        assert!(validator_with_specials("user@name").is_err());
    }

    #[test]
    fn test_not_start_with_digit_pass() {
        let validator = not_start_with_digit();
        assert!(validator("UserName").is_ok());
        assert!(validator("_user").is_ok());
        assert!(validator("").is_ok()); // Edge case: empty string
        assert!(validator("a123").is_ok());
    }

    #[test]
    fn test_not_start_with_digit_fail() {
        let validator = not_start_with_digit();
        assert!(validator("1UserName").is_err());
        assert!(validator("9").is_err());
    }

    // Test suite for Pipeline Execution (run_validators)
    #[test]
    fn test_pipeline_success() {
        // Define a combination of validators that should pass
        let validators: Vec<ValidatorFn> = vec![
            limit_length(20),
            allow_only_alphanumerics_and_specials(Some("!")),
            not_start_with_digit(),
        ];

        let valid_string = "Hello!World";

        // No transformation needed, just run validators
        assert!(run_validators(valid_string, &validators).is_ok());
    }

    #[test]
    fn test_pipeline_fail_on_first_validator() {
        // Define a combination where the first validator fails
        let validators: Vec<ValidatorFn> = vec![
            limit_length(5), // Fails (length is 6)
            not_start_with_digit(),
            allow_only_alphanumerics_and_specials(None),
        ];

        let test_string = "abcdef";
        let result = run_validators(test_string, &validators);

        // Should return Err
        assert!(result.is_err());
        // Should contain the correct failure reason
        assert!(result.unwrap_err().contains("Length limit exceeded"));
    }

    #[test]
    fn test_pipeline_fail_on_last_validator() {
        // Define a combination where the last validator fails
        let validators: Vec<ValidatorFn> = vec![
            limit_length(20),                            // Passes
            not_start_with_digit(),                      // Passes
            allow_only_alphanumerics_and_specials(None), // Fails (contains '_')
        ];

        let test_string = "Valid_Name";
        let result = run_validators(test_string, &validators);

        // Should return Err
        assert!(result.is_err());
        // Should contain the correct failure reason
        assert!(result.unwrap_err().contains("Invalid character '_' found"));
    }
}
