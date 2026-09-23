use librepat_core::{
    Appliance, ApplianceStatus, ImportProvenance, Job, RawField, SourceArchive, TestMode,
};
use rusqlite::{Connection, params};

use crate::{
    StoreError,
    codec::{appliance_status_from_text, date_from_text, time_from_text, timestamp_from_text},
    metadata::read_metadata,
    results::load_tests,
};

pub(crate) fn load_job(connection: &Connection) -> Result<Job, StoreError> {
    Ok(Job {
        metadata: read_metadata(connection, "current")?,
        original_metadata: read_metadata(connection, "original")?,
        appliances: load_appliances(connection)?,
        source: load_source(connection)?,
    })
}

fn load_source(connection: &Connection) -> Result<SourceArchive, StoreError> {
    let (importer_id, format_id, filename, bytes, hash, imported_at, tester_model, tester_serial) =
        connection.query_row(
            "SELECT importer_id, format_id, filename, bytes, sha256, imported_at,
                tester_model, tester_serial
         FROM source_archive WHERE id = 1",
            [],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Vec<u8>>(3)?,
                    row.get::<_, Vec<u8>>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                ))
            },
        )?;
    let sha256 = <[u8; 32]>::try_from(hash.as_slice())
        .map_err(|_| StoreError::Corrupt("source hash is not 32 bytes".into()))?;
    Ok(SourceArchive {
        provenance: ImportProvenance {
            importer_id,
            format_id,
        },
        filename,
        bytes,
        sha256,
        imported_at: timestamp_from_text(&imported_at)?,
        tester_model,
        tester_serial,
    })
}

fn load_appliances(connection: &Connection) -> Result<Vec<Appliance>, StoreError> {
    let mut statement = connection.prepare(
        "SELECT id, source_record_number, appliance_id, description, location,
                test_date, test_time, retest_date, comments, mode_code, mode_label,
                user_name, overall_status, removed
         FROM appliance ORDER BY ordinal",
    )?;
    let rows = statement
        .query_map([], |row| {
            Ok(ApplianceRow {
                id: row.get(0)?,
                source_record_number: row.get(1)?,
                appliance_id: row.get(2)?,
                description: row.get(3)?,
                location: row.get(4)?,
                test_date: row.get(5)?,
                test_time: row.get(6)?,
                retest_date: row.get(7)?,
                comments: row.get(8)?,
                mode_code: row.get(9)?,
                mode_label: row.get(10)?,
                user: row.get(11)?,
                status: row.get(12)?,
                removed: row.get(13)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    rows.into_iter()
        .map(|row| row.into_appliance(connection))
        .collect()
}

struct ApplianceRow {
    id: i64,
    source_record_number: String,
    appliance_id: String,
    description: String,
    location: String,
    test_date: Option<String>,
    test_time: Option<String>,
    retest_date: Option<String>,
    comments: String,
    mode_code: String,
    mode_label: String,
    user: String,
    status: String,
    removed: bool,
}

impl ApplianceRow {
    fn into_appliance(self, connection: &Connection) -> Result<Appliance, StoreError> {
        appliance_status_from_text(&self.status)?;
        let tests = load_tests(connection, self.id)?;
        let status = ApplianceStatus::from_tests(&tests).ok_or_else(|| {
            StoreError::Corrupt("stored appliance has no final pass or fail result".into())
        })?;
        Ok(Appliance {
            source_record_number: self.source_record_number,
            removed: self.removed,
            appliance_id: self.appliance_id,
            description: self.description,
            description_segments: load_segments(connection, self.id, "description")?,
            location: self.location,
            location_segments: load_segments(connection, self.id, "location")?,
            test_date: date_from_text(self.test_date)?,
            test_time: time_from_text(self.test_time)?,
            retest_date: date_from_text(self.retest_date)?,
            comments: self.comments,
            mode: TestMode {
                code: self.mode_code,
                label: self.mode_label,
            },
            user: self.user,
            tests,
            status,
            raw_fields: load_raw_fields(connection, self.id)?,
        })
    }
}

fn load_segments(
    connection: &Connection,
    appliance_id: i64,
    field: &str,
) -> Result<Vec<String>, StoreError> {
    let mut statement = connection.prepare(
        "SELECT value FROM continuation_segment
         WHERE appliance_id = ?1 AND field = ?2 ORDER BY ordinal",
    )?;
    Ok(statement
        .query_map(params![appliance_id, field], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?)
}

fn load_raw_fields(
    connection: &Connection,
    appliance_id: i64,
) -> Result<Vec<RawField>, StoreError> {
    let mut statement = connection.prepare(
        "SELECT name, value, source_line FROM raw_field
         WHERE appliance_id = ?1 ORDER BY ordinal",
    )?;
    let rows = statement
        .query_map([appliance_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    rows.into_iter()
        .map(|(name, value, source_line)| {
            Ok(RawField {
                name,
                value,
                line_number: usize::try_from(source_line)
                    .map_err(|_| StoreError::Corrupt("invalid source line number".into()))?,
            })
        })
        .collect()
}
