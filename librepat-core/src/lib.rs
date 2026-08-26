//! Tester-independent jobs, appliances, immutable electrical results, and import APIs.

mod date_format;
mod dates;
mod identifier_order;
mod import;
mod model;
mod results;

pub use date_format::ReportDateFormat;
pub use dates::{BulkDateEdit, BulkEditError, DateField, apply_bulk_date_edit};
pub use identifier_order::compare_identifiers;
pub use import::{
    Detection, ImportError, ImportSource, ImportWarning, ImportWarningKind, ImporterMetadata,
    ImporterRegistry, TesterImporter,
};
pub use model::{
    Appliance, ApplianceStatus, Comparison, ContinuedText, ImportProvenance, ImportedJob, Job,
    JobMetadata, Limit, Measurement, PostalAddress, RawField, SourceArchive, TestKind, TestMode,
    TestResult, TestStatus,
};
pub use results::{latest_test_indices, latest_tests};
