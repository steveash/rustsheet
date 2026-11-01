---
id: T-2636
title: "PROB and FREQUENCY: an error element inside an array argument is that error (T-2430 follow-up)"
milestone: M9
lane: functions-stat
status: done
priority: P1
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
An error element inside an array argument of PROB and FREQUENCY is that error, as for LARGE and the other functions of T-2430, with the other argument's error still winning where Excel does.

## Context
T-2430 reverted PROB and FREQUENCY to `#VALUE!` for an array's error element: making them propagate it regressed two gen-m9 cells
(gen-m9/589f42e67f4b4b8d A338 `FREQUENCY((TOCOL(..)/Data!$A8),SLOPE(..))` Excel `#VALUE!`, 7fd0a98183c6babf A331 `PROB(VSTACK(..),..)` Excel `#VALUE!`),
so the order of the arguments' errors (data, bins, limits) needs a probe first (excel-probe skill). Code: `numeric_flat_of` in
functions/statistical/prob.rs (T-2430 added `numeric_flat_with_element_errors` for the others), frequency.rs.
Still failing from T-2430's list and not covered here: F.TEST/FTEST (T-2093), FORECAST.ETS* and XNPV `#N/IMPL!`, `SMALL((SMALL({1,#N/A,3},2)/1.005),..)`
(1b31092a0d79e5f1 A363), `T.TEST(SORT(..))` (0620505280d2ea6e A31).
- Conflicting desktop evidence for the bins argument (M5 planner, 2026-10-25). corpus-bench/53062 (`formamspnc (7)`, AE6:AQ6
  and the cells downstream; the case is a whole-case `F *` line in eval/baseline/corpus-bench.txt) uses the CSE array formula
  `LOOKUP(1,1/FREQUENCY(0,ABS($R$1:$AI$1-R6)),$R$1:$AI$1)`, where AI1 is the empty text `""`. Its bins array therefore has a
  `#VALUE!` element. Excel's cached result is a number (AE6 = 0, AI6 = -2), so desktop FREQUENCY did not return `#VALUE!` for
  that error element in bins. The engine gives `#VALUE!` (`ROWS(FREQUENCY(0,ABS({13,15,17,""}-13)))` is `#VALUE!`). This
  failure is most of the case's 338 failing cells. The probe must cover an error element in bins on its own, in addition
  to the data argument, before error propagation is implemented: `ROWS(FREQUENCY(0,{1,#VALUE!,3}))`,
  `INDEX(FREQUENCY(0,{1,#VALUE!,3}),1)`, `FREQUENCY({1;2},{#N/A})`.
- Owner of the whole case corpus-bench/53062/1_53062_input.xlsx (gap triage 2026-10-27, as T-2909 asks):
  the report for a84da30f has 334 visibly-off Excel cells there. They are all on the one sheet, and 325 are `#VALUE!` where
  Excel has a number or text. They are blamed on SUM (72), LOOKUP/ABS/ISBLANK, IF, CONCATENATE, ISNUMBER, COUNTIF(S) and
  SEARCH, but they all lie downstream of the FREQUENCY bins error above. The other 9 are COUNTIF/SEARCH tests that read
  those `#VALUE!` cells.

Update (T-2427, 2026-10-27): FREQUENCY's data array now propagates an error element, and a scalar bins error beats it
(probes/frequency-bins B1-B3). Left here: PROB; FREQUENCY's bins array (still `#VALUE!`, the corpus-bench/53062 evidence
above); and an error cell in a data range against a scalar bins error, which keeps `main`'s data-first order unprobed.
Also to probe (T-2427 review, 2026-10-28): `FREQUENCY({1;2},"2")` and a referenced `"2"` bin (numeric text, skipped in ours);
a typed or computed boolean as data (`FREQUENCY(TRUE,{1;2})`, `FREQUENCY(1=1,{1;2})`); a computed boolean bin
(`FREQUENCY({1;2},1=1)`); and a data-array error element against an error cell in a bins range.

## Acceptance criteria
- [ ] Probe the argument-error order of PROB and FREQUENCY with an array error element; record it in statistics.md.
- [ ] Probe `SLOPE`, `INTERCEPT`, `STEYX`, `RSQ`, `FORECAST.LINEAR` and `CHISQ.TEST` with `{1,#N/A,3},{1,2,3}`, and the reverse position (plain first argument, error element in the second); record it in statistics.md (T-2430 changed them unverified).
- [ ] The PROB and FREQUENCY gen-m9 cells with `{1,#N/A,3}` pass, with no regressions.
- [ ] If the probe shows that desktop and web FREQUENCY agree on an error element in bins, the corpus-bench/53062
      cells fixed by it pass. The Outcome gives the case's visibly-off count before (334) and after, and files a
      follow-up for any cell left.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Visible-parity stage
Target (decision 0114): every visibly-off cell this task owns is within half a unit of the 15th significant digit
of Excel's value, or has Excel's type, error and text. Bit-level criteria still unmet then move to a follow-up
filed with `defer: phase-1b`.

Phase 1 triage (T-0090, eval of 2026-10-27 at `main` 7f6b373a, task ownership as of `main` e5bc60c4): this task owns the 1 visibly-off SMALL cell and, from this triage, FREQUENCY gen-m9/fd9fa22d212080d6 Calc!A32 (#DIV/0! expected, #VALUE! today). Fixing them lifts SMALL to the Phase 1 rate (≥ 99.9% of attributed cells within 15 digits); no other open task owns enough of those cells.

## Outcome
Probed on excel-graph (probes/array-error-element-order, 31 cells): PROB, SLOPE, INTERCEPT, STEYX, RSQ, FORECAST.LINEAR and
CHISQ.TEST return an array's error element, the first argument's error first; an error element in FREQUENCY's bins array is
skipped like text (agrees with desktop corpus-bench/53062). PROB now propagates the element and FREQUENCY skips it in bins.
Recorded in statistics.md. Eval: gen-m9 known failures 3330 -> 3323 (7 fixed), no regressions; new probe case fully passing.
corpus-bench is not fetched in the worker environment, so the case-53062 visibly-off count (334) was not re-measured; the
steward's next full eval will show it. Still unprobed: a data-array error element against an error cell in a bins range.
