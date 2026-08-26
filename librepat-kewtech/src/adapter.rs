use std::borrow::Cow;

use librepat_core::{Detection, ImportError, ImportSource, ImporterMetadata, TestKind};
use librepat_import::{
    FieldMapping, FieldTarget, LabelledTextAdapter, Separator, decode_ascii, has_field,
    parse_dmy_date,
};
use time::Date;

use crate::IMPORTER_METADATA;

const SPACE: Separator = Separator::Whitespace;
const MAPPINGS: &[FieldMapping] = &[
    FieldMapping::new("TEST NUMBER", FieldTarget::RecordNumber, SPACE),
    FieldMapping::new("DATE", FieldTarget::TestDate, SPACE),
    FieldMapping::new("APP NO", FieldTarget::ApplianceId, SPACE),
    FieldMapping::new("DESCRIPTION", FieldTarget::Description(0), SPACE),
    FieldMapping::new("LOCN", FieldTarget::Location(0), SPACE),
    FieldMapping::new("SITE", FieldTarget::Site(0), SPACE),
    FieldMapping::new("TEXT", FieldTarget::Comment, SPACE),
];

/// Kewtech-specific rules for KT74/77 labelled ASCII exports.
#[derive(Clone, Copy, Debug, Default)]
pub struct KewtechAdapter;

impl LabelledTextAdapter for KewtechAdapter {
    fn metadata(&self) -> ImporterMetadata {
        IMPORTER_METADATA
    }

    fn decode<'a>(&self, bytes: &'a [u8]) -> Result<Cow<'a, str>, ImportError> {
        decode_ascii(bytes)
    }

    fn detect_text(&self, _source: &ImportSource<'_>, text: &str) -> Detection {
        let records = has_field(text, &MAPPINGS[0]);
        let fields = has_field(text, &MAPPINGS[3]) && has_field(text, &MAPPINGS[4]);
        if records && fields && has_result(text, "VISUAL CHECK") {
            Detection::Certain
        } else if records && fields {
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

    fn test_kind(&self, name: &str) -> TestKind {
        match name {
            "VISUAL CHECK" => TestKind::Visual,
            "EARTH" => TestKind::EarthBond,
            name if name == "INS" || name.starts_with("INS ") => TestKind::Insulation,
            "LOAD" => TestKind::Load,
            "LKGE" => TestKind::Leakage,
            "LEAD CONTINUITY" => TestKind::LeadContinuity,
            other => TestKind::Unknown(other.to_owned()),
        }
    }

    fn is_end_marker(&self, line: &str) -> bool {
        line == "END OF DATA"
    }

    fn default_tester_model(&self) -> &'static str {
        "Kewtech KT74/77"
    }
}

fn has_result(text: &str, name: &str) -> bool {
    text.lines().any(|line| {
        line.strip_prefix(name)
            .and_then(|value| value.chars().next())
            .is_some_and(char::is_whitespace)
    })
}
