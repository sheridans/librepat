use std::borrow::Cow;

use librepat_core::{Detection, ImportError, ImportSource, ImporterMetadata, TestKind, TestMode};
use time::{Date, Time};

use crate::parser::parse_source;

/// Shared destination for a labelled source field.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FieldTarget {
    TesterModel,
    TesterSerial,
    RecordNumber,
    TestDate,
    TestTime,
    ApplianceId,
    TestMode,
    Site(usize),
    User,
    Description(usize),
    Location(usize),
    Comment,
    Limit,
    Preserve,
}

#[derive(Clone, Copy)]
pub(crate) enum RecordFieldTarget {
    TestDate,
    TestTime,
    ApplianceId,
    TestMode,
    Site(usize),
    User,
    Description(usize),
    Location(usize),
    Comment,
    Limit,
    Preserve,
}

/// Whitespace rule between a label and its value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Separator {
    Whitespace,
    Exact(&'static str),
}

/// One manufacturer label and its shared destination.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FieldMapping {
    pub label: &'static str,
    pub target: FieldTarget,
    pub separator: Separator,
}

impl FieldMapping {
    #[must_use]
    pub const fn new(label: &'static str, target: FieldTarget, separator: Separator) -> Self {
        Self {
            label,
            target,
            separator,
        }
    }
}

/// Continued-text destination used to select format-specific joining rules.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextField {
    Site,
    Description,
    Location,
}

/// Manufacturer-specific rules consumed by the shared labelled-text pipeline.
pub trait LabelledTextAdapter: Clone + Send + Sync {
    fn metadata(&self) -> ImporterMetadata;
    fn decode<'a>(&self, bytes: &'a [u8]) -> Result<Cow<'a, str>, ImportError>;
    fn detect_text(&self, source: &ImportSource<'_>, text: &str) -> Detection;
    /// Returns mappings in match priority order; the parser uses the first matching entry.
    fn mappings(&self) -> &'static [FieldMapping];
    fn parse_date(&self, value: &str) -> Option<Date>;
    fn test_kind(&self, name: &str) -> TestKind;
    fn is_end_marker(&self, line: &str) -> bool;

    fn parse_time(&self, _value: &str) -> Option<Time> {
        None
    }

    fn parse_mode(&self, value: &str) -> TestMode {
        TestMode {
            code: value.trim().to_owned(),
            label: String::new(),
        }
    }

    fn mode_is_supported(&self, _code: &str) -> bool {
        true
    }

    fn default_tester_model(&self) -> &'static str {
        ""
    }

    fn requires_end_marker(&self) -> bool {
        false
    }

    fn continuation_width(&self, _field: TextField) -> Option<usize> {
        None
    }

    fn continuation_segments(&self, _field: TextField) -> usize {
        0
    }

    fn status_token(&self, token: &str) -> Option<librepat_core::TestStatus> {
        match token {
            "P" | "PASS" => Some(librepat_core::TestStatus::Pass),
            "F" | "FAIL" => Some(librepat_core::TestStatus::Fail),
            "S" | "SKIP" | "SKIPPED" => Some(librepat_core::TestStatus::Skipped),
            _ => None,
        }
    }

    fn normalize_unit(&self, unit: &str) -> String {
        unit.to_owned()
    }
}

/// `TesterImporter` implementation backed by one format adapter.
#[derive(Clone, Copy, Debug)]
pub struct LabelledTextImporter<A> {
    adapter: A,
}

impl<A> LabelledTextImporter<A> {
    #[must_use]
    pub const fn new(adapter: A) -> Self {
        Self { adapter }
    }
}

impl<A: Default> Default for LabelledTextImporter<A> {
    fn default() -> Self {
        Self::new(A::default())
    }
}

impl<A: LabelledTextAdapter> librepat_core::TesterImporter for LabelledTextImporter<A> {
    fn metadata(&self) -> ImporterMetadata {
        self.adapter.metadata()
    }

    fn detect(&self, source: &ImportSource<'_>) -> Detection {
        let Ok(text) = self.adapter.decode(source.bytes) else {
            return Detection::NotRecognized;
        };
        self.adapter.detect_text(source, &text)
    }

    fn import(&self, source: &ImportSource<'_>) -> Result<librepat_core::ImportedJob, ImportError> {
        let text = self.adapter.decode(source.bytes)?;
        if self.adapter.detect_text(source, &text) == Detection::NotRecognized {
            return Err(ImportError::NotRecognized);
        }
        parse_source(&self.adapter, source, &text)
    }
}

pub fn decode_utf8(bytes: &[u8]) -> Result<Cow<'_, str>, ImportError> {
    std::str::from_utf8(bytes)
        .map(Cow::Borrowed)
        .map_err(|_| ImportError::InvalidText)
}

pub fn decode_ascii(bytes: &[u8]) -> Result<Cow<'_, str>, ImportError> {
    if !bytes.is_ascii() {
        return Err(ImportError::NonAscii);
    }
    decode_utf8(bytes)
}

#[must_use]
pub fn field<'a>(line: &'a str, mapping: &FieldMapping) -> Option<&'a str> {
    let remainder = line.strip_prefix(mapping.label)?;
    match mapping.separator {
        Separator::Whitespace => remainder
            .chars()
            .next()
            .is_none_or(char::is_whitespace)
            .then(|| remainder.trim_start()),
        Separator::Exact(separator) => remainder.strip_prefix(separator),
    }
}

#[must_use]
pub fn has_field(text: &str, mapping: &FieldMapping) -> bool {
    text.lines().any(|line| field(line, mapping).is_some())
}

#[cfg(test)]
mod tests {
    use librepat_core::ImportError;

    use super::{decode_ascii, decode_utf8};

    #[test]
    fn ascii_decoder_should_distinguish_non_ascii_from_invalid_utf8() {
        let error = match decode_ascii("Café".as_bytes()) {
            Ok(_) => panic!("valid UTF-8 containing non-ASCII text was accepted"),
            Err(error) => error,
        };
        assert!(matches!(error, ImportError::NonAscii));
        assert_eq!(error.to_string(), "the source contains non-ASCII text");
    }

    #[test]
    fn utf8_decoder_should_keep_reporting_invalid_utf8() {
        let error = match decode_utf8(&[0xff]) {
            Ok(_) => panic!("invalid UTF-8 was accepted"),
            Err(error) => error,
        };
        assert!(matches!(error, ImportError::InvalidText));
    }
}
