---
id: T-2094
title: "CHISQ.TEST and CHITEST over a range with no numbers are #DIV/0!, not 1"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
CHISQ.TEST and CHITEST return `#DIV/0!` when the actual or the expected range holds no number (all
blank), as Excel does, instead of the p-value 1.

## Context
Gap triage 2026-10-20. docs/excel-semantics/statistics.md, "Thin distribution functions ... (T-1049)",
"Test functions", already says: "CHISQ.TEST: ... `#DIV/0!` only for an empty range or a zero
expected value". No task implemented the empty-range part.
Failing cells (excel-graph; probes-numeric is not gated, so read them with
`make eval-show CASE=probes-numeric/<case>.case.json`). Each is Excel `#DIV/0!` and ours 1:
- probes-numeric/t1049-chisqtest Calc!A23 `CHISQ.TEST(B1:B4,Z1:Z4)` and A24
  `CHISQ.TEST(Z1:Z4,B1:B4)`, where Z1:Z4 is blank.
- probes-numeric/m9-legacy-chitest Calc!A28 `CHITEST(P1:P3,G1:G3)`, where P1:P3 is blank.
Code: `fn_chisq_test` in crates/rustsheet-engine/src/functions/statistical/chisq.rs (CHITEST,
`Function::ChiTestCompat`, shares it). A pair with a non-number side becomes `(1.0, 1.0)`, which
adds 0 to the statistic, so an all-blank side gives chi2 = 0 and p = 1. Add a check: if either side
has no `Some` value, return `#DIV/0!`. Put it after the existing `width * height < 2` `#N/A` check,
because `CHISQ.TEST(5,5)` (t1049-chisqtest A36) must stay `#N/A`.
Keep the pairing rule for ranges that hold some numbers. Text in one range is already right: t1049-chisqtest
A21 `CHISQ.TEST(G1:G4,H1:H4)` (G1 = "a") and A34/A35 pass. Partly blank ranges and whole columns are
T-1689's probe.

## Acceptance criteria
- [ ] The three cells above pass in `make eval`. Every cell of t1049-chisqtest and m9-legacy-chitest
      that passes today still passes, and no gated baseline (eval/baseline/*.txt) gains an F line.
- [ ] A unit test covers an all-blank actual range and an all-blank expected range (`#DIV/0!`), and a
      range with one text cell among numbers (unchanged value). The code comment cites the probe cells.
- [ ] statistics.md's T-1049 entry says the empty-range rule is implemented (T-2094).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Ranges that each hold numbers, but never in the same position (unprobed); partly blank ranges and
  whole columns (T-1689).
- CHISQ.TEST's last digits (T-0971).
