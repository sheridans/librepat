use librepat_core::{
    Appliance, ApplianceStatus, ContinuedText, ImportWarning, ImportWarningKind, Limit, RawField,
    TestMode, TestResult,
};
use time::{Date, Time};

use crate::{LabelledTextAdapter, TextField, value};

#[derive(Default)]
pub(crate) struct RecordBuilder {
    pub start_line: usize,
    pub number: String,
    pub appliance_id: String,
    pub date: Option<Date>,
    pub time: Option<Time>,
    pub mode: TestMode,
    pub user: String,
    pub site: Vec<String>,
    pub description: Vec<String>,
    pub location: Vec<String>,
    pub comments: Vec<String>,
    pub tests: Vec<TestResult>,
    pub raw_fields: Vec<RawField>,
}

impl RecordBuilder {
    pub fn set_date<A: LabelledTextAdapter>(
        &mut self,
        adapter: &A,
        value: &str,
        line: usize,
        warnings: &mut Vec<ImportWarning>,
    ) {
        self.date = adapter.parse_date(value);
        if self.date.is_none() && !value.trim().is_empty() {
            warnings.push(warning(
                ImportWarningKind::InvalidDate,
                format!("date `{}` is not valid", value.trim()),
                &self.number,
                line,
            ));
        }
    }

    pub fn set_time<A: LabelledTextAdapter>(
        &mut self,
        adapter: &A,
        value: &str,
        line: usize,
        warnings: &mut Vec<ImportWarning>,
    ) {
        self.time = adapter.parse_time(value);
        if self.time.is_none() && !value.trim().is_empty() {
            warnings.push(warning(
                ImportWarningKind::InvalidTime,
                format!("time `{}` is not valid", value.trim()),
                &self.number,
                line,
            ));
        }
    }

    pub fn set_mode<A: LabelledTextAdapter>(
        &mut self,
        adapter: &A,
        value: &str,
        line: usize,
        warnings: &mut Vec<ImportWarning>,
    ) {
        self.mode = adapter.parse_mode(value);
        if !self.mode.code.is_empty() && !adapter.mode_is_supported(&self.mode.code) {
            warnings.push(warning(
                ImportWarningKind::UnsupportedMode,
                format!("preserved unrecognized test mode `{}`", self.mode.code),
                &self.number,
                line,
            ));
        }
    }

    pub fn set_segment(segments: &mut Vec<String>, index: usize, value: &str) {
        if segments.len() <= index {
            segments.resize(index + 1, String::new());
        }
        segments[index] = value.to_owned();
    }

    pub fn add_result<A: LabelledTextAdapter>(&mut self, adapter: &A, line: &str) -> bool {
        let trimmed = line.trim_end();
        let Some(split) = trimmed.rfind(char::is_whitespace) else {
            return false;
        };
        let Some(status) = adapter.status_token(trimmed[split..].trim()) else {
            return false;
        };
        let body = trimmed[..split].trim_end();
        let tokens = body.split_whitespace().collect::<Vec<_>>();
        let (raw_name, mut measurement) = value::parse_measurement(&tokens);
        if raw_name.is_empty() {
            return false;
        }
        if let Some(measurement) = measurement.as_mut() {
            measurement.unit = adapter.normalize_unit(&measurement.unit);
        }
        self.tests.push(TestResult {
            kind: adapter.test_kind(&raw_name),
            raw_name,
            status,
            measurement,
            limit: None,
            raw_line: line.to_owned(),
        });
        true
    }

    pub fn attach_limit<A: LabelledTextAdapter>(&mut self, adapter: &A, raw: &str) -> bool {
        let Some(test) = self.tests.last_mut() else {
            return false;
        };
        if test.measurement.is_none()
            && !matches!(
                &test.kind,
                librepat_core::TestKind::Pelv | librepat_core::TestKind::Unknown(_)
            )
        {
            return false;
        }
        let tokens = raw.split_whitespace().collect::<Vec<_>>();
        let Some((comparison, value)) = tokens.first().and_then(|value| value::parse_exact(value))
        else {
            return false;
        };
        test.limit = Some(Limit {
            comparison,
            value,
            unit: adapter.normalize_unit(tokens.get(1).copied().unwrap_or_default()),
            raw: raw.to_owned(),
        });
        true
    }

    pub fn preserve(&mut self, name: &str, line: &str, line_number: usize) {
        self.raw_fields.push(RawField {
            name: name.to_owned(),
            value: line.to_owned(),
            line_number,
        });
    }

    pub fn finish<A: LabelledTextAdapter>(
        self,
        adapter: &A,
        warnings: &mut Vec<ImportWarning>,
    ) -> Option<(Appliance, ContinuedText)> {
        let record = self.number.clone();
        if record.is_empty() || self.appliance_id.trim().is_empty() || self.tests.is_empty() {
            warnings.push(ImportWarning {
                kind: ImportWarningKind::IncompleteRecord,
                message: "record is missing its number, appliance ID, or test results".into(),
                record: (!record.is_empty()).then_some(record),
                line: Some(self.start_line),
            });
        }
        if self.tests.is_empty() || (self.number.is_empty() && self.appliance_id.trim().is_empty())
        {
            return None;
        }
        let Some(status) = ApplianceStatus::from_tests(&self.tests) else {
            warnings.push(ImportWarning {
                kind: ImportWarningKind::IncompleteRecord,
                message: "discarded record with no final pass or fail result".into(),
                record: (!self.number.is_empty()).then(|| self.number.clone()),
                line: Some(self.start_line),
            });
            return None;
        };

        let site = continued_text(
            self.site,
            adapter.continuation_width(TextField::Site),
            adapter.continuation_segments(TextField::Site),
        );
        let description = continued_text(
            self.description,
            adapter.continuation_width(TextField::Description),
            adapter.continuation_segments(TextField::Description),
        );
        let location = continued_text(
            self.location,
            adapter.continuation_width(TextField::Location),
            adapter.continuation_segments(TextField::Location),
        );
        let comments = self
            .comments
            .iter()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        Some((
            Appliance {
                source_record_number: self.number,
                removed: false,
                appliance_id: self.appliance_id.trim().to_owned(),
                description: description.joined,
                description_segments: description.segments,
                location: location.joined,
                location_segments: location.segments,
                test_date: self.date,
                test_time: self.time,
                retest_date: None,
                comments,
                mode: self.mode,
                user: self.user.trim().to_owned(),
                tests: self.tests,
                status,
                raw_fields: self.raw_fields,
            },
            site,
        ))
    }
}

fn continued_text(
    mut segments: Vec<String>,
    width: Option<usize>,
    minimum_segments: usize,
) -> ContinuedText {
    segments.resize(segments.len().max(minimum_segments), String::new());
    let Some(width) = width else {
        return ContinuedText::from_segments(segments);
    };
    let mut text = String::new();
    for (index, segment) in segments.iter().enumerate() {
        text.push_str(segment);
        if index + 1 < segments.len() && segment.chars().count() < width {
            text.push(' ');
        }
    }
    ContinuedText {
        joined: text.split_whitespace().collect::<Vec<_>>().join(" "),
        segments,
    }
}

pub(crate) fn warning(
    kind: ImportWarningKind,
    message: String,
    record: &str,
    line: usize,
) -> ImportWarning {
    ImportWarning {
        kind,
        message,
        record: (!record.is_empty()).then(|| record.to_owned()),
        line: Some(line),
    }
}
