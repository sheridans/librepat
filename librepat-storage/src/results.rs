use librepat_core::{Limit, Measurement, TestResult};
use rusqlite::Connection;

use crate::{
    StoreError,
    codec::{comparison_from_text, test_kind_from_text, test_status_from_text},
};

pub(crate) fn load_tests(
    connection: &Connection,
    appliance_id: i64,
) -> Result<Vec<TestResult>, StoreError> {
    let mut statement = connection.prepare(
        "SELECT t.kind, t.raw_name, t.status, t.raw_line,
                m.comparison, m.value, m.unit,
                l.comparison, l.value, l.unit, l.raw
         FROM test_result t
         LEFT JOIN measurement m ON m.test_id = t.id
         LEFT JOIN test_limit l ON l.test_id = t.id
         WHERE t.appliance_id = ?1
         ORDER BY t.ordinal",
    )?;
    let rows = statement
        .query_map([appliance_id], |row| {
            Ok(ResultRow {
                kind: row.get(0)?,
                raw_name: row.get(1)?,
                status: row.get(2)?,
                raw_line: row.get(3)?,
                measurement_comparison: row.get(4)?,
                measurement_value: row.get(5)?,
                measurement_unit: row.get(6)?,
                limit_comparison: row.get(7)?,
                limit_value: row.get(8)?,
                limit_unit: row.get(9)?,
                limit_raw: row.get(10)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    rows.into_iter().map(ResultRow::into_test).collect()
}

struct ResultRow {
    kind: String,
    raw_name: String,
    status: String,
    raw_line: String,
    measurement_comparison: Option<String>,
    measurement_value: Option<String>,
    measurement_unit: Option<String>,
    limit_comparison: Option<String>,
    limit_value: Option<String>,
    limit_unit: Option<String>,
    limit_raw: Option<String>,
}

impl ResultRow {
    fn into_test(self) -> Result<TestResult, StoreError> {
        let measurement = match (
            self.measurement_comparison,
            self.measurement_value,
            self.measurement_unit,
        ) {
            (None, None, None) => None,
            (Some(comparison), Some(value), Some(unit)) => Some(Measurement {
                comparison: comparison_from_text(&comparison)?,
                value,
                unit,
            }),
            _ => return Err(StoreError::Corrupt("incomplete measurement row".into())),
        };
        let limit = match (
            self.limit_comparison,
            self.limit_value,
            self.limit_unit,
            self.limit_raw,
        ) {
            (None, None, None, None) => None,
            (Some(comparison), Some(value), Some(unit), Some(raw)) => Some(Limit {
                comparison: comparison_from_text(&comparison)?,
                value,
                unit,
                raw,
            }),
            _ => return Err(StoreError::Corrupt("incomplete test limit row".into())),
        };
        Ok(TestResult {
            kind: test_kind_from_text(&self.kind, &self.raw_name)?,
            raw_name: self.raw_name,
            status: test_status_from_text(&self.status)?,
            measurement,
            limit,
            raw_line: self.raw_line,
        })
    }
}
