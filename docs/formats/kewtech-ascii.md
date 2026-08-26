# Kewtech KT74/77 ASCII

`librepat-kewtech` supports line-oriented KT74/77 ASCII exports.
Detection requires `TEST NUMBER`, `DESCRIPTION`, and `LOCN`; a `VISUAL CHECK`
result raises confidence from possible to certain. `TEST NUMBER` alone is not a
format signature.

The adapter maps `TEST NUMBER`, `DATE`, `APP NO`, `DESCRIPTION`, `LOCN`, `SITE`,
and repeated `TEXT` fields. The shared pipeline maps `VISUAL CHECK`, `EARTH`,
`INS`, `LOAD`, `LKGE`, and `LEAD CONTINUITY` results while retaining exact
readings, units, status, source lines, and bytes.

Dates accept day/month/year with `/` or `-`. Input must be ASCII. The importer
retains unknown record fields and emits warnings. It copies `TEXT` values into
the appliance comment and raw fields.
