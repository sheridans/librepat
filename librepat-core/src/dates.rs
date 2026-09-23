use thiserror::Error;
use time::{Date, Duration};

use crate::Appliance;

/// Editable appliance date column.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DateField {
    Test,
    Retest,
}

/// Atomic date operation applied to selected appliances.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BulkDateEdit {
    Set {
        field: DateField,
        value: Option<Date>,
    },
    Shift {
        field: DateField,
        days: i64,
    },
}

/// Bulk edit validation failure. No appliance is changed on error.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum BulkEditError {
    #[error("selection contains appliance index {0}, which does not exist")]
    InvalidSelection(usize),
    #[error("shifting the date for appliance {0} would exceed the supported range")]
    DateOverflow(usize),
}

/// Validates the complete operation before mutating any appliance.
///
/// Blank dates remain blank when shifted.
///
/// # Errors
/// Returns an error for an invalid selection or any overflowing date.
pub fn apply_bulk_date_edit(
    appliances: &mut [Appliance],
    selection: &[usize],
    edit: BulkDateEdit,
) -> Result<(), BulkEditError> {
    let replacements = selection
        .iter()
        .map(|&index| {
            let appliance = appliances
                .get(index)
                .ok_or(BulkEditError::InvalidSelection(index))?;
            let current = match edit {
                BulkDateEdit::Set { value, .. } => value,
                BulkDateEdit::Shift { field, days } => {
                    let current = field_value(appliance, field);
                    match current {
                        Some(date) => Some(
                            date.checked_add(Duration::days(days))
                                .ok_or(BulkEditError::DateOverflow(index))?,
                        ),
                        None => None,
                    }
                }
            };
            Ok((index, current))
        })
        .collect::<Result<Vec<_>, BulkEditError>>()?;

    let field = match edit {
        BulkDateEdit::Set { field, .. } | BulkDateEdit::Shift { field, .. } => field,
    };
    for (index, value) in replacements {
        set_field(&mut appliances[index], field, value);
    }
    Ok(())
}

fn field_value(appliance: &Appliance, field: DateField) -> Option<Date> {
    match field {
        DateField::Test => appliance.test_date,
        DateField::Retest => appliance.retest_date,
    }
}

fn set_field(appliance: &mut Appliance, field: DateField, value: Option<Date>) {
    match field {
        DateField::Test => appliance.test_date = value,
        DateField::Retest => appliance.retest_date = value,
    }
}

#[cfg(test)]
mod tests {
    use time::Month;

    use super::*;
    use crate::{ApplianceStatus, TestMode};

    fn appliance(retest_date: Option<Date>) -> Appliance {
        Appliance {
            source_record_number: "1".into(),
            removed: false,
            appliance_id: "A-1".into(),
            description: String::new(),
            description_segments: Vec::new(),
            location: String::new(),
            location_segments: Vec::new(),
            test_date: None,
            test_time: None,
            retest_date,
            comments: String::new(),
            mode: TestMode::default(),
            user: String::new(),
            tests: Vec::new(),
            status: ApplianceStatus::Pass,
            raw_fields: Vec::new(),
        }
    }

    #[test]
    fn shift_should_leave_blank_retest_date_blank() {
        let mut appliances = vec![appliance(None)];
        let result = apply_bulk_date_edit(
            &mut appliances,
            &[0],
            BulkDateEdit::Shift {
                field: DateField::Retest,
                days: 30,
            },
        );
        assert!(result.is_ok(), "bulk edit failed: {result:?}");
        assert_eq!(appliances[0].retest_date, None);
    }

    #[test]
    fn invalid_selection_should_not_partially_update() {
        let original = Date::from_calendar_date(2026, Month::January, 1).ok();
        let mut appliances = vec![appliance(original)];
        let result = apply_bulk_date_edit(
            &mut appliances,
            &[0, 4],
            BulkDateEdit::Set {
                field: DateField::Retest,
                value: None,
            },
        );
        assert_eq!(result, Err(BulkEditError::InvalidSelection(4)));
        assert_eq!(appliances[0].retest_date, original);
    }
}
