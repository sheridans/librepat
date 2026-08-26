use librepat_core::{
    ApplianceStatus, Comparison, ImportError, ImportSource, ImportWarningKind, TestKind,
    TestStatus, TesterImporter,
};
use librepat_fluke::FlukeImporter;

const FIXTURE: &[u8] = include_bytes!("fixtures/synthetic_modes.txt");

fn import(bytes: &[u8]) -> librepat_core::ImportedJob {
    FlukeImporter::default()
        .import(&ImportSource::new("synthetic.flk", bytes))
        .unwrap_or_else(|error| panic!("fixture import failed: {error}"))
}

#[test]
fn fixture_should_cover_all_observed_modes() {
    let imported = import(FIXTURE);
    let modes = imported
        .job
        .appliances
        .iter()
        .map(|appliance| appliance.mode.code.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        modes,
        [
            "0", "133", "136", "231", "232", "233", "333", "334", "336", "0"
        ]
    );
}

#[test]
fn crlf_input_should_preserve_original_bytes() {
    let crlf = String::from_utf8_lossy(FIXTURE).replace('\n', "\r\n");
    let imported = import(crlf.as_bytes());
    assert_eq!(imported.job.source.bytes, crlf.as_bytes());
}

#[test]
fn continuation_fields_should_join_and_retain_segments() {
    let imported = import(FIXTURE);
    let appliance = &imported.job.appliances[0];
    assert_eq!(appliance.description, "Bench supply");
    assert_eq!(appliance.description_segments, ["Bench", "supply", ""]);
}

#[test]
fn continuation_fields_should_rejoin_words_split_at_field_boundaries() {
    let imported = import(
        b"MODEL  SYNTHETIC\nTEST NUMBER  1\nAPP NO  A-001\nSITE  EXAMPLE HOUS\nSITE1 E   NORTH WI\nSITE2 NG\nVISUAL CHECK  P\nEND OF DATA\n",
    );
    assert_eq!(imported.job.metadata.site_name, "EXAMPLE HOUSE NORTH WING");
}

#[test]
fn exact_reading_should_preserve_comparison_and_decimal() {
    let imported = import(FIXTURE);
    let measurement = imported.job.appliances[0].tests[2]
        .measurement
        .as_ref()
        .unwrap_or_else(|| panic!("expected insulation measurement"));
    assert_eq!(
        (measurement.comparison, measurement.value.as_str()),
        (Comparison::GreaterThan, "299.9")
    );
}

#[test]
fn limit_should_attach_to_preceding_measurement() {
    let imported = import(FIXTURE);
    let limit = imported.job.appliances[0].tests[1]
        .limit
        .as_ref()
        .unwrap_or_else(|| panic!("expected earth limit"));
    assert_eq!((limit.value.as_str(), limit.unit.as_str()), ("0.10", "OHM"));
}

#[test]
fn limit_without_measurement_should_be_preserved_and_warned() {
    let imported = import(
        b"MODEL  SYNTHETIC\nTEST NUMBER  1\nAPP NO  A-001\nVISUAL CHECK  P\nLIMIT  >0.10 OHM\nEND OF DATA\n",
    );
    assert_eq!(imported.job.appliances[0].tests[0].limit, None);
    assert_eq!(
        imported.job.appliances[0].raw_fields[0].value,
        "LIMIT  >0.10 OHM"
    );
    assert!(imported.warnings.iter().any(|warning| {
        warning.kind == ImportWarningKind::InconsistentField
            && warning.message.contains("does not follow a measurement")
    }));
}

#[test]
fn status_should_fail_when_any_test_fails() {
    let imported = import(FIXTURE);
    assert_eq!(imported.job.appliances[1].status, ApplianceStatus::Fail);
}

#[test]
fn status_should_use_the_final_repeated_test_result() {
    let imported = import(
        b"MODEL  SYNTHETIC\nTEST NUMBER  1\nAPP NO  A-001\nEARTH  9.99 OHM F\nEARTH  0.04 OHM P\nEND OF DATA\n",
    );
    assert_eq!(imported.job.appliances[0].tests.len(), 2);
    assert_eq!(imported.job.appliances[0].status, ApplianceStatus::Pass);
}

#[test]
fn records_with_only_skipped_tests_should_be_discarded() {
    let imported = import(
        b"MODEL SYNTHETIC\nTEST NUMBER 1\nAPP NO A-001\nVISUAL CHECK S\nTEST NUMBER 2\nAPP NO A-002\nVISUAL CHECK P\nEND OF DATA\n",
    );
    assert_eq!(imported.job.appliances.len(), 1);
    assert_eq!(imported.job.appliances[0].appliance_id, "A-002");
    assert!(imported.warnings.iter().any(|warning| {
        warning.kind == ImportWarningKind::IncompleteRecord
            && warning.message.contains("no final pass or fail")
    }));
}

#[test]
fn skipped_unknown_test_should_be_preserved() {
    let imported = import(FIXTURE);
    let test = &imported.job.appliances[9].tests[1];
    assert_eq!(test.kind, TestKind::Unknown("ARC TEST".into()));
    assert_eq!(test.status, TestStatus::Skipped);
}

#[test]
fn pelv_test_should_import_without_mode_schema() {
    let imported = import(FIXTURE);
    assert_eq!(imported.job.appliances[9].tests[0].kind, TestKind::Pelv);
}

#[test]
fn pelv_limit_should_attach_without_a_reported_reading() {
    let imported = import(
        b"MODEL  SYNTHETIC\nTEST NUMBER  1\nAPP NO  A-001\nPROBE PELV  P\nLIMIT  25 V\nEND OF DATA\n",
    );
    let limit = imported.job.appliances[0].tests[0]
        .limit
        .as_ref()
        .unwrap_or_else(|| panic!("expected PELV limit"));
    assert_eq!((limit.value.as_str(), limit.unit.as_str()), ("25", "V"));
}

#[test]
fn bond_range_should_be_preserved_without_an_import_warning() {
    let imported = import(
        b"MODEL  SYNTHETIC\nTEST NUMBER  1\nAPP NO  A-001\nBOND RANGE   200mA\nEARTH  0.10 OHM P\nEND OF DATA\n",
    );
    assert!(
        imported.warnings.is_empty(),
        "unexpected warnings: {:?}",
        imported.warnings
    );
}

#[test]
fn missing_end_marker_should_return_warning() {
    let text = String::from_utf8_lossy(FIXTURE).replace("END OF DATA", "");
    let imported = import(text.as_bytes());
    assert!(imported.warnings.iter().any(|warning| {
        warning.kind == ImportWarningKind::IncompleteRecord
            && warning.message.contains("end-of-data")
    }));
}

#[test]
fn malformed_date_should_warn_and_leave_date_blank() {
    let text = String::from_utf8_lossy(FIXTURE).replacen("01-JAN-80", "31-FEB-24", 1);
    let imported = import(text.as_bytes());
    assert_eq!(imported.job.appliances[0].test_date, None);
    assert!(
        imported
            .warnings
            .iter()
            .any(|warning| warning.kind == ImportWarningKind::InvalidDate)
    );
}

#[test]
fn numeric_month_should_remain_invalid_for_fluke_dates() {
    let imported = import(
        b"MODEL SYNTHETIC\nTEST NUMBER 1\nDATE 01-01-24\nAPP NO A-001\nVISUAL CHECK P\nEND OF DATA\n",
    );
    assert_eq!(imported.job.appliances[0].test_date, None);
    assert!(
        imported
            .warnings
            .iter()
            .any(|warning| warning.kind == ImportWarningKind::InvalidDate)
    );
}

#[test]
fn word_status_should_remain_unrecognized_for_fluke_results() {
    let result = FlukeImporter::default().import(&ImportSource::new(
        "synthetic.flk",
        b"MODEL SYNTHETIC\nTEST NUMBER 1\nAPP NO A-001\nVISUAL CHECK PASS\nEND OF DATA\n",
    ));
    assert!(matches!(result, Err(ImportError::NoUsableRecords)));
}

#[test]
fn modes_333_and_336_should_not_require_earth_result() {
    let imported = import(FIXTURE);
    let relevant = imported
        .job
        .appliances
        .iter()
        .filter(|appliance| matches!(appliance.mode.code.as_str(), "333" | "336"));
    assert!(relevant.into_iter().all(|appliance| {
        appliance
            .tests
            .iter()
            .all(|test| test.kind != TestKind::EarthBond)
    }));
}

#[test]
fn unknown_line_should_be_preserved_and_warned() {
    let text = String::from_utf8_lossy(FIXTURE).replacen(
        "LEAD CONTINUITY  P",
        "TEST VOLTAGE   500V\nLEAD CONTINUITY  P",
        1,
    );
    let imported = import(text.as_bytes());
    assert_eq!(
        imported.job.appliances[0].raw_fields[0].value,
        "TEST VOLTAGE   500V"
    );
    assert!(
        imported
            .warnings
            .iter()
            .any(|warning| warning.kind == ImportWarningKind::UnknownLine)
    );
}

#[test]
fn duplicate_appliance_id_should_warn_without_rejecting_records() {
    let text = String::from_utf8_lossy(FIXTURE).replacen("APP NO  A-002", "APP NO  A-001", 1);
    let imported = import(text.as_bytes());
    assert_eq!(imported.job.appliances.len(), 10);
    assert!(
        imported
            .warnings
            .iter()
            .any(|warning| warning.kind == ImportWarningKind::DuplicateId)
    );
}

#[test]
fn inconsistent_site_should_warn_without_losing_record() {
    let text = String::from_utf8_lossy(FIXTURE).replacen("SITE  Example", "SITE  Different", 1);
    let imported = import(text.as_bytes());
    assert_eq!(imported.job.appliances.len(), 10);
    assert!(
        imported
            .warnings
            .iter()
            .any(|warning| warning.kind == ImportWarningKind::InconsistentField)
    );
}
