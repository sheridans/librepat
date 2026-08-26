# Importer Architecture

LibrePAT keeps each tester format in its own adapter crate. The application
does not choose or call a concrete importer. It passes the selected file to
`ImporterRegistry`, which detects the format and calls the best matching
registered adapter.

Current flow:

```text
file dialog -> ImporterRegistry -> format adapter -> shared pipeline -> Job -> librepat-storage
```

The registry also records the selected importer and format IDs in the embedded
source archive. File extensions only control file-dialog filters; content
detection decides which importer runs.

## Core API

Importers implement the tester-independent API from `librepat-core`:

```rust
pub trait TesterImporter: Send + Sync {
    fn metadata(&self) -> ImporterMetadata;
    fn detect(&self, source: &ImportSource<'_>) -> Detection;
    fn import(&self, source: &ImportSource<'_>) -> Result<ImportedJob, ImportError>;
}
```

`ImporterMetadata` contains:

- `importer_id`: stable adapter identity, such as `librepat-fluke`.
- `format_id`: stable source-format identity, such as `fluke-flk-text`.
- `display_name`: label used in the import file dialog.
- `extensions`: lowercase filename extensions without leading dots.

Do not change an ID after release. Stored jobs use both IDs as immutable source
provenance.

## Adding an Adapter

1. Create a workspace crate named for the tester family or format.
2. Depend on `librepat-core`, not the app or storage crates.
3. Define one importer type and `ImporterMetadata` constant per source format.
4. Implement `LabelledTextAdapter` when the format uses the shared labelled-text grammar.
5. Export a `register(&mut ImporterRegistry)` function from the adapter crate.
6. Add that registration call to `librepat-app/src/importers.rs`.
7. Add synthetic adapter tests and a registry-selection test.
8. Run the contributor checks in `CONTRIBUTING.md`.

Registration should keep the concrete adapter type inside its crate:

```rust
pub const IMPORTER_METADATA: ImporterMetadata = ImporterMetadata::new(
    "librepat-example",
    "example-text-v1",
    "Example tester text export",
    &["example"],
);

pub type ExampleImporter = LabelledTextImporter<ExampleAdapter>;

pub fn register(registry: &mut ImporterRegistry) {
    registry.register(ExampleImporter::new(ExampleAdapter));
}
```

The app should call only the adapter's `register` function. It must not import
or instantiate `ExampleImporter`.

## Detection

Detection must inspect source content. Do not accept a file solely because its
extension matches.

Use the confidence levels consistently:

- `NotRecognized`: required format markers are absent or the encoding is unusable.
- `Possible`: enough structure exists to try the importer, but identifying
  metadata is missing.
- `Certain`: identifying metadata and record structure both match.

When two adapters return the same highest confidence, the registry returns an
explicit ambiguity error. Formats must use manufacturer-specific marker
combinations rather than treating `TEST NUMBER` alone as sufficient.

## Import Requirements

The shared labelled-text pipeline:

- preserve the original filename and bytes exactly;
- calculate the source SHA-256 hash;
- record the import time and selected adapter's provenance;
- map tester-independent fields into `librepat-core` types;
- preserve unknown tester data in raw fields where a record can own it;
- preserve exact decimal text and comparison markers;
- return warnings for recoverable problems;
- return `ImportError` only when the source cannot produce a usable job.

Each manufacturer adapter supplies only content detection, label/alias mappings,
separator and continuation rules, encoding, date/time parsing, status tokens,
unit normalization, mode validation, test-name mappings, end markers, and
format-specific quirks.

The registry overwrites source provenance with the selected adapter metadata
before returning the imported job. Adapters should still set their provenance
because adapter tests and command-line examples may call them directly.

## Boundaries

- Keep the reusable labelled-text state machine in `librepat-import`.
- Keep labels, detection, encoding, date rules, test mappings, and quirks in the adapter crate.
- Keep shared job and result types in `librepat-core`.
- Keep the current SQLite schema and persistence code in `librepat-storage`.
- Keep file dialogs, messages, and background work in `librepat-app`.
- Do not add a tester name to the UI outside adapter metadata.

## Tests

Use synthetic, anonymous fixtures only. Adapter coverage should include:

- positive and negative detection;
- supported line endings and encodings;
- every recognized field and result type;
- malformed values and recoverable warnings;
- unknown-field preservation;
- exact source-byte preservation;
- importer and format provenance;
- selection against at least one other registered importer.

Changes to imported electrical data or source immutability also require storage
API tests and direct SQLite trigger tests.

See [`formats`](formats) for implemented mappings.
