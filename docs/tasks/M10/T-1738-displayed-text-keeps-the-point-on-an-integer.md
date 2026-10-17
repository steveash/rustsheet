---
id: T-1738
title: "Displayed text: ?.?? keeps the point on an integer, #% of 0.0049 is %"
milestone: M10
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Digit placeholders show Excel's text at the edges: `?.??` keeps the point (padded) on an integer, and `#%` of 0.0049 is "%".

## Context
`?.??`: 1000 → "1000.  ", 61 → "61.  " (ours drops the point: "1000  "); `#%`: 0.0049 → "%" (0.49 rounds to 0
and `#` shows no zero; ours "1%"), `%0` of 0.0049 → "%0" (ours "%1"). The round-half cells of `0`, `#,##0`,
`0.00`, `0%` ... (2.5 → "3", 0.125 → "0.13", ours half-to-even) are T-1711's, and the sign of -0.5 under `0`
("-1", ours "1") is T-1129's.
Evidence: suite gen-m10 (excel-graph, 2026-10-16, T-1611): `make eval` report, "Displayed text" section, which lists each failing text cell (Excel vs ours). Text cells are scored apart from values and are not baselined (decision 0068); the TEXT() cells on sheet T that use the same codes are value known failures in eval/baseline/gen-m10.txt. Formatter: crates/rustsheet-engine/src/formatter/ (shared by `Workbook::formatted` and TEXT). docs/excel-semantics/number-formats.md.

## Acceptance criteria
- [ ] The gen-m10 text cells under these codes pass in the report's "Displayed text" section (or the remaining ones are filed); the matching TEXT() cells on sheet T leave eval/baseline/gen-m10.txt; no regressions.
- [ ] number-formats.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
