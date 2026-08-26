use std::collections::HashSet;

use librepat_core::{
    ImportError, ImportSource, ImportWarning, ImportWarningKind, ImportedJob, Job, JobMetadata,
    SourceArchive,
};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;

use crate::{
    FieldTarget, LabelledTextAdapter,
    adapter::RecordFieldTarget,
    field,
    record::{RecordBuilder, warning},
};

pub(crate) fn parse_source<A: LabelledTextAdapter>(
    adapter: &A,
    source: &ImportSource<'_>,
    text: &str,
) -> Result<ImportedJob, ImportError> {
    let mut state = ParserState::new(adapter);
    for (index, line) in text.lines().enumerate() {
        if !state.consume(line, index + 1) {
            break;
        }
    }
    state.finish(source)
}

struct ParserState<'a, A> {
    adapter: &'a A,
    model: String,
    serial: String,
    current: Option<RecordBuilder>,
    appliances: Vec<librepat_core::Appliance>,
    warnings: Vec<ImportWarning>,
    site_name: String,
    tester_name: String,
    record_ids: HashSet<String>,
    appliance_ids: HashSet<String>,
    saw_end: bool,
}

impl<'a, A: LabelledTextAdapter> ParserState<'a, A> {
    fn new(adapter: &'a A) -> Self {
        Self {
            adapter,
            model: String::new(),
            serial: String::new(),
            current: None,
            appliances: Vec::new(),
            warnings: Vec::new(),
            site_name: String::new(),
            tester_name: String::new(),
            record_ids: HashSet::new(),
            appliance_ids: HashSet::new(),
            saw_end: false,
        }
    }

    fn consume(&mut self, line: &str, line_number: usize) -> bool {
        if line.trim().is_empty() {
            return true;
        }
        if self.adapter.is_end_marker(line.trim()) {
            self.finish_record();
            self.saw_end = true;
            return false;
        }
        if let Some((mapping, value)) = self
            .adapter
            .mappings()
            .iter()
            .find_map(|mapping| field(line, mapping).map(|value| (mapping, value)))
        {
            self.consume_field(mapping.target, mapping.label, value, line, line_number);
        } else if let Some(record) = self.current.as_mut() {
            if !record.add_result(self.adapter, line) {
                let name = line.split_whitespace().next().unwrap_or("UNKNOWN");
                record.preserve(name, line, line_number);
                self.warnings.push(warning(
                    ImportWarningKind::UnknownLine,
                    "preserved unrecognized record line in raw fields".into(),
                    &record.number,
                    line_number,
                ));
            }
        } else {
            self.warnings.push(warning(
                ImportWarningKind::UnknownLine,
                "ignored line outside an appliance record".into(),
                "",
                line_number,
            ));
        }
        true
    }

    fn consume_field(
        &mut self,
        target: FieldTarget,
        label: &str,
        value: &str,
        line: &str,
        line_number: usize,
    ) {
        let target = match target {
            FieldTarget::TesterModel => {
                self.model = value.trim().to_owned();
                return;
            }
            FieldTarget::TesterSerial => {
                self.serial = value.trim().to_owned();
                return;
            }
            FieldTarget::RecordNumber => {
                self.finish_record();
                self.start_record(value.trim(), line_number);
                return;
            }
            FieldTarget::TestDate => RecordFieldTarget::TestDate,
            FieldTarget::TestTime => RecordFieldTarget::TestTime,
            FieldTarget::ApplianceId => RecordFieldTarget::ApplianceId,
            FieldTarget::TestMode => RecordFieldTarget::TestMode,
            FieldTarget::Site(index) => RecordFieldTarget::Site(index),
            FieldTarget::User => RecordFieldTarget::User,
            FieldTarget::Description(index) => RecordFieldTarget::Description(index),
            FieldTarget::Location(index) => RecordFieldTarget::Location(index),
            FieldTarget::Comment => RecordFieldTarget::Comment,
            FieldTarget::Limit => RecordFieldTarget::Limit,
            FieldTarget::Preserve => RecordFieldTarget::Preserve,
        };
        let Some(record) = self.current.as_mut() else {
            self.warnings.push(warning(
                ImportWarningKind::UnknownLine,
                "ignored field outside an appliance record".into(),
                "",
                line_number,
            ));
            return;
        };
        apply_record_field(
            self.adapter,
            record,
            target,
            label,
            value,
            line,
            line_number,
            &mut self.warnings,
        );
    }

    fn start_record(&mut self, number: &str, line_number: usize) {
        if !number.is_empty() && !self.record_ids.insert(number.to_owned()) {
            self.warnings.push(warning(
                ImportWarningKind::DuplicateId,
                format!("duplicate source record number `{number}`"),
                number,
                line_number,
            ));
        }
        self.current = Some(RecordBuilder {
            start_line: line_number,
            number: number.to_owned(),
            ..RecordBuilder::default()
        });
    }

    fn finish_record(&mut self) {
        let Some(record) = self.current.take() else {
            return;
        };
        let Some((appliance, site)) = record.finish(self.adapter, &mut self.warnings) else {
            return;
        };
        if !appliance.appliance_id.is_empty()
            && !self.appliance_ids.insert(appliance.appliance_id.clone())
        {
            self.warnings.push(ImportWarning {
                kind: ImportWarningKind::DuplicateId,
                message: format!("duplicate appliance ID `{}`", appliance.appliance_id),
                record: Some(appliance.source_record_number.clone()),
                line: None,
            });
        }
        if self.site_name.is_empty() {
            self.site_name = site.joined;
        } else if !site.joined.is_empty() && self.site_name != site.joined {
            self.warnings.push(ImportWarning {
                kind: ImportWarningKind::InconsistentField,
                message: "site fields differ between appliance records".into(),
                record: Some(appliance.source_record_number.clone()),
                line: None,
            });
        }
        if self.tester_name.is_empty() {
            self.tester_name.clone_from(&appliance.user);
        }
        self.appliances.push(appliance);
    }

    fn finish(mut self, source: &ImportSource<'_>) -> Result<ImportedJob, ImportError> {
        self.finish_record();
        if self.appliances.is_empty() {
            return Err(ImportError::NoUsableRecords);
        }
        if self.adapter.requires_end_marker() && !self.saw_end {
            self.warnings.push(ImportWarning {
                kind: ImportWarningKind::IncompleteRecord,
                message: "source has no end-of-data marker".into(),
                record: None,
                line: None,
            });
        }
        let metadata = JobMetadata {
            site_name: self.site_name,
            tester_name: self.tester_name,
            ..JobMetadata::default()
        };
        let source_archive = SourceArchive {
            provenance: self.adapter.metadata().provenance(),
            filename: source.filename.to_owned(),
            bytes: source.bytes.to_vec(),
            sha256: Sha256::digest(source.bytes).into(),
            imported_at: OffsetDateTime::now_utc(),
            tester_model: if self.model.is_empty() {
                self.adapter.default_tester_model().to_owned()
            } else {
                self.model
            },
            tester_serial: self.serial,
        };
        Ok(ImportedJob {
            job: Job {
                original_metadata: metadata.clone(),
                metadata,
                appliances: self.appliances,
                source: source_archive,
            },
            warnings: self.warnings,
        })
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "field dispatch carries source context"
)]
fn apply_record_field<A: LabelledTextAdapter>(
    adapter: &A,
    record: &mut RecordBuilder,
    target: RecordFieldTarget,
    label: &str,
    value: &str,
    line: &str,
    line_number: usize,
    warnings: &mut Vec<ImportWarning>,
) {
    match target {
        RecordFieldTarget::TestDate => record.set_date(adapter, value, line_number, warnings),
        RecordFieldTarget::TestTime => record.set_time(adapter, value, line_number, warnings),
        RecordFieldTarget::ApplianceId => record.appliance_id = value.to_owned(),
        RecordFieldTarget::TestMode => record.set_mode(adapter, value, line_number, warnings),
        RecordFieldTarget::Site(index) => {
            RecordBuilder::set_segment(&mut record.site, index, value)
        }
        RecordFieldTarget::User => record.user = value.to_owned(),
        RecordFieldTarget::Description(index) => {
            RecordBuilder::set_segment(&mut record.description, index, value);
        }
        RecordFieldTarget::Location(index) => {
            RecordBuilder::set_segment(&mut record.location, index, value);
        }
        RecordFieldTarget::Comment => {
            record.comments.push(value.to_owned());
            record.preserve(label, line, line_number);
        }
        RecordFieldTarget::Limit => {
            if !record.attach_limit(adapter, value.trim()) {
                record.preserve(label, line, line_number);
                warnings.push(warning(
                    ImportWarningKind::InconsistentField,
                    "limit does not follow a measurement".into(),
                    &record.number,
                    line_number,
                ));
            }
        }
        RecordFieldTarget::Preserve => record.preserve(label, line, line_number),
    }
}
