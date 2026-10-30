---
id: T-2907
title: "Pin the four AMORDEGRC past-2^20-period DEVIATION cells (fin-amor-period-bound-c A1, A3, A4, A5) in probes/deviations, or match Excel's walk within the cap"
milestone: M9
lane: functions-financial
status: dropped
priority: P3
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
The four probes/fin-amor-period-bound-c cells that differ from Excel only because AMORDEGRC stops walking at 2^20
periods (a directive-5 work cap, already an accepted DEVIATION) are scored as deviations, not as visibly-off
known failures: either pinned in eval/suites/probes/deviations with a decision record, or made to give Excel's
value by a cheaper exact continuation of the walk past the cap.

## Context
Gap triage 2026-10-27 (`make eval` on origin/main 6087413b). docs/excel-semantics/financial.md
"AMORDEGRC past 2^20 periods" is `Status: DEVIATION (decision 0014)`: past 2^20 periods the engine finishes the
decay in closed form, `remain * exp(n * log1p(-rate))` (functions/financial/amortization.rs,
`DEGRESSIVE_LOOP_PERIODS`, `DegressiveWalk::closed_form`), while Excel walks every period. Its evidence cells fail,
visibly off by one unit or in the 13th digit, and stay as F lines in eval/baseline/probes.txt:

| cell | formula | Excel | ours |
|---|---|---|---|
| Sheet1!A1 | `AMORDEGRC(1E20,39679,39813,300,1100000,1E-7,1)` | 18989305437102 | 18989305437101 |
| Sheet1!A3 | `AMORDEGRC(1E20,39679,39813,300,2400000,1E-7,1)` | 13720292047582 | 13720292047583 |
| Sheet1!A4 | `AMORDEGRC(3E19,39679,39813,300,1500000,1E-7,1)` | 5154670166168 | 5154670166167 |
| Sheet1!A5 | `AMORDEGRC(Z1,39679,39813,300,2000000,1E-7,1)` | 1.51632679480347E+293 | 1.51632679480336E+293 |

No `eval/suites/probes/deviations` line names them, unlike the other directive-5 cells there (array size and
Bessel order caps), so each counts toward the visible-parity lock as an engine miss (decision 0036,
docs/process/quality-gates.md "Accepted deviations"). No other DEVIATION entry has failing evidence cells (checked
across docs/excel-semantics/*.md on this run).

Two routes, in this order:
1. Match Excel within the cap. financial.md notes main's full walk gives A1, A3 and A4 with Excel's final rounding
   but not A2 or A5, because Excel's own per-period arithmetic drifts from main's (T-2849, which owns that
   arithmetic below 2^20). So this route is open only if T-2849 finds Excel's arithmetic and a bounded-work
   continuation reproduces it (for example a walk in chunks with a periodic exact reset). Do not raise the cap.
2. Otherwise pin them: one `deviations` line per cell with the engine's value, citing decision 0014 and
   `docs/excel-semantics/financial.md#amordegrc-past-220-periods` (check the anchor `make eval-check` resolves).
   Editing the deviations file is a protected change: write the decision record with `make new-decision` (decision
   0036 asks for one; the Excel-fidelity reviewer confirms the cap is a directive-5 limit).

## Acceptance criteria
- [ ] probes/fin-amor-period-bound-c Sheet1!A1, A3, A4 and A5 no longer appear as `F` lines in
      eval/baseline/probes.txt (`make eval-baseline`), and no new F line appears in any baseline.
- [ ] Either they pass bit for bit, or each is a line in eval/suites/probes/deviations and the report counts them
      under `deviations`; then a decision record in docs/decisions/ accepts the four lines.
- [ ] financial.md "AMORDEGRC past 2^20 periods" says which route was taken and links the decision or the code.
- [ ] `make check` green; reviewers (including Excel fidelity) approve.

## Out of scope
- AMORDEGRC's per-period arithmetic below 2^20 periods (T-2849).
- Raising or removing the 2^20 work cap (directive 5).

## Visible-parity stage
The four cells above are no longer visibly-off known failures: they pass, or they score as accepted deviations.

## Outcome
Dropped: T-2918 took route 1 another way, by raising AMORDEGRC's walk bound (`DEGRESSIVE_LOOP_PERIODS`) from 2^20 to
2^22 periods with the lifted weight refitted (financial.md "AMORDEGRC past 2^22 periods" and "AMORLINC and AMORDEGRC
work cap"). probes/fin-amor-period-bound-c A1, A3, A4 and A5 now pass bit for bit, so nothing is pinned in
probes/deviations.
