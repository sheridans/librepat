use librepat_core::{ImportSource, TesterImporter};
use librepat_fluke::FlukeImporter;
use time::macros::date;

fn import(bytes: &[u8]) -> librepat_core::ImportedJob {
    FlukeImporter::default()
        .import(&ImportSource::new("anonymous.flk", bytes))
        .unwrap_or_else(|error| panic!("synthetic import failed: {error}"))
}

#[test]
fn end_marker_should_stop_later_records_from_being_imported() {
    let imported = import(
        b"MODEL SYNTHETIC\nTEST NUMBER 1\nAPP NO A-001\nVISUAL CHECK P\nEND OF DATA\nTEST NUMBER 2\nAPP NO A-002\nVISUAL CHECK F\n",
    );
    assert_eq!(
        imported
            .job
            .appliances
            .iter()
            .map(|appliance| appliance.appliance_id.as_str())
            .collect::<Vec<_>>(),
        ["A-001"]
    );
}

#[test]
fn header_and_record_fields_should_reach_their_destinations() {
    let imported = import(
        b"MODEL SYNTHETIC\nSN ANONYMOUS-0001\nTEST NUMBER 7\nDATE 24-AUG-26\nAPP NO A-007\nVISUAL CHECK P\nEND OF DATA\n",
    );
    let appliance = &imported.job.appliances[0];
    assert_eq!(
        (
            imported.job.source.tester_model.as_str(),
            imported.job.source.tester_serial.as_str(),
            appliance.source_record_number.as_str(),
            appliance.appliance_id.as_str(),
            appliance.test_date,
        ),
        (
            "SYNTHETIC",
            "ANONYMOUS-0001",
            "7",
            "A-007",
            Some(date!(2026 - 08 - 24)),
        )
    );
}
