//! Lossless importer for supported Fluke text exports.

use librepat_core::{ImporterMetadata, ImporterRegistry};
use librepat_import::LabelledTextImporter;

mod adapter;

pub use adapter::FlukeAdapter;

/// Shared labelled-text importer configured with Fluke format rules.
pub type FlukeImporter = LabelledTextImporter<FlukeAdapter>;

/// Metadata for the Fluke text-export adapter.
pub const IMPORTER_METADATA: ImporterMetadata = ImporterMetadata::new(
    "librepat-fluke",
    "fluke-flk-text",
    "Fluke FLK text export",
    &["flk"],
);

/// Adds this adapter's supported formats to an importer registry.
pub fn register(registry: &mut ImporterRegistry) {
    registry.register(FlukeImporter::new(FlukeAdapter));
}
