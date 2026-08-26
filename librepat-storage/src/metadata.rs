use std::collections::HashMap;

use librepat_core::{JobMetadata, PostalAddress, ReportDateFormat};
use rusqlite::{Connection, params};

use crate::{StoreError, codec::date_from_text};

pub(crate) fn write_metadata(
    connection: &Connection,
    snapshot: &str,
    metadata: &JobMetadata,
) -> Result<(), StoreError> {
    let mut statement = connection
        .prepare("INSERT INTO job_metadata (snapshot, field, value) VALUES (?1, ?2, ?3)")?;
    for (field, value) in metadata_pairs(metadata) {
        statement.execute(params![snapshot, field, value])?;
    }
    Ok(())
}

pub(crate) fn read_metadata(
    connection: &Connection,
    snapshot: &str,
) -> Result<JobMetadata, StoreError> {
    let mut statement =
        connection.prepare("SELECT field, value FROM job_metadata WHERE snapshot = ?1")?;
    let entries = statement
        .query_map([snapshot], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<Result<HashMap<_, _>, _>>()?;
    if entries.is_empty() {
        return Err(StoreError::Corrupt(format!(
            "missing `{snapshot}` metadata snapshot"
        )));
    }
    metadata_from_entries(entries)
}

fn metadata_pairs(metadata: &JobMetadata) -> Vec<(&'static str, String)> {
    let mut pairs = vec![
        ("customer_name", metadata.customer_name.clone()),
        ("site_name", metadata.site_name.clone()),
        ("contractor_name", metadata.contractor_name.clone()),
        (
            "contractor_telephone",
            metadata.contractor_telephone.clone(),
        ),
        ("contractor_email", metadata.contractor_email.clone()),
        ("tester_name", metadata.tester_name.clone()),
        ("certificate_number", metadata.certificate_number.clone()),
        ("order_reference", metadata.order_reference.clone()),
        (
            "valid_until",
            metadata
                .valid_until
                .map(|date| date.to_string())
                .unwrap_or_default(),
        ),
        (
            "report_date_format",
            metadata.report_date_format.storage_value().to_owned(),
        ),
        ("report_notes", metadata.report_notes.clone()),
    ];
    push_address(&mut pairs, "customer", &metadata.customer_address);
    push_address(&mut pairs, "site", &metadata.site_address);
    push_address(&mut pairs, "contractor", &metadata.contractor_address);
    pairs
}

fn push_address(pairs: &mut Vec<(&'static str, String)>, prefix: &str, address: &PostalAddress) {
    let fields = match prefix {
        "customer" => [
            "customer_address_1",
            "customer_address_2",
            "customer_town",
            "customer_county",
            "customer_postcode",
        ],
        "site" => [
            "site_address_1",
            "site_address_2",
            "site_town",
            "site_county",
            "site_postcode",
        ],
        _ => [
            "contractor_address_1",
            "contractor_address_2",
            "contractor_town",
            "contractor_county",
            "contractor_postcode",
        ],
    };
    for (field, value) in fields.into_iter().zip([
        &address.line_1,
        &address.line_2,
        &address.town,
        &address.county,
        &address.postcode,
    ]) {
        pairs.push((field, value.clone()));
    }
}

fn metadata_from_entries(mut entries: HashMap<String, String>) -> Result<JobMetadata, StoreError> {
    let valid_until = match take(&mut entries, "valid_until").as_str() {
        "" => None,
        value => date_from_text(Some(value.to_owned()))?,
    };
    let report_date_format = match take(&mut entries, "report_date_format").as_str() {
        "" => ReportDateFormat::default(),
        value => ReportDateFormat::from_storage_value(value).ok_or_else(|| {
            StoreError::Corrupt(format!("unsupported report date format `{value}`"))
        })?,
    };
    Ok(JobMetadata {
        customer_name: take(&mut entries, "customer_name"),
        customer_address: take_address(&mut entries, "customer"),
        site_name: take(&mut entries, "site_name"),
        site_address: take_address(&mut entries, "site"),
        contractor_name: take(&mut entries, "contractor_name"),
        contractor_address: take_address(&mut entries, "contractor"),
        contractor_telephone: take(&mut entries, "contractor_telephone"),
        contractor_email: take(&mut entries, "contractor_email"),
        tester_name: take(&mut entries, "tester_name"),
        certificate_number: take(&mut entries, "certificate_number"),
        order_reference: take(&mut entries, "order_reference"),
        valid_until,
        report_date_format,
        report_notes: take(&mut entries, "report_notes"),
    })
}

fn take_address(entries: &mut HashMap<String, String>, prefix: &str) -> PostalAddress {
    PostalAddress {
        line_1: take(entries, &format!("{prefix}_address_1")),
        line_2: take(entries, &format!("{prefix}_address_2")),
        town: take(entries, &format!("{prefix}_town")),
        county: take(entries, &format!("{prefix}_county")),
        postcode: take(entries, &format!("{prefix}_postcode")),
    }
}

fn take(entries: &mut HashMap<String, String>, field: &str) -> String {
    entries.remove(field).unwrap_or_default()
}
