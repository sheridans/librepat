use librepat_core::{Appliance, ApplianceStatus, Job, TestResult};
use rusqlite::{Connection, params};

use crate::{
    StoreError,
    codec::{
        appliance_status_to_text, comparison_to_text, date_to_text, test_kind_to_text,
        test_status_to_text, time_to_text, timestamp_to_text,
    },
    metadata::write_metadata,
};

pub(crate) fn insert_job(connection: &Connection, job: &Job) -> Result<(), StoreError> {
    connection.execute(
        "INSERT INTO source_archive
         (id, importer_id, format_id, filename, bytes, sha256, imported_at,
          tester_model, tester_serial)
         VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            job.source.provenance.importer_id,
            job.source.provenance.format_id,
            job.source.filename,
            job.source.bytes,
            job.source.sha256.as_slice(),
            timestamp_to_text(job.source.imported_at)?,
            job.source.tester_model,
            job.source.tester_serial,
        ],
    )?;
    write_metadata(connection, "original", &job.original_metadata)?;
    write_metadata(connection, "current", &job.metadata)?;
    for (ordinal, appliance) in job.appliances.iter().enumerate() {
        insert_appliance(connection, ordinal, appliance)?;
    }
    Ok(())
}

pub(crate) fn validate_job(job: &Job) -> Result<(), StoreError> {
    for appliance in &job.appliances {
        derived_status(appliance)?;
    }
    Ok(())
}

fn insert_appliance(
    connection: &Connection,
    ordinal: usize,
    appliance: &Appliance,
) -> Result<(), StoreError> {
    let status = derived_status(appliance)?;
    let test_date = date_to_text(appliance.test_date);
    let test_time = time_to_text(appliance.test_time)?;
    let retest_date = date_to_text(appliance.retest_date);
    connection.execute(
        "INSERT INTO appliance (
            ordinal, source_record_number, appliance_id_original, appliance_id,
            description_original, description, location_original, location,
            test_date_original, test_date, test_time, retest_date_original, retest_date,
            comments, mode_code, mode_label, user_name, overall_status
         ) VALUES (
            ?1, ?2, ?3, ?3, ?4, ?4, ?5, ?5, ?6, ?6, ?7, ?8, ?8, ?9, ?10, ?11, ?12, ?13
         )",
        params![
            to_i64(ordinal)?,
            appliance.source_record_number,
            appliance.appliance_id,
            appliance.description,
            appliance.location,
            test_date,
            test_time,
            retest_date,
            appliance.comments,
            appliance.mode.code,
            appliance.mode.label,
            appliance.user,
            appliance_status_to_text(status),
        ],
    )?;
    let appliance_row_id = connection.last_insert_rowid();
    insert_segments(
        connection,
        appliance_row_id,
        "description",
        &appliance.description_segments,
    )?;
    insert_segments(
        connection,
        appliance_row_id,
        "location",
        &appliance.location_segments,
    )?;
    for (test_ordinal, test) in appliance.tests.iter().enumerate() {
        insert_test(connection, appliance_row_id, test_ordinal, test)?;
    }
    for (raw_ordinal, field) in appliance.raw_fields.iter().enumerate() {
        connection.execute(
            "INSERT INTO raw_field
             (appliance_id, ordinal, name, value, source_line) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                appliance_row_id,
                to_i64(raw_ordinal)?,
                field.name,
                field.value,
                to_i64(field.line_number)?,
            ],
        )?;
    }
    Ok(())
}

fn derived_status(appliance: &Appliance) -> Result<ApplianceStatus, StoreError> {
    ApplianceStatus::from_tests(&appliance.tests).ok_or_else(|| {
        StoreError::InvalidJob(format!(
            "appliance `{}` has no final pass or fail result",
            appliance.appliance_id
        ))
    })
}

fn insert_segments(
    connection: &Connection,
    appliance_id: i64,
    field: &str,
    segments: &[String],
) -> Result<(), StoreError> {
    for (ordinal, value) in segments.iter().enumerate() {
        connection.execute(
            "INSERT INTO continuation_segment
             (appliance_id, field, ordinal, value) VALUES (?1, ?2, ?3, ?4)",
            params![appliance_id, field, to_i64(ordinal)?, value],
        )?;
    }
    Ok(())
}

fn insert_test(
    connection: &Connection,
    appliance_id: i64,
    ordinal: usize,
    test: &TestResult,
) -> Result<(), StoreError> {
    connection.execute(
        "INSERT INTO test_result
         (appliance_id, ordinal, kind, raw_name, status, raw_line)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            appliance_id,
            to_i64(ordinal)?,
            test_kind_to_text(&test.kind),
            test.raw_name,
            test_status_to_text(test.status),
            test.raw_line,
        ],
    )?;
    let test_id = connection.last_insert_rowid();
    if let Some(measurement) = &test.measurement {
        connection.execute(
            "INSERT INTO measurement (test_id, comparison, value, unit)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                test_id,
                comparison_to_text(measurement.comparison),
                measurement.value,
                measurement.unit,
            ],
        )?;
    }
    if let Some(limit) = &test.limit {
        connection.execute(
            "INSERT INTO test_limit (test_id, comparison, value, unit, raw)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                test_id,
                comparison_to_text(limit.comparison),
                limit.value,
                limit.unit,
                limit.raw,
            ],
        )?;
    }
    Ok(())
}

fn to_i64(value: usize) -> Result<i64, StoreError> {
    i64::try_from(value)
        .map_err(|_| StoreError::Corrupt("job contains too many ordered values".into()))
}
