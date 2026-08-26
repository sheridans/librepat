use serde::{Deserialize, Serialize};
use time::{Date, OffsetDateTime, Time};

use crate::ImportWarning;

/// Editable postal address fields used for customer, site, and contractor details.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct PostalAddress {
    pub line_1: String,
    pub line_2: String,
    pub town: String,
    pub county: String,
    pub postcode: String,
}

/// Editable job and certificate metadata.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct JobMetadata {
    pub customer_name: String,
    pub customer_address: PostalAddress,
    pub site_name: String,
    pub site_address: PostalAddress,
    pub contractor_name: String,
    pub contractor_address: PostalAddress,
    pub contractor_telephone: String,
    pub contractor_email: String,
    pub tester_name: String,
    pub certificate_number: String,
    pub order_reference: String,
    pub valid_until: Option<Date>,
    pub report_date_format: crate::ReportDateFormat,
    pub report_notes: String,
}

/// A tester mode code and its optional text label.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct TestMode {
    pub code: String,
    pub label: String,
}

/// Raw segments and their display form for continued tester fields.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContinuedText {
    pub joined: String,
    pub segments: Vec<String>,
}

impl ContinuedText {
    #[must_use]
    pub fn from_segments(segments: Vec<String>) -> Self {
        let joined = segments
            .iter()
            .map(|segment| segment.trim())
            .filter(|segment| !segment.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        Self { joined, segments }
    }
}

/// Imported fields that do not have a tester-independent representation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RawField {
    pub name: String,
    pub value: String,
    pub line_number: usize,
}

/// Comparison marker associated with an exact decimal reading.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Comparison {
    LessThan,
    LessThanOrEqual,
    Equal,
    GreaterThanOrEqual,
    GreaterThan,
}

impl Comparison {
    #[must_use]
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::LessThan => "<",
            Self::LessThanOrEqual => "<=",
            Self::Equal => "",
            Self::GreaterThanOrEqual => ">=",
            Self::GreaterThan => ">",
        }
    }
}

/// Exact tester reading. The decimal text is never converted to floating point.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Measurement {
    pub comparison: Comparison,
    pub value: String,
    pub unit: String,
}

impl Measurement {
    #[must_use]
    pub fn display_value(&self) -> String {
        format!("{}{} {}", self.comparison.symbol(), self.value, self.unit)
            .trim_end()
            .to_owned()
    }
}

/// Exact tester threshold attached to the preceding measurement.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Limit {
    pub comparison: Comparison,
    pub value: String,
    pub unit: String,
    pub raw: String,
}

/// Tester-independent test classification with lossless unknown support.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum TestKind {
    Visual,
    LeadContinuity,
    IecLead,
    EarthBond,
    Insulation,
    PolarityContinuity,
    TouchCurrent,
    Load,
    Current,
    Leakage,
    SubstituteLeakage,
    Pelv,
    Unknown(String),
}

/// Outcome reported for an individual test.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TestStatus {
    Pass,
    Fail,
    Skipped,
}

/// Immutable imported electrical result.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TestResult {
    pub kind: TestKind,
    pub raw_name: String,
    pub status: TestStatus,
    pub measurement: Option<Measurement>,
    pub limit: Option<Limit>,
    pub raw_line: String,
}

/// Derived appliance outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ApplianceStatus {
    Pass,
    Fail,
}

impl ApplianceStatus {
    #[must_use]
    pub fn from_tests(tests: &[TestResult]) -> Option<Self> {
        let latest = crate::latest_tests(tests);
        if latest.iter().any(|test| test.status == TestStatus::Fail) {
            Some(Self::Fail)
        } else if latest.iter().any(|test| test.status == TestStatus::Pass) {
            Some(Self::Pass)
        } else {
            None
        }
    }
}

/// Appliance metadata and immutable imported electrical results.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Appliance {
    pub source_record_number: String,
    pub appliance_id: String,
    pub description: String,
    pub description_segments: Vec<String>,
    pub location: String,
    pub location_segments: Vec<String>,
    pub test_date: Option<Date>,
    pub test_time: Option<Time>,
    pub retest_date: Option<Date>,
    pub comments: String,
    pub mode: TestMode,
    pub user: String,
    pub tests: Vec<TestResult>,
    pub status: ApplianceStatus,
    pub raw_fields: Vec<RawField>,
}

/// Stable importer and format identity recorded when a source is imported.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ImportProvenance {
    pub importer_id: String,
    pub format_id: String,
}

impl ImportProvenance {
    #[must_use]
    pub fn new(importer_id: impl Into<String>, format_id: impl Into<String>) -> Self {
        Self {
            importer_id: importer_id.into(),
            format_id: format_id.into(),
        }
    }
}

/// Original import bytes, provenance, and tester identity embedded in a portable job.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SourceArchive {
    pub provenance: ImportProvenance,
    pub filename: String,
    pub bytes: Vec<u8>,
    pub sha256: [u8; 32],
    pub imported_at: OffsetDateTime,
    pub tester_model: String,
    pub tester_serial: String,
}

/// Editable job state loaded by the application.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Job {
    pub metadata: JobMetadata,
    pub original_metadata: JobMetadata,
    pub appliances: Vec<Appliance>,
    pub source: SourceArchive,
}

/// Result returned by an importer before the destination job is accepted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportedJob {
    pub job: Job,
    pub warnings: Vec<ImportWarning>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn result(status: TestStatus) -> TestResult {
        TestResult {
            kind: TestKind::Visual,
            raw_name: "VISUAL CHECK".into(),
            status,
            measurement: None,
            limit: None,
            raw_line: String::new(),
        }
    }

    #[test]
    fn overall_status_should_fail_when_any_test_fails() {
        assert_eq!(
            ApplianceStatus::from_tests(&[result(TestStatus::Pass), result(TestStatus::Fail)]),
            Some(ApplianceStatus::Fail)
        );
    }

    #[test]
    fn overall_status_should_pass_when_at_least_one_test_passes() {
        assert_eq!(
            ApplianceStatus::from_tests(&[result(TestStatus::Skipped), result(TestStatus::Pass)]),
            Some(ApplianceStatus::Pass)
        );
    }

    #[test]
    fn overall_status_should_use_the_final_result_for_repeated_tests() {
        assert_eq!(
            ApplianceStatus::from_tests(&[result(TestStatus::Fail), result(TestStatus::Pass)]),
            Some(ApplianceStatus::Pass)
        );
    }

    #[test]
    fn overall_status_should_be_absent_when_all_tests_are_skipped() {
        assert_eq!(
            ApplianceStatus::from_tests(&[result(TestStatus::Skipped)]),
            None
        );
    }
}
