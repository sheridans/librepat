mod common;

use common::{TestFile, create_store};
use librepat_storage::StoreError;
use rusqlite::Connection;

#[test]
fn source_provenance_should_round_trip() {
    let file = TestFile::new("provenance-round-trip");
    let loaded = create_store(&file)
        .load()
        .unwrap_or_else(|error| panic!("could not load test job: {error}"));
    assert_eq!(
        (
            loaded.source.provenance.importer_id.as_str(),
            loaded.source.provenance.format_id.as_str(),
        ),
        ("synthetic-importer", "synthetic-format")
    );
}

#[test]
fn save_should_reject_source_provenance_update() {
    let file = TestFile::new("api-provenance-update");
    let mut store = create_store(&file);
    let mut job = store
        .load()
        .unwrap_or_else(|error| panic!("could not load test job: {error}"));
    job.source.provenance.format_id = "changed-format".into();
    assert!(matches!(
        store.save(&job),
        Err(StoreError::ImmutableResults)
    ));
}

#[test]
fn trigger_should_reject_source_provenance_update() {
    let file = TestFile::new("trigger-provenance-update");
    drop(create_store(&file));
    let connection = Connection::open(file.path())
        .unwrap_or_else(|error| panic!("could not open raw database: {error}"));
    let result = connection.execute(
        "UPDATE source_archive SET importer_id = 'changed-importer'",
        [],
    );
    assert!(
        result.is_err(),
        "direct provenance update unexpectedly succeeded"
    );
}
