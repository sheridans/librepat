mod common;

use common::{TestFile, create_store, sample_job};
use librepat_core::{ApplianceStatus, TestStatus};
use librepat_storage::{JobStore, StoreError};
use rusqlite::Connection;

#[test]
fn create_should_reject_an_appliance_without_a_final_result() {
    let source = TestFile::new("all-skipped-source");
    let mut job = create_store(&source)
        .load()
        .unwrap_or_else(|error| panic!("could not load test job: {error}"));
    for test in &mut job.appliances[0].tests {
        test.status = TestStatus::Skipped;
    }
    let destination = TestFile::new("all-skipped-destination");

    assert!(matches!(
        JobStore::create(destination.path(), &job),
        Err(StoreError::InvalidJob(message))
            if message.contains("no final pass or fail result")
    ));
    assert!(!destination.path().exists());
}

#[test]
fn create_should_store_status_derived_from_results() {
    let source = TestFile::new("wrong-status-source");
    let mut job = create_store(&source)
        .load()
        .unwrap_or_else(|error| panic!("could not load test job: {error}"));
    job.appliances[0].status = ApplianceStatus::Fail;
    let destination = TestFile::new("wrong-status-destination");

    let _store = JobStore::create(destination.path(), &job)
        .unwrap_or_else(|error| panic!("could not create test job: {error}"));
    let connection = Connection::open(destination.path())
        .unwrap_or_else(|error| panic!("could not inspect test database: {error}"));
    let stored_status: String = connection
        .query_row("SELECT overall_status FROM appliance", [], |row| row.get(0))
        .unwrap_or_else(|error| panic!("could not read stored status: {error}"));

    assert_eq!(stored_status, "pass");
}

#[test]
fn load_should_derive_status_from_test_results() {
    let file = TestFile::new("stored-status");
    let mut job = sample_job();
    let final_result = job.appliances[0].tests[0].clone();
    job.appliances[0].tests[0].status = TestStatus::Fail;
    job.appliances[0].tests.push(final_result);
    job.appliances[0].status = ApplianceStatus::Fail;
    let store = JobStore::create(file.path(), &job)
        .unwrap_or_else(|error| panic!("could not create test job: {error}"));
    drop(store);
    let connection = Connection::open(file.path())
        .unwrap_or_else(|error| panic!("could not alter cached status: {error}"));
    connection
        .execute_batch(
            "DROP TRIGGER appliance_result_no_update;
             UPDATE appliance SET overall_status = 'fail';",
        )
        .unwrap_or_else(|error| panic!("could not alter cached status: {error}"));
    drop(connection);
    let store = JobStore::open(file.path())
        .unwrap_or_else(|error| panic!("could not open test job: {error}"));
    let loaded = store
        .load()
        .unwrap_or_else(|error| panic!("could not load test job: {error}"));

    assert_eq!(loaded.appliances[0].status, ApplianceStatus::Pass);
}

#[test]
fn save_should_ignore_cached_status_when_results_are_unchanged() {
    let source = TestFile::new("save-cached-status-source");
    let mut job = create_store(&source)
        .load()
        .unwrap_or_else(|error| panic!("could not load test job: {error}"));
    job.appliances[0].status = ApplianceStatus::Fail;
    let destination = TestFile::new("save-cached-status-destination");
    let mut store = JobStore::create(destination.path(), &job)
        .unwrap_or_else(|error| panic!("could not create test job: {error}"));

    store
        .save(&job)
        .unwrap_or_else(|error| panic!("could not save test job: {error}"));
    let loaded = store
        .load()
        .unwrap_or_else(|error| panic!("could not load saved test job: {error}"));

    assert_eq!(loaded.appliances[0].status, ApplianceStatus::Pass);
}
