use std::borrow::Cow;

use librepat_core::{
    Detection, ImportError, ImportSource, ImporterMetadata, TestKind, TestMode, TestStatus,
};
use librepat_import::{
    FieldMapping, FieldTarget, LabelledTextAdapter, Separator, TextField, decode_utf8, has_field,
    parse_dmy_date,
};
use time::Date;

use crate::IMPORTER_METADATA;

const WHITESPACE: Separator = Separator::Whitespace;
const MAPPINGS: &[FieldMapping] = &[
    FieldMapping::new("MODEL", FieldTarget::TesterModel, WHITESPACE),
    FieldMapping::new("SN", FieldTarget::TesterSerial, WHITESPACE),
    FieldMapping::new("TEST NUMBER", FieldTarget::RecordNumber, WHITESPACE),
    FieldMapping::new("DATE", FieldTarget::TestDate, WHITESPACE),
    FieldMapping::new("APP NO", FieldTarget::ApplianceId, WHITESPACE),
    FieldMapping::new("TEST MODE", FieldTarget::TestMode, WHITESPACE),
    FieldMapping::new("SITE1", FieldTarget::Site(1), Separator::Exact(" ")),
    FieldMapping::new("SITE2", FieldTarget::Site(2), Separator::Exact(" ")),
    FieldMapping::new("SITE", FieldTarget::Site(0), Separator::Exact("  ")),
    FieldMapping::new("USER", FieldTarget::User, WHITESPACE),
    FieldMapping::new("DES1", FieldTarget::Description(0), Separator::Exact("  ")),
    FieldMapping::new("DES2", FieldTarget::Description(1), Separator::Exact("  ")),
    FieldMapping::new("DES3", FieldTarget::Description(2), Separator::Exact("  ")),
    FieldMapping::new("LOC1", FieldTarget::Location(0), Separator::Exact("  ")),
    FieldMapping::new("LOC2", FieldTarget::Location(1), Separator::Exact("  ")),
    FieldMapping::new("BOND RANGE", FieldTarget::Preserve, WHITESPACE),
    FieldMapping::new("LIMIT", FieldTarget::Limit, WHITESPACE),
];

/// Fluke-specific rules for the shared labelled-text pipeline.
#[derive(Clone, Copy, Debug, Default)]
pub struct FlukeAdapter;

impl LabelledTextAdapter for FlukeAdapter {
    fn metadata(&self) -> ImporterMetadata {
        IMPORTER_METADATA
    }

    fn decode<'a>(&self, bytes: &'a [u8]) -> Result<Cow<'a, str>, ImportError> {
        decode_utf8(bytes)
    }

    fn detect_text(&self, source: &ImportSource<'_>, text: &str) -> Detection {
        let has_records = has_field(text, &MAPPINGS[2]);
        let has_model = has_field(text, &MAPPINGS[0]);
        if has_records && has_model {
            Detection::Certain
        } else if has_records
            && source
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("flk"))
        {
            Detection::Possible
        } else {
            Detection::NotRecognized
        }
    }

    fn mappings(&self) -> &'static [FieldMapping] {
        MAPPINGS
    }

    fn parse_date(&self, value: &str) -> Option<Date> {
        let month = value.trim().split('-').nth(1)?;
        if !month.bytes().all(|byte| byte.is_ascii_alphabetic()) {
            return None;
        }
        parse_dmy_date(value, '-', 80)
    }

    fn test_kind(&self, name: &str) -> TestKind {
        match name {
            "VISUAL CHECK" => TestKind::Visual,
            "LEAD CONTINUITY" => TestKind::LeadContinuity,
            "EARTH" => TestKind::EarthBond,
            name if name.starts_with("INS ") => TestKind::Insulation,
            "PN CONTINUITY" => TestKind::PolarityContinuity,
            "TOUCH" => TestKind::TouchCurrent,
            "LOAD" => TestKind::Load,
            "CURRENT" => TestKind::Current,
            "LKGE" => TestKind::Leakage,
            name if name.starts_with("SUBST ") => TestKind::SubstituteLeakage,
            "PROBE PELV" => TestKind::Pelv,
            other => TestKind::Unknown(other.to_owned()),
        }
    }

    fn parse_mode(&self, value: &str) -> TestMode {
        let mut parts = value.split_whitespace();
        TestMode {
            code: parts.next().unwrap_or_default().to_owned(),
            label: parts.collect::<Vec<_>>().join(" "),
        }
    }

    fn mode_is_supported(&self, code: &str) -> bool {
        const OBSERVED: [&str; 9] = ["0", "133", "136", "231", "232", "233", "333", "334", "336"];
        OBSERVED.contains(&code)
    }

    fn is_end_marker(&self, line: &str) -> bool {
        line == "END OF DATA"
    }

    fn status_token(&self, token: &str) -> Option<TestStatus> {
        match token {
            "P" => Some(TestStatus::Pass),
            "F" => Some(TestStatus::Fail),
            "S" => Some(TestStatus::Skipped),
            _ => None,
        }
    }

    fn requires_end_marker(&self) -> bool {
        true
    }

    fn continuation_width(&self, _field: TextField) -> Option<usize> {
        Some(12)
    }

    fn continuation_segments(&self, field: TextField) -> usize {
        match field {
            TextField::Site | TextField::Description => 3,
            TextField::Location => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_digit_year_80_should_use_twentieth_century() {
        assert_eq!(
            FlukeAdapter.parse_date("01-JAN-80").map(|date| date.year()),
            Some(1980)
        );
    }

    #[test]
    fn two_digit_year_79_should_use_twenty_first_century() {
        assert_eq!(
            FlukeAdapter.parse_date("31-DEC-79").map(|date| date.year()),
            Some(2079)
        );
    }

    #[test]
    fn detection_should_not_treat_test_number_as_a_fluke_signature() {
        let bytes = b"TEST NUMBER 1\nAPP NO A-1\nVISUAL CHECK P\n";
        assert_eq!(
            FlukeAdapter.detect_text(&ImportSource::new("source.txt", bytes), "TEST NUMBER 1\n"),
            Detection::NotRecognized
        );
        assert_eq!(
            FlukeAdapter.detect_text(&ImportSource::new("source.flk", bytes), "TEST NUMBER 1\n"),
            Detection::Possible
        );
    }
}
