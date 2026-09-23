use rusqlite::{Connection, TransactionBehavior};

use crate::{
    StoreError,
    store::{APPLICATION_ID, SCHEMA_VERSION},
};

pub(crate) fn create_tables(connection: &Connection) -> Result<(), StoreError> {
    connection.execute_batch(
        "
        CREATE TABLE source_archive (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            importer_id TEXT NOT NULL,
            format_id TEXT NOT NULL,
            filename TEXT NOT NULL,
            bytes BLOB NOT NULL,
            sha256 BLOB NOT NULL CHECK (length(sha256) = 32),
            imported_at TEXT NOT NULL,
            tester_model TEXT NOT NULL,
            tester_serial TEXT NOT NULL
        );
        CREATE TABLE job_metadata (
            snapshot TEXT NOT NULL CHECK (snapshot IN ('original', 'current')),
            field TEXT NOT NULL,
            value TEXT NOT NULL,
            PRIMARY KEY (snapshot, field)
        );
        CREATE TABLE appliance (
            id INTEGER PRIMARY KEY,
            ordinal INTEGER NOT NULL UNIQUE,
            source_record_number TEXT NOT NULL,
            appliance_id_original TEXT NOT NULL,
            appliance_id TEXT NOT NULL,
            description_original TEXT NOT NULL,
            description TEXT NOT NULL,
            location_original TEXT NOT NULL,
            location TEXT NOT NULL,
            test_date_original TEXT,
            test_date TEXT,
            test_time TEXT,
            retest_date_original TEXT,
            retest_date TEXT,
            comments TEXT NOT NULL,
            mode_code TEXT NOT NULL,
            mode_label TEXT NOT NULL,
            user_name TEXT NOT NULL,
            overall_status TEXT NOT NULL,
            removed INTEGER NOT NULL DEFAULT 0 CHECK (removed IN (0, 1))
        );
        CREATE TABLE test_result (
            id INTEGER PRIMARY KEY,
            appliance_id INTEGER NOT NULL REFERENCES appliance(id) ON DELETE CASCADE,
            ordinal INTEGER NOT NULL,
            kind TEXT NOT NULL,
            raw_name TEXT NOT NULL,
            status TEXT NOT NULL,
            raw_line TEXT NOT NULL,
            UNIQUE (appliance_id, ordinal)
        );
        CREATE TABLE measurement (
            test_id INTEGER PRIMARY KEY REFERENCES test_result(id) ON DELETE CASCADE,
            comparison TEXT NOT NULL,
            value TEXT NOT NULL,
            unit TEXT NOT NULL
        );
        CREATE TABLE test_limit (
            test_id INTEGER PRIMARY KEY REFERENCES test_result(id) ON DELETE CASCADE,
            comparison TEXT NOT NULL,
            value TEXT NOT NULL,
            unit TEXT NOT NULL,
            raw TEXT NOT NULL
        );
        CREATE TABLE raw_field (
            appliance_id INTEGER NOT NULL REFERENCES appliance(id) ON DELETE CASCADE,
            ordinal INTEGER NOT NULL,
            name TEXT NOT NULL,
            value TEXT NOT NULL,
            source_line INTEGER NOT NULL,
            PRIMARY KEY (appliance_id, ordinal)
        );
        CREATE TABLE continuation_segment (
            appliance_id INTEGER NOT NULL REFERENCES appliance(id) ON DELETE CASCADE,
            field TEXT NOT NULL CHECK (field IN ('description', 'location')),
            ordinal INTEGER NOT NULL,
            value TEXT NOT NULL,
            PRIMARY KEY (appliance_id, field, ordinal)
        );
        ",
    )?;
    Ok(())
}

pub(crate) fn validate_identity(connection: &Connection) -> Result<i64, StoreError> {
    let application_id: i64 =
        connection.query_row("PRAGMA application_id", [], |row| row.get(0))?;
    if application_id != APPLICATION_ID {
        return Err(StoreError::InvalidApplicationId);
    }
    let version: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if version > SCHEMA_VERSION {
        return Err(StoreError::UnsupportedSchema(version));
    }
    if !(1..=SCHEMA_VERSION).contains(&version) {
        return Err(StoreError::InvalidSchema(version));
    }
    Ok(version)
}

pub(crate) fn migrate(connection: &mut Connection, version: i64) -> Result<(), StoreError> {
    if version == SCHEMA_VERSION {
        return Ok(());
    }
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if version == 1 {
        transaction.execute_batch(
            "ALTER TABLE appliance ADD COLUMN removed INTEGER NOT NULL DEFAULT 0
             CHECK (removed IN (0, 1));",
        )?;
    } else {
        return Err(StoreError::InvalidSchema(version));
    }
    transaction.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    transaction.commit()?;
    Ok(())
}

pub(crate) fn create_immutability_triggers(connection: &Connection) -> Result<(), StoreError> {
    connection.execute_batch(
        "
        CREATE TRIGGER source_archive_no_insert BEFORE INSERT ON source_archive
        BEGIN SELECT RAISE(ABORT, 'imported source is immutable'); END;
        CREATE TRIGGER source_archive_no_update BEFORE UPDATE ON source_archive
        BEGIN SELECT RAISE(ABORT, 'imported source is immutable'); END;
        CREATE TRIGGER source_archive_no_delete BEFORE DELETE ON source_archive
        BEGIN SELECT RAISE(ABORT, 'imported source is immutable'); END;

        CREATE TRIGGER original_metadata_no_insert BEFORE INSERT ON job_metadata
        WHEN NEW.snapshot = 'original'
        BEGIN SELECT RAISE(ABORT, 'original metadata is immutable'); END;
        CREATE TRIGGER original_metadata_no_update BEFORE UPDATE ON job_metadata
        WHEN OLD.snapshot = 'original'
        BEGIN SELECT RAISE(ABORT, 'original metadata is immutable'); END;
        CREATE TRIGGER original_metadata_no_delete BEFORE DELETE ON job_metadata
        WHEN OLD.snapshot = 'original'
        BEGIN SELECT RAISE(ABORT, 'original metadata is immutable'); END;

        CREATE TRIGGER appliance_no_insert BEFORE INSERT ON appliance
        BEGIN SELECT RAISE(ABORT, 'imported appliances are immutable'); END;
        CREATE TRIGGER appliance_no_delete BEFORE DELETE ON appliance
        BEGIN SELECT RAISE(ABORT, 'imported appliances are immutable'); END;
        CREATE TRIGGER test_result_no_insert BEFORE INSERT ON test_result
        BEGIN SELECT RAISE(ABORT, 'electrical results are immutable'); END;
        CREATE TRIGGER test_result_no_update BEFORE UPDATE ON test_result
        BEGIN SELECT RAISE(ABORT, 'electrical results are immutable'); END;
        CREATE TRIGGER test_result_no_delete BEFORE DELETE ON test_result
        BEGIN SELECT RAISE(ABORT, 'electrical results are immutable'); END;
        ",
    )?;
    create_appliance_result_trigger(connection)?;
    create_child_triggers(connection, "measurement")?;
    create_child_triggers(connection, "test_limit")?;
    create_child_triggers(connection, "raw_field")?;
    create_child_triggers(connection, "continuation_segment")?;
    Ok(())
}

pub(crate) fn create_appliance_result_trigger(connection: &Connection) -> Result<(), StoreError> {
    connection.execute_batch(
        "CREATE TRIGGER appliance_result_no_update BEFORE UPDATE ON appliance
         WHEN OLD.ordinal IS NOT NEW.ordinal
           OR OLD.source_record_number IS NOT NEW.source_record_number
           OR OLD.appliance_id_original IS NOT NEW.appliance_id_original
           OR OLD.description_original IS NOT NEW.description_original
           OR OLD.location_original IS NOT NEW.location_original
           OR OLD.test_date_original IS NOT NEW.test_date_original
           OR OLD.test_time IS NOT NEW.test_time
           OR OLD.retest_date_original IS NOT NEW.retest_date_original
           OR OLD.comments IS NOT NEW.comments
           OR OLD.mode_code IS NOT NEW.mode_code
           OR OLD.mode_label IS NOT NEW.mode_label
           OR OLD.user_name IS NOT NEW.user_name
           OR OLD.overall_status IS NOT NEW.overall_status
         BEGIN SELECT RAISE(ABORT, 'electrical results are immutable'); END;",
    )?;
    Ok(())
}

fn create_child_triggers(connection: &Connection, table: &str) -> Result<(), StoreError> {
    connection.execute_batch(&format!(
        "
        CREATE TRIGGER {table}_no_insert BEFORE INSERT ON {table}
        BEGIN SELECT RAISE(ABORT, 'electrical results are immutable'); END;
        CREATE TRIGGER {table}_no_update BEFORE UPDATE ON {table}
        BEGIN SELECT RAISE(ABORT, 'electrical results are immutable'); END;
        CREATE TRIGGER {table}_no_delete BEFORE DELETE ON {table}
        BEGIN SELECT RAISE(ABORT, 'electrical results are immutable'); END;
        "
    ))?;
    Ok(())
}
