use std::cmp::Ordering;

/// Compares numeric appliance IDs by value and other IDs case-insensitively.
#[must_use]
pub fn compare_identifiers(left: &str, right: &str) -> Ordering {
    let left = left.trim();
    let right = right.trim();
    match (normalized_digits(left), normalized_digits(right)) {
        (Some(left_digits), Some(right_digits)) => left_digits
            .len()
            .cmp(&right_digits.len())
            .then_with(|| left_digits.cmp(right_digits))
            .then_with(|| left.len().cmp(&right.len()))
            .then_with(|| left.cmp(right)),
        _ => left
            .to_ascii_lowercase()
            .cmp(&right.to_ascii_lowercase())
            .then_with(|| left.cmp(right)),
    }
}

fn normalized_digits(value: &str) -> Option<&str> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let normalized = value.trim_start_matches('0');
    Some(if normalized.is_empty() {
        "0"
    } else {
        normalized
    })
}

#[cfg(test)]
mod tests {
    use super::compare_identifiers;

    #[test]
    fn numeric_identifiers_should_sort_by_value() {
        let mut identifiers = ["10", "2", "001", "1"];
        identifiers.sort_by(|left, right| compare_identifiers(left, right));
        assert_eq!(identifiers, ["1", "001", "2", "10"]);
    }
}
