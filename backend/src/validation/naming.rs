use crate::validation::*;

pub fn validate_username(username: &str) -> Result<String, String> {
    let lowercased = force_lowercase()(username);
    let validators: Vec<ValidatorFn> = vec![
            limit_length(25),
            limit_min_length(2),
            allow_only_alphanumerics_and_specials(Some("_")),
            not_start_with_digit(),
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
        // 26 chars
        let name = "abcdefghijklmnopqrstuvwxyzz";
        let err = validate_username(name).unwrap_err();
    }

    #[test]
    fn invalid_characters() {
        let err = validate_username("john*doe").unwrap_err();
    }

    #[test]
    fn starts_with_digit() {
        let err = validate_username("1abc").unwrap_err();
    }

    #[test]
    fn case_conversion_happens_first() {
        let r = validate_username("abcXYZ").unwrap();
        assert_eq!(r, "abcxyz");
    }
}
