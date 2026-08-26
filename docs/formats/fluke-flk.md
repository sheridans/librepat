# Fluke FLK Text Format

This document describes the format implemented by `librepat-fluke`. It is an
implementation reference, not a vendor specification.

The supported `.FLK` input is UTF-8, line-oriented label/value text. It is not
CSV. LibrePAT accepts LF and CRLF line endings and embeds the original bytes
without rewriting them.

## Detection

`FlukeImporter` assigns confidence from file content:

- `Certain`: at least one line starts with `MODEL` and one starts with
  `TEST NUMBER`.
- `Possible`: `TEST NUMBER` exists, `MODEL` is absent, and the filename uses
  the `.flk` extension.
- `NotRecognized`: neither combination matches, or the bytes are not UTF-8.

The `.flk` extension is only a file-dialog hint.

## File Structure

A file can contain tester headers followed by appliance records:

```text
MODEL   6500
SN      SYNTHETIC-0001

TEST NUMBER      1
DATE     03-FEB-26
APP NO  A-001
TEST MODE   0  MAN
SITE  Example Site
USER    Operator
VISUAL CHECK     P
DES1  Example appliance
LOC1  Workshop

END OF DATA
```

Blank lines are ignored. `END OF DATA` finishes the current record. A new
`TEST NUMBER` also finishes the preceding record, so a missing end marker does
not discard the last usable appliance.

Header and ordinary record fields require whitespace between the field name and
value. Leading whitespace in the value is ignored.

## Field Mapping

| FLK field | LibrePAT destination |
| --- | --- |
| `MODEL` | `Job.source.tester_model` |
| `SN` | `Job.source.tester_serial` |
| `TEST NUMBER` | `Appliance.source_record_number` |
| `DATE` | `Appliance.test_date` |
| `APP NO` | `Appliance.appliance_id` |
| `TEST MODE` | `Appliance.mode.code` and `.label` |
| `SITE`, `SITE1`, `SITE2` | Initial job site name |
| `USER` | Appliance user and initial job tester name |
| `DES1`, `DES2`, `DES3` | Appliance description and raw segments |
| `LOC1`, `LOC2` | Appliance location and raw segments |
| `BOND RANGE` | Raw appliance field without a warning |
| `LIMIT` | Limit on the preceding test result |

The first non-empty joined site value seeds the editable job site name. Later
non-empty site values that differ produce an import warning. The first non-empty
`USER` value seeds the editable tester name.

The exact source remains available in `Job.source.bytes`, even for fields that
do not have a separate domain representation.

## Continued Fields

The continued fields use the spacing found in FLK text exports:

- four-character names `SITE`, `DES1`, `DES2`, `DES3`, `LOC1`, and `LOC2` use
  two spaces before the value;
- five-character names `SITE1` and `SITE2` use one space.

Description and location segments are retained separately. When joining
segments, LibrePAT assumes a 12-character tester field width. A segment shorter
than 12 characters receives a separating space; a full-width segment joins
directly to the next segment so words split at the tester boundary are repaired.
Whitespace is then normalized.

## Dates and Modes

Dates use `DD-MMM-YY` or `DD-MMM-YYYY`, with English three-letter month names.
Month matching is case-insensitive.

Two-digit years map as follows:

- `80` through `99` become 1980 through 1999;
- `00` through `79` become 2000 through 2079.

Invalid dates produce a warning and leave the appliance test date blank.

Observed mode codes are `0`, `133`, `136`, `231`, `232`, `233`, `333`, `334`,
and `336`. Other non-empty codes are preserved with an `UnsupportedMode`
warning.

## Test Results

A result line ends with a status token:

```text
TEST NAME [READING UNIT] STATUS
```

Status values are:

- `P`: pass;
- `F`: fail;
- `S`: skipped.

If the two tokens before the status are a numeric token and a unit, they become
a `Measurement`. A numeric token can contain ASCII digits, `.`, `+`, or `-`.
Decimal text is not converted to floating point. Supported comparison prefixes
are `<`, `<=`, `=`, `>=`, and `>`.

Recognized test names are:

| FLK name | `TestKind` |
| --- | --- |
| `VISUAL CHECK` | `Visual` |
| `LEAD CONTINUITY` | `LeadContinuity` |
| `EARTH` | `EarthBond` |
| names beginning `INS ` | `Insulation` |
| `PN CONTINUITY` | `PolarityContinuity` |
| `TOUCH` | `TouchCurrent` |
| `LOAD` | `Load` |
| `CURRENT` | `Current` |
| `LKGE` | `Leakage` |
| names beginning `SUBST ` | `SubstituteLeakage` |
| `PROBE PELV` | `Pelv` |

A syntactically valid result with another name becomes `TestKind::Unknown` and
retains its name, reading, status, and raw line.

All result occurrences remain stored. The appliance outcome uses the final
occurrence of each `TestKind`: any final failure makes the appliance fail; with
no failures, at least one final pass makes it pass. The importer warns and
discards records whose final results are all skipped.

## Limits

`LIMIT` applies to the immediately preceding result. It accepts an exact decimal
with an optional comparison prefix and an optional unit:

```text
LIMIT     >0.10 OHM
```

A limit normally requires a preceding measurement. PELV and unknown tests may
carry a limit without a reported measurement. An unattached or malformed limit
is preserved as a raw field and produces an `InconsistentField` warning.

## Record Acceptance and Warnings

A record with no test results is discarded. A record with tests is retained when
it has either a source record number or an appliance ID; missing identifiers
produce an `IncompleteRecord` warning.

The importer also warns about:

- duplicate source record numbers;
- duplicate non-empty appliance IDs;
- malformed dates;
- unobserved mode codes;
- inconsistent site values;
- unknown record lines;
- a missing `END OF DATA` marker.

Unknown lines inside an appliance record are stored in `raw_fields`. Unknown
lines outside a record produce a warning but have no appliance to own a raw
field. `BOND RANGE` is preserved without a warning.

## Source Provenance

Successful imports record:

- importer ID `librepat-fluke`;
- format ID `fluke-flk-text`;
- original filename and bytes;
- SHA-256 hash and import timestamp;
- tester model and serial number.

The source archive and imported electrical results become immutable when the
job is stored.

## Not Supported

`librepat-fluke` does not currently parse:

- binary `.PAT` files;
- CSV exports;
- non-UTF-8 text;
- Seaward, Kewtech, or other tester formats.

Those formats require separate adapters and their own synthetic test fixtures.
