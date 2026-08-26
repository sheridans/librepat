use librepat_core::{ApplianceStatus, Comparison, TestKind, TestStatus};
use time::{Date, OffsetDateTime, Time, format_description::well_known::Rfc3339};

use crate::StoreError;

pub(crate) fn date_to_text(date: Option<Date>) -> Option<String> {
    date.map(|value| value.to_string())
}

pub(crate) fn date_from_text(value: Option<String>) -> Result<Option<Date>, StoreError> {
    value
        .map(|text| {
            Date::parse(
                &text,
                time::macros::format_description!("[year]-[month]-[day]"),
            )
            .map_err(|error| StoreError::Corrupt(format!("invalid stored date `{text}`: {error}")))
        })
        .transpose()
}

pub(crate) fn time_to_text(value: Option<Time>) -> Result<Option<String>, StoreError> {
    value
        .map(|time| {
            time.format(time::macros::format_description!(
                "[hour]:[minute]:[second].[subsecond digits:9]"
            ))
            .map_err(|error| StoreError::Corrupt(format!("could not format test time: {error}")))
        })
        .transpose()
}

pub(crate) fn time_from_text(value: Option<String>) -> Result<Option<Time>, StoreError> {
    value
        .map(|text| {
            Time::parse(
                &text,
                time::macros::format_description!("[hour]:[minute]:[second].[subsecond digits:9]"),
            )
            .map_err(|error| StoreError::Corrupt(format!("invalid stored time `{text}`: {error}")))
        })
        .transpose()
}

pub(crate) fn timestamp_to_text(value: OffsetDateTime) -> Result<String, StoreError> {
    value
        .format(&Rfc3339)
        .map_err(|error| StoreError::Corrupt(format!("could not format import timestamp: {error}")))
}

pub(crate) fn timestamp_from_text(value: &str) -> Result<OffsetDateTime, StoreError> {
    OffsetDateTime::parse(value, &Rfc3339)
        .map_err(|error| StoreError::Corrupt(format!("invalid import timestamp: {error}")))
}

pub(crate) const fn comparison_to_text(value: Comparison) -> &'static str {
    match value {
        Comparison::LessThan => "lt",
        Comparison::LessThanOrEqual => "le",
        Comparison::Equal => "eq",
        Comparison::GreaterThanOrEqual => "ge",
        Comparison::GreaterThan => "gt",
    }
}

pub(crate) fn comparison_from_text(value: &str) -> Result<Comparison, StoreError> {
    match value {
        "lt" => Ok(Comparison::LessThan),
        "le" => Ok(Comparison::LessThanOrEqual),
        "eq" => Ok(Comparison::Equal),
        "ge" => Ok(Comparison::GreaterThanOrEqual),
        "gt" => Ok(Comparison::GreaterThan),
        other => Err(StoreError::Corrupt(format!("invalid comparison `{other}`"))),
    }
}

pub(crate) const fn test_status_to_text(value: TestStatus) -> &'static str {
    match value {
        TestStatus::Pass => "pass",
        TestStatus::Fail => "fail",
        TestStatus::Skipped => "skipped",
    }
}

pub(crate) fn test_status_from_text(value: &str) -> Result<TestStatus, StoreError> {
    match value {
        "pass" => Ok(TestStatus::Pass),
        "fail" => Ok(TestStatus::Fail),
        "skipped" => Ok(TestStatus::Skipped),
        other => Err(StoreError::Corrupt(format!(
            "invalid test status `{other}`"
        ))),
    }
}

pub(crate) const fn appliance_status_to_text(value: ApplianceStatus) -> &'static str {
    match value {
        ApplianceStatus::Pass => "pass",
        ApplianceStatus::Fail => "fail",
    }
}

pub(crate) fn appliance_status_from_text(value: &str) -> Result<ApplianceStatus, StoreError> {
    match value {
        "pass" => Ok(ApplianceStatus::Pass),
        "fail" => Ok(ApplianceStatus::Fail),
        other => Err(StoreError::Corrupt(format!(
            "invalid appliance status `{other}`"
        ))),
    }
}

pub(crate) fn test_kind_to_text(value: &TestKind) -> &'static str {
    match value {
        TestKind::Visual => "visual",
        TestKind::LeadContinuity => "lead_continuity",
        TestKind::IecLead => "iec_lead",
        TestKind::EarthBond => "earth_bond",
        TestKind::Insulation => "insulation",
        TestKind::PolarityContinuity => "polarity_continuity",
        TestKind::TouchCurrent => "touch_current",
        TestKind::Load => "load",
        TestKind::Current => "current",
        TestKind::Leakage => "leakage",
        TestKind::SubstituteLeakage => "substitute_leakage",
        TestKind::Pelv => "pelv",
        TestKind::Unknown(_) => "unknown",
    }
}

pub(crate) fn test_kind_from_text(value: &str, raw_name: &str) -> Result<TestKind, StoreError> {
    match value {
        "visual" => Ok(TestKind::Visual),
        "lead_continuity" => Ok(TestKind::LeadContinuity),
        "iec_lead" => Ok(TestKind::IecLead),
        "earth_bond" => Ok(TestKind::EarthBond),
        "insulation" => Ok(TestKind::Insulation),
        "polarity_continuity" => Ok(TestKind::PolarityContinuity),
        "touch_current" => Ok(TestKind::TouchCurrent),
        "load" => Ok(TestKind::Load),
        "current" => Ok(TestKind::Current),
        "leakage" => Ok(TestKind::Leakage),
        "substitute_leakage" => Ok(TestKind::SubstituteLeakage),
        "pelv" => Ok(TestKind::Pelv),
        "unknown" => Ok(TestKind::Unknown(raw_name.to_owned())),
        other => Err(StoreError::Corrupt(format!("invalid test kind `{other}`"))),
    }
}
