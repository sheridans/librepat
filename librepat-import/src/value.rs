use librepat_core::{Comparison, Measurement};
use time::{Date, Month, Time};

pub(crate) fn parse_measurement(tokens: &[&str]) -> (String, Option<Measurement>) {
    if tokens.len() >= 2
        && let Some((comparison, value, unit)) = split_reading(tokens[tokens.len() - 1])
    {
        return (
            tokens[..tokens.len() - 1].join(" "),
            Some(Measurement {
                comparison,
                value,
                unit,
            }),
        );
    }
    if tokens.len() >= 3
        && let Some((comparison, value)) = parse_exact(tokens[tokens.len() - 2])
    {
        return (
            tokens[..tokens.len() - 2].join(" "),
            Some(Measurement {
                comparison,
                value,
                unit: tokens[tokens.len() - 1].to_owned(),
            }),
        );
    }
    (tokens.join(" "), None)
}

pub(crate) fn parse_exact(input: &str) -> Option<(Comparison, String)> {
    let (comparison, value) = if let Some(value) = input.strip_prefix("<=") {
        (Comparison::LessThanOrEqual, value)
    } else if let Some(value) = input.strip_prefix(">=") {
        (Comparison::GreaterThanOrEqual, value)
    } else if let Some(value) = input.strip_prefix('<') {
        (Comparison::LessThan, value)
    } else if let Some(value) = input.strip_prefix('>') {
        (Comparison::GreaterThan, value)
    } else if let Some(value) = input.strip_prefix('=') {
        (Comparison::Equal, value)
    } else {
        (Comparison::Equal, input)
    };
    is_decimal(value).then(|| (comparison, value.to_owned()))
}

fn split_reading(input: &str) -> Option<(Comparison, String, String)> {
    let unit_index = input.find(|character: char| character.is_ascii_alphabetic())?;
    let (reading, unit) = input.split_at(unit_index);
    let (comparison, value) = parse_exact(reading)?;
    (!unit.is_empty()).then(|| (comparison, value, unit.to_owned()))
}

fn is_decimal(value: &str) -> bool {
    let unsigned = value.strip_prefix(['+', '-']).unwrap_or(value);
    !unsigned.is_empty()
        && unsigned.chars().any(|character| character.is_ascii_digit())
        && unsigned
            .chars()
            .filter(|character| *character == '.')
            .count()
            <= 1
        && unsigned
            .chars()
            .all(|character| character.is_ascii_digit() || character == '.')
}

#[must_use]
pub fn parse_dmy_date(input: &str, separator: char, year_pivot: i32) -> Option<Date> {
    let mut parts = input.trim().split(separator);
    let day = parts.next()?.parse::<u8>().ok()?;
    let month = parse_month(parts.next()?)?;
    let raw_year = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    let year = match raw_year.len() {
        2 => {
            let year = raw_year.parse::<i32>().ok()?;
            if year >= year_pivot {
                1900 + year
            } else {
                2000 + year
            }
        }
        4 => raw_year.parse::<i32>().ok()?,
        _ => return None,
    };
    Date::from_calendar_date(year, month, day).ok()
}

fn parse_month(input: &str) -> Option<Month> {
    if let Ok(number) = input.parse::<u8>() {
        return Month::try_from(number).ok();
    }
    match input.to_ascii_uppercase().as_str() {
        "JAN" => Some(Month::January),
        "FEB" => Some(Month::February),
        "MAR" => Some(Month::March),
        "APR" => Some(Month::April),
        "MAY" => Some(Month::May),
        "JUN" => Some(Month::June),
        "JUL" => Some(Month::July),
        "AUG" => Some(Month::August),
        "SEP" => Some(Month::September),
        "OCT" => Some(Month::October),
        "NOV" => Some(Month::November),
        "DEC" => Some(Month::December),
        _ => None,
    }
}

#[must_use]
pub fn parse_hms_time(input: &str) -> Option<Time> {
    let mut parts = input.trim().split(':');
    let hour = parts.next()?.parse::<u8>().ok()?;
    let minute = parts.next()?.parse::<u8>().ok()?;
    let second = parts.next().map_or(Some(0), |value| value.parse().ok())?;
    if parts.next().is_some() {
        return None;
    }
    Time::from_hms(hour, minute, second).ok()
}

#[cfg(test)]
mod tests {
    use librepat_core::Comparison;

    use super::parse_measurement;

    #[test]
    fn combined_reading_should_preserve_a_numeric_test_name_suffix() {
        let (name, measurement) = parse_measurement(&["INS", "1", ">299.9MEG"]);
        let measurement = measurement.unwrap_or_else(|| panic!("combined reading was not parsed"));
        assert_eq!(
            (
                name.as_str(),
                measurement.comparison,
                measurement.value.as_str(),
                measurement.unit.as_str(),
            ),
            ("INS 1", Comparison::GreaterThan, "299.9", "MEG")
        );
    }

    #[test]
    fn separate_reading_and_unit_should_remain_supported() {
        let (name, measurement) = parse_measurement(&["INS", "1", ">299.9", "MEG"]);
        let measurement = measurement.unwrap_or_else(|| panic!("separate reading was not parsed"));
        assert_eq!(
            (
                name.as_str(),
                measurement.comparison,
                measurement.value.as_str(),
                measurement.unit.as_str(),
            ),
            ("INS 1", Comparison::GreaterThan, "299.9", "MEG")
        );
    }
}
