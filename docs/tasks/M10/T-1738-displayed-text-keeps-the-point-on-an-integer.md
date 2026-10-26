---
id: T-1738
title: "Displayed text: ?.?? keeps the point on an integer, #% of 0.0049 is %"
milestone: M10
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
campaign: semantics
depends_on: []
---

## Goal
Digit placeholders show Excel's text at the edges: `?.??` keeps the point (padded) on an integer, and `#%` of 0.0049 is "%".

## Context
`?.??`: 1000 → "1000.  ", 61 → "61.  " (ours drops the point: "1000  "); `#%`: 0.0049 → "%" (0.49 rounds to 0
and `#` shows no zero; ours "1%"), `%0` of 0.0049 → "%0" (ours "%1"). The round-half cells of `0`, `#,##0`,
`0%` ... (2.5 → "3", ours half-to-even) are T-2206's (moved from T-1711, gap triage 2026-10-20), and the sign of -0.5 under `0`
("-1", ours "1") is T-1129's.
Evidence: suite gen-m10 (excel-graph, 2026-10-16, T-1611): `make eval` report, "Displayed text" section, which lists each failing text cell (Excel vs ours). Text cells are scored apart from values and are not baselined (decision 0068); the TEXT() cells on sheet T that use the same codes are value known failures in eval/baseline/gen-m10.txt. Formatter: crates/rustsheet-engine/src/formatter/ (shared by `Workbook::formatted` and TEXT). docs/excel-semantics/number-formats.md.

Update (T-2206): `0%` and `%0` of 0.0049 are fixed by the rounding change; `?.??` and `#%` of 0.0049 remain.

## Acceptance criteria
- [x] The gen-m10 text cells under these codes pass in the report's "Displayed text" section (or the remaining ones are filed); the matching TEXT() cells on sheet T leave eval/baseline/gen-m10.txt; no regressions.
- [x] number-formats.md updated.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Visible-parity stage
Decision 0083: a wrong TEXT or displayed-text result is visible to every user of the format, so this task is in the campaign. Its acceptance criteria are already the visible-parity stage.

## Outcome
A `?` padding the first decimal now prints the pending point before its space (formatter/format.rs), so `?.??` on 1000
is "1000.  " and on 61 "61.  " (formatter/test/test_digit_edges.rs). `#%` on 0.0049 was already "%" after T-2206; the
test pins it. `make eval-baseline`: gen-m10 text cells 394 -> 366 (28 fixed), no regressions in any suite.
