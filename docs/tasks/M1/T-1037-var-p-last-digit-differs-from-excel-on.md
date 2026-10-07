---
id: T-1037
title: "VAR.P last digit differs from Excel on small integer data"
milestone: M1
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
VAR.P (and AGGREGATE 11) match Excel to the last digit.

## Context
Found by T-0993: `AGGREGATE(11,0,E1:E6)` over 3,1,4,1,5,9 is 7.472222222222222 in Excel and 7.472222222222221 in the engine (probes/aggregate-grid Grid!B31, C31, known failures). AGGREGATE hands the numbers to `fn_var_p` (crates/rustsheet-engine/src/functions/statistical/variance.rs), which uses the one-pass sum-of-squares form. Add a direct `VAR.P` probe of the same data first to confirm the gap is VAR.P's, then fit Excel's algorithm (it also overflows for equal huge values, numbers.md "Overflow": not a stable two-pass deviation; T-1029).

## Acceptance criteria
- [ ] probes/aggregate-grid B31, C31 pass and a direct VAR.P probe of the same data passes, with no regressions in other VAR.P cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Handoff
Opus run 2026-10-07. I stopped at the session budget, before implementing. The evidence is
committed on this branch: probes/variance-digits (234 cells) and probes/variance-switch (82 cells),
oracle-filled, with the new known failures baselined by `make eval-baseline`. The analysis is in
docs/excel-semantics/statistics.md, "VAR/STDEV switch between a one-pass and a two-pass formula".
Every cell is either the one-pass form (P: `(n*ss - s*s)/(n*n)`, S: `(ss - s*s/n)/(n-1)`) or the
two-pass form `Σ(x - s/n)²/(n-d)`. Excel switches to two-pass on cancellation (relative test,
t in (6.93e-4, 1.01e-2)) or on small magnitude (absolute test; brackets listed in the entry).
Next:
1. Probe to narrow t and a, for example {1,2,4}+10^j·c for c in 1..9, and {1,2,4}×c·10^-3.
2. Find the exact test form, perhaps with nice constants such as 1e-3 or 2^-10.
3. Implement one shared helper for VAR.P/VAR.S/STDEV.P/STDEV.S (variance.rs, standard_dev.rs).
   Check VARA/VARPA/STDEVA/STDEVPA, which have the same accumulate code.
4. Run `make eval-baseline` to record the fixes, including probes/aggregate-grid B31/C31.
