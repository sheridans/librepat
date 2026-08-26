//! Kewtech KT74/77 labelled ASCII adapter.

use librepat_core::{ImporterMetadata, ImporterRegistry};
use librepat_import::LabelledTextImporter;

mod adapter;

pub use adapter::KewtechAdapter;

/// Shared labelled-text importer configured with Kewtech ASCII rules.
pub type KewtechImporter = LabelledTextImporter<KewtechAdapter>;

/// Stable identity for the Kewtech KT74/77 ASCII format.
pub const IMPORTER_METADATA: ImporterMetadata = ImporterMetadata::new(
    "librepat-kewtech",
    "kewtech-kt74-77-ascii",
    "Kewtech KT74/77 ASCII",
    &["txt", "asc"],
);

/// Adds the Kewtech ASCII adapter to an importer registry.
pub fn register(registry: &mut ImporterRegistry) {
    registry.register(KewtechImporter::new(KewtechAdapter));
}
