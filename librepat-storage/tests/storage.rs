mod common;

use std::fs;

use common::{TestFile, create_store, sample_job};
use librepat_core::ReportDateFormat;
use librepat_storage::{APPLICATION_ID, JobStore, SCHEMA_VERSION, StoreError};
use rusqlite::Connection;
use time::macros::date;

#[test]
fn save_should_round_trip_editable_metadata_atomically() {
    let file = TestFile::new("round-trip");
    let mut store = create_store(&file);
    let mut job = store
        .load()
        .unwrap_or_else(|error| panic!("could not load test job: {error}"));
    job.metadata.customer_name = "Edited customer".into();
    job.metadata.contractor_telephone = "01234 567890".into();
    job.metadata.contractor_email = "testing@example.invalid".into();
    job.metadata.report_date_format = ReportDateFormat::DayMonthYear;
    job.appliances[0].description = "Edited description".into();
    job.appliances[0].retest_date = Some(date!(2027 - 01 - 02));
    store
        .save(&job)
        .unwrap_or_else(|error| panic!("could not save test job: {error}"));
    let loaded = store
        .load()
        .unwrap_or_else(|error| panic!("could not reload test job: {error}"));
    assert_eq!(loaded.metadata.customer_name, "Edited customer");
    assert_eq!(loaded.metadata.contractor_telephone, "01234 567890");
    assert_eq!(loaded.metadata.contractor_email, "testing@example.invalid");
    assert_eq!(
        loaded.metadata.report_date_format,
        ReportDateFormat::DayMonthYear
    );
    assert_eq!(loaded.appliances[0].description, "Edited description");
    assert_eq!(
        loaded.appliances[0].test_time,
        Some(time::macros::time!(09:30:15))
    );
    assert_eq!(loaded.appliances[0].comments, "Synthetic comment");
    assert_eq!(
        loaded.appliances[0].tests[1].kind,
        librepat_core::TestKind::IecLead
    );
    assert_eq!(
        loaded.appliances[0].retest_date,
        Some(date!(2027 - 01 - 02))
    );
}

#[test]
fn replace_should_swap_in_a_complete_new_job() {
    let file = TestFile::new("replace");
    drop(create_store(&file));
    let mut replacement = sample_job();
    replacement.metadata.site_name = "Replacement site".into();
    drop(
        JobStore::replace(file.path(), &replacement)
            .unwrap_or_else(|error| panic!("could not replace job: {error}")),
    );
    let loaded = JobStore::open(file.path())
        .and_then(|store| store.load())
        .unwrap_or_else(|error| panic!("could not load replacement: {error}"));
    assert_eq!(loaded.metadata.site_name, "Replacement site");
}

#[test]
fn replace_should_preserve_original_when_replacement_is_invalid() {
    let file = TestFile::new("replace-invalid");
    drop(create_store(&file));
    let mut replacement = sample_job();
    replacement.source.bytes = b"changed without updating hash".to_vec();
    assert!(matches!(
        JobStore::replace(file.path(), &replacement),
        Err(StoreError::SourceHashMismatch)
    ));
    let loaded = JobStore::open(file.path())
        .and_then(|store| store.load())
        .unwrap_or_else(|error| panic!("original job was not preserved: {error}"));
    assert_eq!(loaded.metadata.site_name, "Example site");
}

#[test]
fn save_should_reject_result_update_through_api() {
    let file = TestFile::new("api-result-update");
    let mut store = create_store(&file);
    let mut job = sample_job();
    let measurement = job.appliances[0].tests[0]
        .measurement
        .as_mut()
        .unwrap_or_else(|| panic!("sample result has no measurement"));
    measurement.value = "9.99".into();
    assert!(matches!(
        store.save(&job),
        Err(StoreError::ImmutableResults)
    ));
}

#[test]
fn save_should_reject_result_delete_through_api() {
    let file = TestFile::new("api-result-delete");
    let mut store = create_store(&file);
    let mut job = sample_job();
    job.appliances[0].tests.clear();
    assert!(matches!(
        store.save(&job),
        Err(StoreError::ImmutableResults)
    ));
}

#[test]
fn save_should_reject_imported_time_and_comment_updates() {
    let file = TestFile::new("api-imported-fields-update");
    let mut store = create_store(&file);
    let mut job = sample_job();
    job.appliances[0].test_time = Some(time::macros::time!(10:45:00));
    assert!(matches!(
        store.save(&job),
        Err(StoreError::ImmutableResults)
    ));

    let mut job = sample_job();
    job.appliances[0].comments = "Changed comment".into();
    assert!(matches!(
        store.save(&job),
        Err(StoreError::ImmutableResults)
    ));
}

#[test]
fn trigger_should_reject_measurement_update() {
    let file = TestFile::new("trigger-update");
    drop(create_store(&file));
    let connection = Connection::open(file.path())
        .unwrap_or_else(|error| panic!("could not open raw database: {error}"));
    let result = connection.execute("UPDATE measurement SET value = '9.99'", []);
    assert!(
        result.is_err(),
        "direct result update unexpectedly succeeded"
    );
}

#[test]
fn trigger_should_reject_test_kind_update() {
    let file = TestFile::new("trigger-kind-update");
    drop(create_store(&file));
    let connection = Connection::open(file.path())
        .unwrap_or_else(|error| panic!("could not open raw database: {error}"));
    let result = connection.execute("UPDATE test_result SET kind = 'visual'", []);
    assert!(
        result.is_err(),
        "direct test-kind update unexpectedly succeeded"
    );
}

#[test]
fn trigger_should_reject_result_delete() {
    let file = TestFile::new("trigger-delete");
    drop(create_store(&file));
    let connection = Connection::open(file.path())
        .unwrap_or_else(|error| panic!("could not open raw database: {error}"));
    let result = connection.execute("DELETE FROM test_result", []);
    assert!(
        result.is_err(),
        "direct result delete unexpectedly succeeded"
    );
}

#[test]
fn trigger_should_reject_imported_time_and_comment_updates() {
    let file = TestFile::new("trigger-imported-fields-update");
    drop(create_store(&file));
    let connection = Connection::open(file.path())
        .unwrap_or_else(|error| panic!("could not open raw database: {error}"));
    let time_result = connection.execute("UPDATE appliance SET test_time = '10:45:00'", []);
    assert!(
        time_result.is_err(),
        "direct test-time update unexpectedly succeeded"
    );
    let comment_result = connection.execute("UPDATE appliance SET comments = 'changed'", []);
    assert!(
        comment_result.is_err(),
        "direct imported-comment update unexpectedly succeeded"
    );
}

#[test]
fn trigger_should_reject_original_metadata_insert() {
    let file = TestFile::new("trigger-original-metadata");
    drop(create_store(&file));
    let connection = Connection::open(file.path())
        .unwrap_or_else(|error| panic!("could not open raw database: {error}"));
    let result = connection.execute(
        "INSERT INTO job_metadata (snapshot, field, value) VALUES ('original', 'extra', 'changed')",
        [],
    );
    assert!(
        result.is_err(),
        "direct original metadata insert unexpectedly succeeded"
    );
}

#[test]
fn open_should_detect_embedded_source_hash_mismatch() {
    let file = TestFile::new("source-hash");
    drop(create_store(&file));
    let connection = Connection::open(file.path())
        .unwrap_or_else(|error| panic!("could not open raw database: {error}"));
    connection
        .execute_batch(
            "DROP TRIGGER source_archive_no_update;
             UPDATE source_archive SET bytes = x'00';",
        )
        .unwrap_or_else(|error| panic!("could not corrupt test source: {error}"));
    drop(connection);
    assert!(matches!(
        JobStore::open(file.path()),
        Err(StoreError::SourceHashMismatch)
    ));
}

#[test]
fn open_should_refuse_future_schema_without_modifying_file() {
    let file = TestFile::new("future-schema");
    drop(create_store(&file));
    let connection = Connection::open(file.path())
        .unwrap_or_else(|error| panic!("could not open raw database: {error}"));
    connection
        .pragma_update(None, "user_version", SCHEMA_VERSION + 1)
        .unwrap_or_else(|error| panic!("could not set future schema: {error}"));
    drop(connection);
    let before = fs::read(file.path())
        .unwrap_or_else(|error| panic!("could not read database before open: {error}"));
    let result = JobStore::open(file.path());
    let after = fs::read(file.path())
        .unwrap_or_else(|error| panic!("could not read database after open: {error}"));
    assert!(matches!(
        result,
        Err(StoreError::UnsupportedSchema(version)) if version == SCHEMA_VERSION + 1
    ));
    assert_eq!(after, before);
}

#[test]
fn open_should_refuse_an_unversioned_schema() {
    let file = TestFile::new("unversioned-schema");
    drop(create_store(&file));
    let connection = Connection::open(file.path())
        .unwrap_or_else(|error| panic!("could not open raw database: {error}"));
    connection
        .pragma_update(None, "user_version", 0)
        .unwrap_or_else(|error| panic!("could not clear schema version: {error}"));
    drop(connection);
    assert!(matches!(
        JobStore::open(file.path()),
        Err(StoreError::InvalidSchema(0))
    ));
}

#[test]
fn created_database_should_use_portable_sqlite_settings() {
    let file = TestFile::new("settings");
    drop(create_store(&file));
    let connection = Connection::open(file.path())
        .unwrap_or_else(|error| panic!("could not open raw database: {error}"));
    let application_id: i64 = connection
        .query_row("PRAGMA application_id", [], |row| row.get(0))
        .unwrap_or_else(|error| panic!("could not read application ID: {error}"));
    let journal_mode: String = connection
        .query_row("PRAGMA journal_mode", [], |row| row.get(0))
        .unwrap_or_else(|error| panic!("could not read journal mode: {error}"));
    assert_eq!(application_id, APPLICATION_ID);
    assert_eq!(journal_mode.to_ascii_lowercase(), "delete");
}
