use librepat_core::{
    Appliance, ApplianceStatus, ImportProvenance, Job, JobMetadata, SourceArchive, TestMode,
    TestResult,
};
use time::OffsetDateTime;

use super::*;

fn appliance(id: &str, description: &str, location: &str) -> Appliance {
    Appliance {
        source_record_number: id.into(),
        removed: false,
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
fn visible_indices_should_separate_active_and_removed_appliances() {
    let mut job = job();
    job.appliances[0].removed = true;
    let removed_state = TableState {
        show_removed: true,
        ..TableState::default()
    };
    assert_eq!(
        (
            visible_indices(&job, &TableState::default()),
            visible_indices(&job, &removed_state),
        ),
        (vec![1], vec![0])
    );
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
