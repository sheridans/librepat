mod common;

use common::{TestFile, create_store};
use librepat_storage::{JobStore, SCHEMA_VERSION};
use rusqlite::Connection;

#[test]
fn removed_state_should_round_trip_through_storage_api() {
    let file = TestFile::new("removed-round-trip");
    let mut store = create_store(&file);
    let mut job = store
        .load()
        .unwrap_or_else(|error| panic!("could not load test job: {error}"));
    job.appliances[0].removed = true;
    store
        .save(&job)
        .unwrap_or_else(|error| panic!("could not save removed state: {error}"));

    let loaded = store
        .load()
        .unwrap_or_else(|error| panic!("could not reload test job: {error}"));

    assert!(loaded.appliances[0].removed);
}

#[test]
fn restored_state_should_round_trip_through_storage_api() {
    let file = TestFile::new("restored-round-trip");
    let mut store = create_store(&file);
    let mut job = store
        .load()
        .unwrap_or_else(|error| panic!("could not load test job: {error}"));
    job.appliances[0].removed = true;
    store
        .save(&job)
        .unwrap_or_else(|error| panic!("could not save removed state: {error}"));
    job.appliances[0].removed = false;
    store
        .save(&job)
        .unwrap_or_else(|error| panic!("could not save restored state: {error}"));

    let loaded = store
        .load()
        .unwrap_or_else(|error| panic!("could not reload restored job: {error}"));

    assert!(!loaded.appliances[0].removed);
}

#[test]
fn direct_removed_state_update_should_preserve_appliance_row() {
    let file = TestFile::new("direct-removed-update");
    drop(create_store(&file));
    let connection = Connection::open(file.path())
        .unwrap_or_else(|error| panic!("could not open raw database: {error}"));

    connection
        .execute("UPDATE appliance SET removed = 1", [])
        .unwrap_or_else(|error| panic!("could not update removed state: {error}"));
    let removed: bool = connection
        .query_row("SELECT removed FROM appliance", [], |row| row.get(0))
        .unwrap_or_else(|error| panic!("could not read removed state: {error}"));

    assert!(removed);
}

#[test]
fn direct_appliance_delete_should_remain_blocked() {
    let file = TestFile::new("direct-appliance-delete");
    drop(create_store(&file));
    let connection = Connection::open(file.path())
        .unwrap_or_else(|error| panic!("could not open raw database: {error}"));

    let result = connection.execute("DELETE FROM appliance", []);

    assert!(
        result.is_err(),
        "direct appliance delete unexpectedly succeeded"
    );
}

#[test]
fn open_should_upgrade_schema_one_with_active_appliances() {
    let file = TestFile::new("schema-one-upgrade");
    drop(create_store(&file));
    let connection = Connection::open(file.path())
        .unwrap_or_else(|error| panic!("could not open raw database: {error}"));
    connection
        .execute_batch(
            "ALTER TABLE appliance DROP COLUMN removed;
             PRAGMA user_version = 1;",
        )
        .unwrap_or_else(|error| panic!("could not prepare schema-one job: {error}"));
    drop(connection);

    let store = JobStore::open(file.path())
        .unwrap_or_else(|error| panic!("could not migrate schema-one job: {error}"));
    let job = store
        .load()
        .unwrap_or_else(|error| panic!("could not load migrated job: {error}"));
    let removed = job.appliances[0].removed;
    drop(store);
    let connection = Connection::open(file.path())
        .unwrap_or_else(|error| panic!("could not inspect migrated job: {error}"));
    let version: i64 = connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap_or_else(|error| panic!("could not read migrated version: {error}"));

    assert_eq!((version, removed), (SCHEMA_VERSION, false));
}
