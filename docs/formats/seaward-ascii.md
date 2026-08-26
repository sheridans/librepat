# Seaward PrimeTest ASCII

`librepat-seaward` supports line-oriented ASCII exports from the
PrimeTest 300/350 family. It does not parse `.SSS` or `.GAR` files.

Detection requires `TEST NUMBER`, `TESTER`, `TEST MODE`, and either `EARTH
CURRENT` or an `IEC` result for certain confidence. `TEST NUMBER`, `TESTER`, and
`TIME` give possible confidence. The adapter rejects generic labelled files.

The adapter maps `TESTER`, `TEST NUMBER`, `DATE`, `TIME`, `APP NO`, `TEST MODE`,
`EARTH CURRENT`, `USER`, `SITE`, and repeated `TEXT` fields. The shared pipeline
maps `EARTH`, `IEC`, `INS`, `LEAD CONTINUITY`, `VISUAL CHECK`, `LOAD`, and `LKGE`
results while retaining exact readings, units, status, source lines, and bytes.

Dates accept day/month/year with `/` or `-`; times accept `HH:MM` or `HH:MM:SS`.
Input must be ASCII. The importer retains unknown record fields and emits
warnings. It copies `TEXT` values into the appliance comment and raw fields.
