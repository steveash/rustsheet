---
id: T-2113
title: "Oracle canary refuses only sheet-count calls: allow SHEETS with a reference and INFO with a literal type other than numfile"
milestone: M5
lane: harness
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`rustsheet-oracle fill` accepts workbooks whose SHEETS and INFO calls cannot see the canary sheet. It
keeps refusing the calls whose answer depends on the workbook's sheet count. In-scope M5 functions
INFO and SHEETS (`oracle: none` in eval/catalog/functions.yaml) can then get Excel evidence.

## Context
- tools/src/rustsheet_tools/oracle/canary.py `_SHEET_COUNT_FUNCTIONS` refuses any workbook that
  contains `SHEETS(` or `INFO(`. The reason is that the appended trailing `RustSheetCanary` sheet
  changes the sheet count (docs/architecture/oracle.md, "INFO and SHEETS have no web values at all").
  As a result the report's "Need an evidence source other than the oracle" list is INFO and SHEETS.
  Their Excel cell counts are 0 and 1, below the 20-cell floor that M5's catalog coverage needs.
- Only these calls depend on the sheet count: `SHEETS()` with no argument, and `INFO("numfile")`,
  matched case-insensitively and with any spaces. The canary is appended after the last sheet, so
  `SHEETS(ref)` of a cell, a range or a 3-D span cannot include it. Neither can INFO with another
  literal type ("directory", "origin", "osversion", "recalc", "release", "system", or invalid text).
- Keep refusing what the regex cannot decide. INFO with a non-literal argument (`INFO(A1)`,
  `INFO("num"&"file")`) is refused, and so is any SHEETS argument that is not a reference or a
  reference function. When unsure, refuse.
- A 3-D reference with the canary sheet's name cannot occur: `embed_canary` already refuses
  workbooks that have a sheet with that name.

## Acceptance criteria
- [ ] Tools tests: refused: `=SHEETS()`, `=SHEETS( )`, `=_xlfn.SHEETS()`, `=INFO("numfile")`,
      `=INFO("NUMFILE")`, `=INFO( "numfile" )`, `=INFO(A1)`, `=INFO("num"&"file")`.
      Accepted: `=SHEETS(A1)`, `=SHEETS(Data!A1:B2)`, `=SHEETS(S1:S3!A1)`, `=INFO("recalc")`,
      `=INFO("system")`, `=INFO("x")`.
- [ ] The refusal message still names the offending call.
- [ ] canary.py's module docstring and docs/architecture/oracle.md describe the narrower rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Running the probes (T-2115).
- Engine changes to SHEETS or INFO.
- A different canary placement for bare `SHEETS()` (file a follow-up if T-2115 shows it is needed).

## Outcome
`canary.sheet_count_call` replaces the blanket SHEETS/INFO regex. It refuses bare `SHEETS()` and
`INFO("numfile")` (any case, spaces) and every call it cannot decide; it accepts `SHEETS(<plain cell
or range, with optional 3-D sheet prefix>)` and `INFO("<other literal>")`. The refusal names the
offending call text. Tests in tools/tests/test_oracle_canary.py cover every listed form; docs
updated (canary.py docstring, oracle.md). No eval change: running the probes is T-2115.
