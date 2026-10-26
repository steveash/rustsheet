---
id: T-2709
title: "Oracle: type an entered formula longer than 255 characters, then probe SUM 255/256 and TEXTJOIN 254/255 typed"
milestone: M7
lane: harness
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`rustsheet-oracle fill -b excel-graph` types an `entered` cell of any length Excel takes (up to its 8,192-character
formula limit), and the 255-argument typed maxima are probed.

## Context
T-2682 could not probe `SUM` with 255 and 256 arguments or `TEXTJOIN` with 254 and 255 (Microsoft: 252 texts) typed: the
oracle hands each entered cell to the workbook as a JSON custom property, and a custom property holds at most 255
characters (`ValueError: entered cell Sheet1!A1 is too long for a custom property (535 > 255)`). Split a long entry
over several properties, or carry it another way, (tools/src/rustsheet_tools/spec2xlsx.py `_set_json_property`, `ENTERED_PROPERTY`). Then probe, typed: SUM 255/256,
TEXTJOIN 254/255/256, and one other 255-repeating function (MAX or CONCAT), and record each in
docs/excel-semantics/syntax.md "Minimum argument counts" (its UNVERIFIED note names this task). The engine table gives
SUM and TEXTJOIN 255 (crates/rustsheet-engine/src/functions/kinds/arg_counts.txt).

## Acceptance criteria
- [x] An entered cell longer than 255 characters fills on excel-graph (a tools test covers the split).
- [x] The counts above are recorded as accepted (sidecar) or refused (Graph 400, dated); any mismatch with arg_counts.txt
      is fixed through tools/tests/test_catalog_arity.py `MAX_OVERRIDES`, cited.
- [x] `make check` green; reviewers approve.

## Out of scope
- Stored (file) forms of the same calls.

## Outcome
spec2xlsx splits a recorded JSON property over `<name>`, `<name>.2`, ... pieces of at most 255 UTF-16 units and reads
them back joined (tools/tests/test_spec2xlsx_entered.py). Typed on excel-graph: SUM and MAX take 255 arguments and are
refused at 256; CONCAT and TEXTJOIN take 254 and are refused at 255, so both are `MAX_OVERRIDES` at 254 and
arg_counts.txt follows (syntax.md "Minimum argument counts"); one-character arguments show it is the count, not the
length. A typed 8,192-character formula fills and 8,193 is refused; a typed 256-character literal is accepted (the
literal bound is T-2739).
