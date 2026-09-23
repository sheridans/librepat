use librepat_core::{ApplianceStatus, Job};
use time::Date;

/// Totals and testing period used by reports and the desktop summary.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct JobSummary {
    pub appliances: usize,
    pub passed: usize,
    pub failed: usize,
    pub first_test: Option<Date>,
    pub last_test: Option<Date>,
}

#[must_use]
pub fn summarize_job(job: &Job) -> JobSummary {
    let mut summary = JobSummary::default();
    for appliance in job.appliances.iter().filter(|appliance| !appliance.removed) {
        summary.appliances += 1;
        match appliance.status {
            ApplianceStatus::Pass => summary.passed += 1,
            ApplianceStatus::Fail => summary.failed += 1,
        }
        if let Some(date) = appliance.test_date {
            summary.first_test = Some(summary.first_test.map_or(date, |current| current.min(date)));
            summary.last_test = Some(summary.last_test.map_or(date, |current| current.max(date)));
        }
    }
    summary
}
