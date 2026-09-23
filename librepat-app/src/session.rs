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

    pub fn set_removed(
        &mut self,
        selection: &BTreeSet<usize>,
        removed: bool,
    ) -> Result<(), String> {
        if let Some(index) = selection
            .iter()
            .find(|&&index| index >= self.job.appliances.len())
        {
            return Err(format!(
                "Appliance selection contains invalid index {index}"
            ));
        }
        for &index in selection {
            self.job.appliances[index].removed = removed;
        }
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
    pub selection_anchor: Option<usize>,
    pub show_removed: bool,
    pub removal_request: Option<RemovalRequest>,
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
            selection_anchor: None,
            show_removed: false,
            removal_request: None,
            exact_date: String::new(),
            shift_days: String::new(),
            detail: None,
        }
    }
}

#[derive(Clone)]
pub(crate) struct RemovalRequest {
    pub selection: BTreeSet<usize>,
    pub removed: bool,
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
            appliance.removed == state.show_removed
                && (needle.is_empty()
                    || matches_filter(appliance, &needle, job.metadata.report_date_format))
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
#[path = "session_tests.rs"]
mod tests;
