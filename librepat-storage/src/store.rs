use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use librepat_core::{Appliance, Job};
use rusqlite::{Connection, OpenFlags, TransactionBehavior, params};
use sha2::{Digest, Sha256};

use crate::{
    StoreError,
    codec::date_to_text,
    insert::{insert_job, validate_job},
    metadata::write_metadata,
    read::load_job,
    schema::{create_immutability_triggers, create_tables},
};

/// SQLite application ID encoding the ASCII bytes `LPAT`.
pub const APPLICATION_ID: i64 = 0x4C50_4154;
/// Current `.librepat` schema version.
pub const SCHEMA_VERSION: i64 = 1;
static NEXT_REPLACEMENT_FILE: AtomicU64 = AtomicU64::new(1);

/// Open portable job and its transactional persistence API.
pub struct JobStore {
    connection: Connection,
    path: PathBuf,
}

impl JobStore {
    /// Creates a new portable job without replacing an existing file.
    ///
    /// # Errors
    /// Returns an error if an appliance lacks a final pass/fail result, the destination exists,
    /// or the database cannot be written.
    pub fn create(path: impl AsRef<Path>, job: &Job) -> Result<Self, StoreError> {
        let path = path.as_ref();
        if path.exists() {
            return Err(StoreError::DestinationExists(path.to_owned()));
        }
        validate_job(job)?;
        let mut connection = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )?;
        configure(&connection)?;
        {
            let transaction =
                connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            create_tables(&transaction)?;
            insert_job(&transaction, job)?;
            transaction.pragma_update(None, "application_id", APPLICATION_ID)?;
            transaction.pragma_update(None, "user_version", SCHEMA_VERSION)?;
            create_immutability_triggers(&transaction)?;
            transaction.commit()?;
        }
        Ok(Self {
            connection,
            path: path.to_owned(),
        })
    }

    /// Replaces a job after creating and validating its successor beside the original file.
    ///
    /// # Errors
    /// Returns an error if the replacement cannot be created, swapped, or validated. The original
    /// file is restored when a swap fails.
    pub fn replace(path: impl AsRef<Path>, job: &Job) -> Result<Self, StoreError> {
        let path = path.as_ref();
        if !path.exists() {
            return Self::create(path, job);
        }

        let temporary = unused_sibling(path, "new");
        let backup = unused_sibling(path, "old");
        let replacement = match Self::create(&temporary, job) {
            Ok(store) => store,
            Err(error) => {
                let _cleanup = fs::remove_file(&temporary);
                return Err(error);
            }
        };
        if let Err(error) = replacement.load() {
            drop(replacement);
            let _cleanup = fs::remove_file(&temporary);
            return Err(error);
        }
        drop(replacement);

        if let Err(error) = fs::rename(path, &backup) {
            let _cleanup = fs::remove_file(&temporary);
            return Err(error.into());
        }
        if let Err(error) = fs::rename(&temporary, path) {
            restore_original(path, &backup, &temporary)?;
            return Err(error.into());
        }

        let replacement = match Self::open(path) {
            Ok(store) => store,
            Err(error) => {
                restore_original(path, &backup, &temporary)?;
                return Err(error);
            }
        };
        if let Err(error) = fs::remove_file(&backup) {
            drop(replacement);
            restore_original(path, &backup, &temporary)?;
            return Err(error.into());
        }
        Ok(replacement)
    }

    /// Opens and validates an existing portable job.
    ///
    /// `JobStore::open` rejects non-current schemas before changing database settings.
    ///
    /// # Errors
    /// Returns an error for an invalid identity, unsupported schema, corrupt source, or I/O failure.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let path = path.as_ref();
        let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_WRITE)?;
        validate_identity(&connection)?;
        configure(&connection)?;
        let store = Self {
            connection,
            path: path.to_owned(),
        };
        store.load()?;
        Ok(store)
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Loads current editable metadata and immutable imported results.
    ///
    /// # Errors
    /// Returns an error if stored rows are corrupt or the source hash no longer matches.
    pub fn load(&self) -> Result<Job, StoreError> {
        let job = load_job(&self.connection)?;
        validate_source_hash(&job)?;
        Ok(job)
    }

    /// Saves all editable metadata in one transaction.
    ///
    /// The save is rejected before mutation if imported source or electrical results changed.
    ///
    /// # Errors
    /// Returns an error for immutable changes, row mismatches, or SQLite failures.
    pub fn save(&mut self, job: &Job) -> Result<(), StoreError> {
        let persisted = self.load()?;
        if !immutable_content_matches(&persisted, job) {
            return Err(StoreError::ImmutableResults);
        }
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        transaction.execute("DELETE FROM job_metadata WHERE snapshot = 'current'", [])?;
        write_metadata(&transaction, "current", &job.metadata)?;
        update_appliances(&transaction, &job.appliances)?;
        transaction.commit()?;
        Ok(())
    }
}

fn unused_sibling(path: &Path, label: &str) -> PathBuf {
    loop {
        let sequence = NEXT_REPLACEMENT_FILE.fetch_add(1, Ordering::Relaxed);
        let mut filename = path
            .file_name()
            .map_or_else(|| OsString::from("job.librepat"), OsString::from);
        filename.push(format!(".{label}-{}-{sequence}", std::process::id()));
        let candidate = path.with_file_name(filename);
        if !candidate.exists() {
            return candidate;
        }
    }
}

fn restore_original(path: &Path, backup: &Path, temporary: &Path) -> Result<(), StoreError> {
    if path.exists() {
        fs::rename(path, temporary).map_err(|error| {
            StoreError::ReplacementRecovery(format!("could not move replacement aside: {error}"))
        })?;
    }
    fs::rename(backup, path).map_err(|error| {
        StoreError::ReplacementRecovery(format!("backup remains at {}: {error}", backup.display()))
    })?;
    if temporary.exists() {
        fs::remove_file(temporary).map_err(|error| {
            StoreError::ReplacementRecovery(format!(
                "original restored but temporary file remains at {}: {error}",
                temporary.display()
            ))
        })?;
    }
    Ok(())
}

fn configure(connection: &Connection) -> Result<(), StoreError> {
    connection.busy_timeout(Duration::from_secs(5))?;
    connection.execute_batch(
        "PRAGMA foreign_keys = ON;
         PRAGMA journal_mode = DELETE;
         PRAGMA synchronous = FULL;",
    )?;
    Ok(())
}

fn validate_identity(connection: &Connection) -> Result<(), StoreError> {
    let application_id: i64 =
        connection.query_row("PRAGMA application_id", [], |row| row.get(0))?;
    if application_id != APPLICATION_ID {
        return Err(StoreError::InvalidApplicationId);
    }
    let schema_version: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if schema_version > SCHEMA_VERSION {
        return Err(StoreError::UnsupportedSchema(schema_version));
    }
    if schema_version != SCHEMA_VERSION {
        return Err(StoreError::InvalidSchema(schema_version));
    }
    Ok(())
}

fn validate_source_hash(job: &Job) -> Result<(), StoreError> {
    let actual: [u8; 32] = Sha256::digest(&job.source.bytes).into();
    if actual != job.source.sha256 {
        return Err(StoreError::SourceHashMismatch);
    }
    Ok(())
}

fn update_appliances(connection: &Connection, appliances: &[Appliance]) -> Result<(), StoreError> {
    let mut id_statement = connection.prepare("SELECT id FROM appliance ORDER BY ordinal")?;
    let ids = id_statement
        .query_map([], |row| row.get::<_, i64>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    drop(id_statement);
    if ids.len() != appliances.len() {
        return Err(StoreError::ImmutableResults);
    }
    for (row_id, appliance) in ids.into_iter().zip(appliances) {
        let changed = connection.execute(
            "UPDATE appliance SET
                appliance_id = ?1, description = ?2, location = ?3,
                test_date = ?4, retest_date = ?5
             WHERE id = ?6",
            params![
                appliance.appliance_id,
                appliance.description,
                appliance.location,
                date_to_text(appliance.test_date),
                date_to_text(appliance.retest_date),
                row_id,
            ],
        )?;
        if changed != 1 {
            return Err(StoreError::Corrupt(
                "appliance row disappeared during save".into(),
            ));
        }
    }
    Ok(())
}

fn immutable_content_matches(persisted: &Job, candidate: &Job) -> bool {
    persisted.source == candidate.source
        && persisted.original_metadata == candidate.original_metadata
        && persisted.appliances.len() == candidate.appliances.len()
        && persisted
            .appliances
            .iter()
            .zip(&candidate.appliances)
            .all(|(left, right)| immutable_appliance_matches(left, right))
}

fn immutable_appliance_matches(left: &Appliance, right: &Appliance) -> bool {
    left.source_record_number == right.source_record_number
        && left.description_segments == right.description_segments
        && left.location_segments == right.location_segments
        && left.test_time == right.test_time
        && left.comments == right.comments
        && left.mode == right.mode
        && left.user == right.user
        && left.tests == right.tests
        && left.raw_fields == right.raw_fields
}
