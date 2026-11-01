---
id: T-3590
title: "AMORDEGRC below period 1 in gen-m9 d912671baa1364aa A76: find which cell leaves Excel's 1 (bisect the workbook with probes)"
milestone: M9
lane: functions-financial
status: done
priority: P3
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
gen-m9 d912671baa1364aa Calc!A76 gives Excel's value. If the probes cannot explain Excel's 1, financial.md records
what they ruled out, and the cell is listed there as UNEXPLAINED with the probe ids.

## Context
- Gap triage 2026-11-01 (`make eval` on origin/main 408444f20): the only gen-m9 visibly-off cell that no function
  attribution and no open task owns (`make unowned-off`). Excel 1, ours 0.
  - Formula: `=AMORDEGRC(98882,37287,38647,SUMPRODUCT((1e-15*Data!A10)),TIMEVALUE("1:30 PM"),0.1,1)`.
  - Data!A10 is 100, so the salvage is 1E-13 and the period is 0.5625.
- financial.md "AMORDEGRC below period 1" (T-1895, T-2825, T-2844) says that below period 1 Excel returns a value
  left over from another AMORDEGRC/AMORLINC call, taken in Excel's fresh-load order. The engine models that through
  `OrderState::amor_left_over` (evaluation.rs), `Model::amortization_call` (functions/financial/amortization.rs) and
  `excel_order.rs` (SPEC-0017). The same entry says A76 is UNEXPLAINED:
  - the next AMOR* call in that order, A96 `AMORLINC(92598,36663,39379,343,6,1E-300,3)`, leaves 0 (probe
    amordegrc-fraction-state-r);
  - the case's other AMORDEGRC and AMORLINC cells are errors: A27, A47, A125, A145, A194, A223, A243, A272, A321
    `#NUM!`; A174, A390 `#N/A`; A292, A341, A370 `#VALUE!`.
  - So no cell visible in the table leaves 1.
- Hypotheses to separate:
  1. A call that errors still leaves a state: an error raised after the walk started, for example in a later
     argument check.
  2. The value is not left over: some argument pattern gives 1. The salvage here is a SUMPRODUCT of a tiny product,
     while the probed ones were literals.
  3. The order reaches a cell on Pivot or Db first (4 sheets; the probes used one or two).
- Method: excel-graph probes (`excel-probe` skill) on copies of the case:
  - drop one sheet, then halves of the AMOR* cells, then the non-AMOR cells, until the 1 goes;
  - then a minimal workbook that reproduces it.
  Name each probe in the Outcome.
- Related: T-3393 (dynamic-array formulas in the order, probes -ag and -ah).

## Visible-parity stage
Decision 0083: the cell is visibly off (1 against 0). The target is Excel's value. If no rule is found, the target
is an UNEXPLAINED entry naming the probes.

## Acceptance criteria
- [x] Probe cases (probes/, tag M9, excel-graph) that bisect the workbook, each listed in the Outcome with its
      result.
- [x] gen-m9/d912671baa1364aa.case.json (1): Calc!A76 passes and its line leaves eval/baseline/gen-m9.txt, or
      financial.md states the rule ruled in or out and the cell stays a known failure.
- [x] Every amordegrc-* and amor-* probe still passes. `make eval-baseline` records no regression.
- [x] `make check` green; reviewers approve.

## Out of scope
- Dynamic-array formulas in the order (T-3393).

## Outcome
Excel's 1 does not reproduce, so there was nothing to bisect. The cell stays a known failure, and financial.md
("AMORDEGRC below period 1") records it as UNEXPLAINED and not reproducible.

Probes (probes/, M9, excel-graph, 2026-11-01). Each checks Calc!A76 only, except -g:

| probe | workbook | Excel A76 |
|---|---|---|
| amordegrc-a76-bisect-a | gen-m9 d912671baa1364aa unchanged (same input sha256 685de544… as the 10-28 run) | 0 |
| -b | without the pivot table | 0 |
| -c | Calc A1:A76 only | 0 |
| -d | Calc A76:A400 only | 0 |
| -e | Calc A76 only (Data, Pivot and Db kept) | 0 |
| -f | Calc A76:A96 only | 0 |
| -g | a lone `AMORDEGRC(2400,39679,39813,300,2,0.15,1)`, filled just before -h | 485 (Sheet1!A1) |
| -h | the whole workbook plus a constant Data!Z1, filled 9 s after -g | 0 |

- In -a, the 10-28 run differs from today's in one cell of 413: A76. The bytes are identical, so the 10-28 value
  is not a function of the workbook alone. That puts it with WEEKNUM's state after edits (decision 0038), not with
  the fresh-load rule.
- -g then -h shows the left-over value does not carry from one workbook to the next one the service calculates.
- Every subset agrees with the documented rule (no AMOR* call leaves a value for A76: 0), so hypotheses 1 to 3 cannot
  be told apart.
- Not done: re-filling gen-m9 d912671baa1364aa's oracle to today's 0. Picking the run that matches us would be
  choosing evidence by outcome. The sidecar stays as Excel produced it.
- Eval delta: 8 new probe cases, all passing (ours 0 and 485). No baseline line added or removed: gen-m9 A76 stays F.

