# LibrePAT contributor guidance

- Keep tester-independent domain code in `librepat-core`.
- Keep import formats isolated in importer crates such as `librepat-fluke`.
- Keep SQLite persistence in `librepat-storage`, PDF primitives in
  `librepat-report`, and native UI code in `librepat-app`.
- Do not add private exports, customer data, supplied reports, or identifying
  values to tracked files, tests, snapshots, logs, or screenshots.
- Rust source files must not exceed 300 physical lines. Split by responsibility
  before reaching the limit.
- Comments explain public behaviour, errors, non-obvious invariants, format
  quirks, and safety constraints. Prefer clear names and small functions.
- New dependencies, fonts, icons, templates, and binaries require a licence
  review against `deny.toml` and `CONTRIBUTING.md`.
- Electrical outcomes and readings are immutable after import. Changes require
  storage API tests and direct SQLite trigger tests.
- Review touched Rust code with the project quality commands before handoff.
- Do not commit, tag, or publish unless the user explicitly requests it.

