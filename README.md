# LibrePAT

![CI](https://github.com/sheridans/librepat/actions/workflows/ci.yml/badge.svg?branch=main)
![License](https://img.shields.io/github/license/sheridans/librepat)
[![Buy Me a Coffee](https://img.shields.io/badge/Buy%20Me%20a%20Coffee-support-ffdd00?logo=buymeacoffee)](https://buymeacoffee.com/sheridans)

LibrePAT is a Rust desktop application for importing portable appliance tester
exports, storing each job in a self-contained `.librepat` SQLite file, editing
job metadata, and generating PDF reports.

## Supported Imports

LibrePAT supports line-oriented labelled-text exports through interchangeable
manufacturer adapters and one shared import pipeline:

- Fluke `.FLK` UTF-8 text;
- Seaward PrimeTest 300/350 ASCII;
- Kewtech KT74/77 ASCII.

- [Implemented FLK format and field mapping](docs/formats/fluke-flk.md)
- [Implemented Seaward ASCII mapping](docs/formats/seaward-ascii.md)
- [Implemented Kewtech ASCII mapping](docs/formats/kewtech-ascii.md)
- [Importer architecture and adding another tester](docs/importers.md)

## Workspace

- `librepat-core`: tester-independent jobs, results, and importer APIs.
- `librepat-import`: shared labelled-text parsing and record processing.
- `librepat-fluke`: Fluke detection, labels, dates, modes, and test mappings.
- `librepat-seaward`: PrimeTest ASCII format rules.
- `librepat-kewtech`: KT74/77 ASCII format rules.
- `librepat-storage`: portable SQLite persistence and immutability checks.
- `librepat-report`: PDF report and certificate generation.
- `librepat-app`: native desktop UI and background tasks.

## Build

The workspace uses the Rust version declared in `Cargo.toml`.

```text
cargo build --release --workspace --all-features --locked
```

Contributor requirements and validation commands are in
[`CONTRIBUTING.md`](CONTRIBUTING.md).

## Data Safety

Do not commit customer exports, supplied reports, `.librepat` jobs, or other
identifying data. Tests and documentation use synthetic values only.

LibrePAT is licensed under the BSD 3-Clause licence. See [`LICENSE`](LICENSE).
