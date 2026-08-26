use librepat_core::{
    ApplianceStatus, Comparison, Detection, ImportSource, ImportWarningKind, TestKind,
    TesterImporter,
};
use librepat_seaward::{IMPORTER_METADATA, SeawardImporter};
use time::macros::{date, time};

const FIXTURE: &[u8] = include_bytes!("fixtures/synthetic_primetest_ascii.txt");

fn import(bytes: &[u8]) -> librepat_core::ImportedJob {
    SeawardImporter::default()
        .import(&ImportSource::new("synthetic.txt", bytes))
        .unwrap_or_else(|error| panic!("fixture import failed: {error}"))
}

#[test]
fn detection_should_require_seaward_specific_markers() {
    let importer = SeawardImporter::default();
    assert_eq!(
        importer.detect(&ImportSource::new("synthetic.txt", FIXTURE)),
        Detection::Certain
    );
    assert_eq!(
        importer.detect(&ImportSource::new(
            "generic.txt",
            b"TEST NUMBER  1\nAPP NO  A-001\n"
        )),
        Detection::NotRecognized
    );
}

#[test]
fn fixture_should_normalize_fields_and_preserve_source() {
    let imported = import(FIXTURE);
    let appliance = &imported.job.appliances[0];
    assert_eq!(imported.job.source.bytes, FIXTURE);
    assert_eq!(
        imported.job.source.provenance,
        IMPORTER_METADATA.provenance()
    );
    assert_eq!(imported.job.source.tester_model, "PrimeTest 350");
    assert_eq!(imported.job.metadata.site_name, "Example Site");
    assert_eq!(imported.job.metadata.tester_name, "Operator");
    assert_eq!(appliance.source_record_number, "1");
    assert_eq!(appliance.appliance_id, "A-001");
    assert_eq!(appliance.test_date, Some(date!(2026 - 08 - 24)));
    assert_eq!(appliance.test_time, Some(time!(14:05:09)));
    assert_eq!(appliance.comments, "Anonymous synthetic appliance");
    assert_eq!(appliance.mode.code, "AUTO");
    assert_eq!(appliance.status, ApplianceStatus::Fail);
    assert_eq!(
        appliance
            .tests
            .iter()
            .map(|test| &test.kind)
            .collect::<Vec<_>>(),
        [
            &TestKind::Visual,
            &TestKind::EarthBond,
            &TestKind::IecLead,
            &TestKind::Insulation,
            &TestKind::Load,
            &TestKind::Leakage,
            &TestKind::LeadContinuity,
        ]
    );
    assert_eq!(appliance.raw_fields[0].name, "EARTH CURRENT");
    assert_eq!(
        appliance.tests[3]
            .measurement
            .as_ref()
            .map(|measurement| measurement.comparison),
        Some(Comparison::GreaterThan)
    );
}

#[test]
fn malformed_values_unknown_fields_and_duplicates_should_warn() {
    let imported = import(
        b"TESTER PrimeTest 350\nTEST NUMBER 1\nDATE 31/02/2026\nTIME 25:00\nAPP NO A-1\nTEST MODE AUTO\nEARTH CURRENT 200mA\nMYSTERY VALUE\nEARTH 0.1 OHM P\nTEST NUMBER 1\nAPP NO A-1\nEARTH 0.1 OHM P\n",
    );
    for kind in [
        ImportWarningKind::InvalidDate,
        ImportWarningKind::InvalidTime,
        ImportWarningKind::UnknownLine,
        ImportWarningKind::DuplicateId,
    ] {
        assert!(imported.warnings.iter().any(|warning| warning.kind == kind));
    }
    assert_eq!(
        imported.job.appliances[0].raw_fields[1].value,
        "MYSTERY VALUE"
    );
}
