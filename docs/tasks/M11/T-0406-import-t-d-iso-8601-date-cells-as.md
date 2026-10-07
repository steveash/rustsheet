---
id: T-0406
title: "Import t=d (ISO 8601) date cells as serials"
milestone: M11
lane: engine
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A cell stored as `t="d"` with ISO 8601 text (Strict OOXML writes dates this way) imports with
the value Excel gives it, instead of being dropped.

## Context
Found by T-0112. poi/SimpleStrict.xlsx "Sheet Number 2"!A4 is `<c r="A4" s="1" t="d"><v>1990-01-01</v></c>`;
the importer prints `Invalid type (d) in Sheet Number 2!A4` and leaves the cell empty
(crates/rustsheet-xlsx/src/import/worksheets.rs, the `"d"` arms near the `Invalid type (d)`
messages). See docs/excel-semantics/xlsx-storage.md "Strict OOXML uses other namespaces".
Probe first (excel-probe skill): a fixture with `t="d"` cells for a date, a date-time
(`1990-01-01T12:00:00`), a pre-1900-03-01 date and an unparsable text, each read by
`=A1`, `=ISNUMBER(A1)`; the expected serial for 1990-01-01 is presumably 32874.

## Acceptance criteria
- [ ] Oracle-backed fixture cases for the `t="d"` forms above pass.
- [ ] The eval snapshot reader decodes `t="d"` the same way (one shared parser, no copy).
- [ ] The semantics entry moves from UNVERIFIED to VERIFIED for `t="d"`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Exporting Strict OOXML.
