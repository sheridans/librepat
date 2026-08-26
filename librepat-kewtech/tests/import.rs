use librepat_core::{
    Detection, ImportSource, ImportWarningKind, TestKind, TestStatus, TesterImporter,
};
use librepat_kewtech::{IMPORTER_METADATA, KewtechImporter};
use time::macros::date;

const FIXTURE: &[u8] = include_bytes!("fixtures/synthetic_kt_ascii.txt");

fn import(bytes: &[u8]) -> librepat_core::ImportedJob {
    KewtechImporter::default()
        .import(&ImportSource::new("synthetic.asc", bytes))
        .unwrap_or_else(|error| panic!("fixture import failed: {error}"))
}

#[test]
fn detection_should_require_kewtech_specific_fields() {
    let importer = KewtechImporter::default();
    assert_eq!(
        importer.detect(&ImportSource::new("synthetic.asc", FIXTURE)),
        Detection::Certain
    );
    assert_eq!(
        importer.detect(&ImportSource::new(
            "generic.asc",
            b"TEST NUMBER 1\nAPP NO A-1\nVISUAL CHECK P\n"
        )),
        Detection::NotRecognized
    );
}

#[test]
fn fixture_should_map_every_supported_field_and_result() {
    let imported = import(FIXTURE);
    let appliance = &imported.job.appliances[0];
    assert_eq!(
        imported.job.source.provenance,
        IMPORTER_METADATA.provenance()
    );
    assert_eq!(imported.job.source.tester_model, "Kewtech KT74/77");
    assert_eq!(imported.job.source.bytes, FIXTURE);
    assert_eq!(imported.job.metadata.site_name, "Example Site");
    assert_eq!(appliance.source_record_number, "1");
    assert_eq!(appliance.appliance_id, "A-001");
    assert_eq!(appliance.description, "Anonymous appliance");
    assert_eq!(appliance.location, "Test room");
    assert_eq!(appliance.test_date, Some(date!(2026 - 08 - 24)));
    assert_eq!(appliance.comments, "Synthetic fixture only");
    assert_eq!(appliance.raw_fields[0].name, "TEXT");
    assert_eq!(
        appliance.raw_fields[0].value,
        "TEXT  Synthetic fixture only"
    );
    assert_eq!(
        appliance
            .tests
            .iter()
            .map(|test| test.kind.clone())
            .collect::<Vec<_>>(),
        [
            TestKind::Visual,
            TestKind::EarthBond,
            TestKind::Insulation,
            TestKind::Load,
            TestKind::Leakage,
            TestKind::LeadContinuity,
        ]
    );
    assert!(
        appliance
            .tests
            .iter()
            .all(|test| test.status == TestStatus::Pass)
    );
    assert_eq!(
        appliance
            .tests
            .iter()
            .map(|test| test.measurement.as_ref().map(|value| value.display_value()))
            .collect::<Vec<_>>(),
        [
            None,
            Some("0.09 OHM".into()),
            Some(">199.9 MOHM".into()),
            Some("250 VA".into()),
            Some("0.10 mA".into()),
            None,
        ]
    );
}

#[test]
fn malformed_unknown_and_duplicate_fields_should_warn_without_data_loss() {
    let imported = import(
        b"TEST NUMBER 1\nDATE 31/02/2026\nAPP NO A-1\nDESCRIPTION Anonymous item\nLOCN Test room\nMYSTERY VALUE\nEARTH 0.09 OHM P\nTEST NUMBER 1\nAPP NO A-1\nDESCRIPTION Anonymous item\nLOCN Test room\nEARTH 0.08 OHM P\n",
    );
    assert_eq!(imported.job.appliances.len(), 2);
    assert_eq!(imported.job.appliances[0].test_date, None);
    assert_eq!(
        imported.job.appliances[0].raw_fields[0].value,
        "MYSTERY VALUE"
    );
    for kind in [
        ImportWarningKind::InvalidDate,
        ImportWarningKind::UnknownLine,
        ImportWarningKind::DuplicateId,
    ] {
        assert!(imported.warnings.iter().any(|warning| warning.kind == kind));
    }
    assert_eq!(
        imported
            .warnings
            .iter()
            .filter(|warning| warning.kind == ImportWarningKind::DuplicateId)
            .count(),
        2
    );
}
