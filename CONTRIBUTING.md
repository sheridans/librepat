# Contributing to LibrePAT

LibrePAT is licensed under the BSD 3-Clause licence. Contributions must be
compatible with that licence and must not contain private customer or tester
export data.

## Structure

- `librepat-core`: tester-independent job and result types.
- `librepat-import`: shared labelled-text parsing and record processing.
- `librepat-fluke`: Fluke text export rules.
- `librepat-seaward`: Seaward PrimeTest ASCII rules.
- `librepat-kewtech`: Kewtech KT74/77 ASCII rules.
- `librepat-storage`: portable `.librepat` SQLite files.
- `librepat-report`: original PDF reports and certificates.
- `librepat-app`: synchronous native desktop workflow.

See [`docs/importers.md`](docs/importers.md) before adding a tester format.
See [`docs/formats`](docs/formats) for implemented mappings.

Keep each Rust source file at or below 300 physical lines. Comments should be
reserved for public API behaviour, errors, non-obvious invariants, format
quirks, and safety constraints.

## Licence policy

Dependencies and assets may use 0BSD, BSD-2-Clause, BSD-3-Clause, MIT,
Apache-2.0, ISC, Zlib, Unicode-3.0, CC0-1.0, or BSL-1.0. `BSL-1.0` means the
Boost Software Licence. For dual-licensed crates, an approved permissive branch
must be selected explicitly.

GPL, AGPL, LGPL, MPL, EPL, CDDL, SSPL, BUSL, source-available licences, OFL,
Ubuntu Font Licence, unknown licences, and unlicensed-only material are not
accepted. Cargo checks do not cover fonts, icons, templates, or bundled
binaries; review those separately and record them in third-party notices.

The `default_fonts` feature of egui and the `html` feature of printpdf must stay
disabled.

## Review

Run the following before requesting review:

```text
cargo fmt --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo deny check
cargo audit
./scripts/check-rust-lines.sh
./scripts/check-private-fixtures.sh
cargo build --release --workspace --all-features --locked
```

Never commit private root-level `.FLK`, `.PAT`, `.pdf`, or `.librepat` files.
