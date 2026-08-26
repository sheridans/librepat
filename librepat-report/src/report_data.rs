use librepat_core::{
    Appliance, ApplianceStatus, Job, ReportDateFormat, TestResult, TestStatus, compare_identifiers,
};

use crate::canvas::{MUTED, TEAL};

pub(crate) fn sorted_appliances(job: &Job) -> Vec<&Appliance> {
    let mut appliances = job.appliances.iter().collect::<Vec<_>>();
    appliances.sort_by(|left, right| {
        compare_identifiers(&left.appliance_id, &right.appliance_id).then_with(|| {
            compare_identifiers(&left.source_record_number, &right.source_record_number)
        })
    });
    appliances
}

pub(crate) fn test_summary(test: &TestResult) -> String {
    let mut parts = vec![test.raw_name.clone(), test_status_text(test.status).into()];
    if let Some(measurement) = &test.measurement {
        parts.push(measurement.display_value());
    }
    if let Some(limit) = &test.limit {
        parts.push(
            format!(
                "limit {}{} {}",
                limit.comparison.symbol(),
                limit.value,
                limit.unit
            )
            .trim_end()
            .to_owned(),
        );
    }
    parts.join(" · ")
}

pub(crate) fn format_date(date: Option<time::Date>, format: ReportDateFormat) -> String {
    date.map_or_else(|| "—".into(), |value| format.format(value))
}

pub(crate) fn status_text(status: ApplianceStatus) -> &'static str {
    match status {
        ApplianceStatus::Pass => "PASS",
        ApplianceStatus::Fail => "FAIL",
    }
}

pub(crate) fn status_color(status: ApplianceStatus) -> (f32, f32, f32) {
    match status {
        ApplianceStatus::Pass => TEAL,
        ApplianceStatus::Fail => (0.68, 0.12, 0.10),
    }
}

pub(crate) const fn test_status_text(status: TestStatus) -> &'static str {
    match status {
        TestStatus::Pass => "PASS",
        TestStatus::Fail => "FAIL",
        TestStatus::Skipped => "SKIPPED",
    }
}

pub(crate) const fn test_status_color(status: TestStatus) -> (f32, f32, f32) {
    match status {
        TestStatus::Pass => TEAL,
        TestStatus::Fail => (0.68, 0.12, 0.10),
        TestStatus::Skipped => MUTED,
    }
}

pub(crate) fn blank_as_dash(value: &str) -> &str {
    if value.trim().is_empty() {
        "—"
    } else {
        value
    }
}
