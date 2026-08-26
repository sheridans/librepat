//! Seaward PrimeTest 300/350 labelled ASCII adapter.

use librepat_core::{ImporterMetadata, ImporterRegistry};
use librepat_import::LabelledTextImporter;

mod adapter;

pub use adapter::SeawardAdapter;

/// Shared labelled-text importer configured with Seaward ASCII rules.
pub type SeawardImporter = LabelledTextImporter<SeawardAdapter>;

/// Stable identity for the Seaward PrimeTest ASCII format.
pub const IMPORTER_METADATA: ImporterMetadata = ImporterMetadata::new(
    "librepat-seaward",
    "seaward-primetest-ascii",
    "Seaward PrimeTest 300/350 ASCII",
    &["txt", "asc"],
);

/// Adds the Seaward ASCII adapter to an importer registry.
pub fn register(registry: &mut ImporterRegistry) {
    registry.register(SeawardImporter::new(SeawardAdapter));
}
