use std::borrow::Cow;

use librepat_core::{Detection, ImportError, ImportSource, ImporterMetadata, TestKind, TestMode};
use librepat_import::{
    FieldMapping, FieldTarget, LabelledTextAdapter, Separator, decode_ascii, has_field,
    parse_dmy_date, parse_hms_time,
};
use time::{Date, Time};

use crate::IMPORTER_METADATA;

const SPACE: Separator = Separator::Whitespace;
const MAPPINGS: &[FieldMapping] = &[
    FieldMapping::new("TESTER", FieldTarget::TesterModel, SPACE),
    FieldMapping::new("TEST NUMBER", FieldTarget::RecordNumber, SPACE),
    FieldMapping::new("DATE", FieldTarget::TestDate, SPACE),
    FieldMapping::new("TIME", FieldTarget::TestTime, SPACE),
    FieldMapping::new("APP NO", FieldTarget::ApplianceId, SPACE),
    FieldMapping::new("TEST MODE", FieldTarget::TestMode, SPACE),
    FieldMapping::new("EARTH CURRENT", FieldTarget::Preserve, SPACE),
    FieldMapping::new("USER", FieldTarget::User, SPACE),
    FieldMapping::new("SITE", FieldTarget::Site(0), SPACE),
    FieldMapping::new("TEXT", FieldTarget::Comment, SPACE),
];

/// Seaward-specific rules for PrimeTest labelled ASCII exports.
#[derive(Clone, Copy, Debug, Default)]
pub struct SeawardAdapter;

impl LabelledTextAdapter for SeawardAdapter {
    fn metadata(&self) -> ImporterMetadata {
        IMPORTER_METADATA
    }

    fn decode<'a>(&self, bytes: &'a [u8]) -> Result<Cow<'a, str>, ImportError> {
        decode_ascii(bytes)
    }

    fn detect_text(&self, _source: &ImportSource<'_>, text: &str) -> Detection {
        let records = has_field(text, &MAPPINGS[1]);
        let tester = has_field(text, &MAPPINGS[0]);
        let specific = has_field(text, &MAPPINGS[5])
            && (has_field(text, &MAPPINGS[6]) || has_result(text, "IEC"));
        if records && tester && specific {
            Detection::Certain
        } else if records && tester && has_field(text, &MAPPINGS[3]) {
            Detection::Possible
        } else {
            Detection::NotRecognized
        }
    }

    fn mappings(&self) -> &'static [FieldMapping] {
        MAPPINGS
    }

    fn parse_date(&self, value: &str) -> Option<Date> {
        parse_dmy_date(value, '/', 80).or_else(|| parse_dmy_date(value, '-', 80))
    }

    fn parse_time(&self, value: &str) -> Option<Time> {
        parse_hms_time(value)
    }

    fn test_kind(&self, name: &str) -> TestKind {
        match name {
            "VISUAL CHECK" => TestKind::Visual,
            "EARTH" => TestKind::EarthBond,
            "IEC" => TestKind::IecLead,
            "LEAD CONTINUITY" => TestKind::LeadContinuity,
            name if name == "INS" || name.starts_with("INS ") => TestKind::Insulation,
            "LOAD" => TestKind::Load,
            "LKGE" => TestKind::Leakage,
            other => TestKind::Unknown(other.to_owned()),
        }
    }

    fn parse_mode(&self, value: &str) -> TestMode {
        let value = value.trim();
        TestMode {
            code: value
                .split_whitespace()
                .next()
                .unwrap_or_default()
                .to_owned(),
            label: value.to_owned(),
        }
    }

    fn is_end_marker(&self, line: &str) -> bool {
        line == "END OF DATA"
    }
}

fn has_result(text: &str, name: &str) -> bool {
    text.lines().any(|line| {
        line.strip_prefix(name)
            .and_then(|value| value.chars().next())
            .is_some_and(char::is_whitespace)
    })
}
