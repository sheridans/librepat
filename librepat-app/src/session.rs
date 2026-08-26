use std::{cmp::Ordering, collections::BTreeSet, path::Path};

use librepat_core::{
    Appliance, ApplianceStatus, BulkDateEdit, BulkEditError, Job, ReportDateFormat,
    apply_bulk_date_edit, compare_identifiers,
};
use librepat_storage::{JobStore, StoreError};
use time::{Date, Time};

pub(crate) struct JobSession {
    pub job: Job,
    pub store: JobStore,
    pub dirty: bool,
    pub valid_until_text: String,
}

impl JobSession {
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let store = JobStore::open(path)?;
        let job = store.load()?;
        let valid_until_text = format_date(job.metadata.valid_until);
        Ok(Self {
            job,
            store,
            dirty: false,
            valid_until_text,
        })
    }

    pub fn save(&mut self) -> Result<(), StoreError> {
        self.store.save(&self.job)?;
        self.dirty = false;
        Ok(())
    }

    pub fn apply_bulk(
        &mut self,
        selection: &BTreeSet<usize>,
        edit: BulkDateEdit,
    ) -> Result<(), BulkEditError> {
        let indices = selection.iter().copied().collect::<Vec<_>>();
        apply_bulk_date_edit(&mut self.job.appliances, &indices, edit)?;
        self.dirty = true;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SortColumn {
    Id,
    Description,
    Location,
    TestDate,
    RetestDate,
    Status,
}

pub(crate) struct TableState {
    pub filter: String,
    pub sort: SortColumn,
    pub ascending: bool,
    pub selection: BTreeSet<usize>,
    pub exact_date: String,
    pub shift_days: String,
    pub detail: Option<DetailState>,
}

impl Default for TableState {
    fn default() -> Self {
        Self {
            filter: String::new(),
            sort: SortColumn::Id,
            ascending: true,
            selection: BTreeSet::new(),
            exact_date: String::new(),
            shift_days: String::new(),
            detail: None,
        }
    }
}

pub(crate) struct DetailState {
    pub index: usize,
    pub test_date: String,
    pub retest_date: String,
    pub error: Option<String>,
}

impl DetailState {
    pub fn new(index: usize, appliance: &Appliance) -> Self {
        Self {
            index,
            test_date: format_date(appliance.test_date),
            retest_date: format_date(appliance.retest_date),
            error: None,
        }
    }
}

pub(crate) fn visible_indices(job: &Job, state: &TableState) -> Vec<usize> {
    let needle = state.filter.trim().to_ascii_lowercase();
    let mut indices = job
        .appliances
        .iter()
        .enumerate()
        .filter(|(_, appliance)| {
            needle.is_empty() || matches_filter(appliance, &needle, job.metadata.report_date_format)
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    indices.sort_by(|&left, &right| {
        let ordering = compare(&job.appliances[left], &job.appliances[right], state.sort);
        if state.ascending {
            ordering
        } else {
            ordering.reverse()
        }
    });
    indices
}

pub(crate) fn parse_date(value: &str) -> Result<Option<Date>, String> {
    if value.trim().is_empty() {
        return Ok(None);
    }
    Date::parse(
        value.trim(),
        time::macros::format_description!("[year]-[month]-[day]"),
    )
    .map(Some)
    .map_err(|_| "Use a valid date in YYYY-MM-DD format".into())
}

pub(crate) fn format_date(value: Option<Date>) -> String {
    value.map_or_else(String::new, |date| date.to_string())
}

pub(crate) fn format_display_date(value: Option<Date>, format: ReportDateFormat) -> String {
    value.map_or_else(String::new, |date| format.format(date))
}

pub(crate) fn format_time(value: Option<Time>) -> String {
    value.map_or_else(String::new, |time| {
        format!(
            "{:02}:{:02}:{:02}",
            time.hour(),
            time.minute(),
            time.second()
        )
    })
}

fn matches_filter(appliance: &Appliance, needle: &str, format: ReportDateFormat) -> bool {
    [
        appliance.appliance_id.as_str(),
        appliance.description.as_str(),
        appliance.location.as_str(),
        status_text(appliance.status),
    ]
    .iter()
    .any(|value| value.to_ascii_lowercase().contains(needle))
        || format_display_date(appliance.test_date, format)
            .to_ascii_lowercase()
            .contains(needle)
        || format_display_date(appliance.retest_date, format)
            .to_ascii_lowercase()
            .contains(needle)
}

fn compare(left: &Appliance, right: &Appliance, column: SortColumn) -> Ordering {
    match column {
        SortColumn::Id => compare_identifiers(&left.appliance_id, &right.appliance_id),
        SortColumn::Description => left.description.cmp(&right.description),
        SortColumn::Location => left.location.cmp(&right.location),
        SortColumn::TestDate => left.test_date.cmp(&right.test_date),
        SortColumn::RetestDate => left.retest_date.cmp(&right.retest_date),
        SortColumn::Status => status_text(left.status).cmp(status_text(right.status)),
    }
}

pub(crate) const fn status_text(status: ApplianceStatus) -> &'static str {
    match status {
        ApplianceStatus::Pass => "Pass",
        ApplianceStatus::Fail => "Fail",
    }
}

#[cfg(test)]
mod tests {
    use librepat_core::{TestMode, TestResult};

    use super::*;

    fn appliance(id: &str, description: &str, location: &str) -> Appliance {
        Appliance {
            source_record_number: id.into(),
            appliance_id: id.into(),
            description: description.into(),
            description_segments: vec![description.into()],
            location: location.into(),
            location_segments: vec![location.into()],
            test_date: None,
            test_time: None,
            retest_date: None,
            comments: String::new(),
            mode: TestMode {
                code: "0".into(),
                label: "MAN".into(),
            },
            user: String::new(),
            tests: Vec::<TestResult>::new(),
            status: ApplianceStatus::Pass,
            raw_fields: Vec::new(),
        }
    }

    fn job() -> Job {
        use librepat_core::{ImportProvenance, JobMetadata, SourceArchive};
        use time::OffsetDateTime;

        Job {
            metadata: JobMetadata::default(),
            original_metadata: JobMetadata::default(),
            appliances: vec![
                appliance("B-002", "Desk fan", "Office"),
                appliance("A-001", "Kettle", "Kitchen"),
            ],
            source: SourceArchive {
                provenance: ImportProvenance {
                    importer_id: "synthetic-importer".into(),
                    format_id: "synthetic-format".into(),
                },
                filename: "synthetic.flk".into(),
                bytes: Vec::new(),
                sha256: [0; 32],
                imported_at: OffsetDateTime::UNIX_EPOCH,
                tester_model: String::new(),
                tester_serial: String::new(),
            },
        }
    }

    #[test]
    fn visible_indices_should_filter_across_location() {
        let state = TableState {
            filter: "kitchen".into(),
            ..TableState::default()
        };
        assert_eq!(visible_indices(&job(), &state), [1]);
    }

    #[test]
    fn visible_indices_should_sort_ids_ascending_by_default() {
        assert_eq!(visible_indices(&job(), &TableState::default()), [1, 0]);
    }

    #[test]
    fn parse_date_should_reject_malformed_input() {
        assert!(parse_date("31-02-2026").is_err());
    }

    #[test]
    fn display_date_should_use_the_selected_report_format() {
        assert_eq!(
            format_display_date(
                Some(time::macros::date!(2026 - 08 - 24)),
                librepat_core::ReportDateFormat::DayMonthYear,
            ),
            "24/08/2026"
        );
    }
}
