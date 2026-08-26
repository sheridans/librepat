use serde::{Deserialize, Serialize};
use time::{Date, Month};

/// Date presentation used in generated customer documents.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum ReportDateFormat {
    #[default]
    DayMonthNameYear,
    DayMonthYear,
    Iso,
}

impl ReportDateFormat {
    pub const ALL: [Self; 3] = [Self::DayMonthNameYear, Self::DayMonthYear, Self::Iso];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::DayMonthNameYear => "19 Aug 2026",
            Self::DayMonthYear => "19/08/2026",
            Self::Iso => "2026-08-19",
        }
    }

    #[must_use]
    pub fn format(self, date: Date) -> String {
        match self {
            Self::DayMonthNameYear => format!(
                "{:02} {} {}",
                date.day(),
                month_abbreviation(date.month()),
                date.year()
            ),
            Self::DayMonthYear => format!(
                "{:02}/{:02}/{}",
                date.day(),
                u8::from(date.month()),
                date.year()
            ),
            Self::Iso => date.to_string(),
        }
    }

    #[must_use]
    pub const fn storage_value(self) -> &'static str {
        match self {
            Self::DayMonthNameYear => "day-month-name-year",
            Self::DayMonthYear => "day-month-year",
            Self::Iso => "iso",
        }
    }

    #[must_use]
    pub fn from_storage_value(value: &str) -> Option<Self> {
        match value {
            "day-month-name-year" => Some(Self::DayMonthNameYear),
            "day-month-year" => Some(Self::DayMonthYear),
            "iso" => Some(Self::Iso),
            _ => None,
        }
    }
}

const fn month_abbreviation(month: Month) -> &'static str {
    match month {
        Month::January => "Jan",
        Month::February => "Feb",
        Month::March => "Mar",
        Month::April => "Apr",
        Month::May => "May",
        Month::June => "Jun",
        Month::July => "Jul",
        Month::August => "Aug",
        Month::September => "Sep",
        Month::October => "Oct",
        Month::November => "Nov",
        Month::December => "Dec",
    }
}

#[cfg(test)]
mod tests {
    use time::macros::date;

    use super::ReportDateFormat;

    #[test]
    fn default_format_should_be_unambiguous_customer_date() {
        assert_eq!(
            ReportDateFormat::default().format(date!(2026 - 08 - 19)),
            "19 Aug 2026"
        );
    }
}
