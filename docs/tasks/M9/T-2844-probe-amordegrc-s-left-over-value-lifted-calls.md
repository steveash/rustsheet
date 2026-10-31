---
id: T-2844
title: "Probe AMORDEGRC's left-over value: lifted calls, dynamic-array formulas in the order, a NaN AMORLINC charge"
milestone: M9
lane: functions-financial
status: done
priority: P3
size: S
tier: sonnet
depends_on: [T-2825]
---

## Goal
Excel evidence for the three unprobed parts of financial.md "AMORDEGRC below period 1", and the engine matching it.

## Context
T-2825 (decision 0131) models the value an AMORDEGRC with 0 < period < 1 returns: what the last AMORDEGRC or
AMORLINC Excel ran left (`OrderState::amor_left_over`, `Model::amortization_call` in
functions/financial/amortization.rs, `LeftOver`). Three behaviours are guesses, marked unprobed in the entry:
1. What a lifted call leaves. Ours: the last element's value. Probe, for example, A1 the fraction
   `=AMORDEGRC(10000,39000,39400,500,0.3,0.3,0)`, A2 `=SUM(AMORDEGRC(2400,39679,39813,300,{1,2},0.15,1))` (776 and
   485), A3 another value-leaving charge; and the fraction lifted with a non-fractional element.
2. What a dynamic-array formula's place in Excel's order does to the value (ours: nothing; for WEEKNUM it empties
   the state, excel_order.rs). Probe the fraction above a `=SEQUENCE(2)` above a 485 charge.
3. An AMORLINC error after the arguments are checked (a NaN charge, `linear` in amortization.rs). Ours: keeps the
   value. Only if a reachable input exists; otherwise record that none does.
Use the three-cell shape of probes/amordegrc-fraction-state-z (fraction / subject / 485 charge), so "keeps" and
"sets" are told apart. Probes in eval/suites/probes/cases/amordegrc-fraction-state-*.case.json, tag M9.

## Acceptance criteria
- [x] Probe cases for items 1 and 2 (and 3 if reachable) with excel-graph oracle values.
- [x] financial.md "AMORDEGRC below period 1" states each finding with its probe; the engine matches them (or a
  known failure is named in the entry with its reason).
- [x] `make check` green; reviewers approve.

## Out of scope
- gen-m9 d912671baa1364aa Calc!A76 (unexplained, financial.md).

## Outcome
- Probes `amordegrc-fraction-state-ac` to `-ai` (excel-graph, 2026-10-30). Findings in financial.md "AMORDEGRC below period 1":
  a lifted call runs on a state of its own (empty at the start, elements in order) and leaves the workbook's state empty;
  engine matches (`Model::amortization_call`): -ac to -af, -ai pass.
- Item 3 (NaN AMORLINC charge): no reachable input (needs 1024 single walk steps); recorded, no probe.
- Known failures, named in the entry: -ag (a spilling dynamic-array formula should empty the state; emptying it at its place
  in `run_excel_order` had no effect, so the pass's handling of a spilling anchor is unclear) and -ah (the charges inside a
  dynamic-array formula). Follow-up: T-3393.
- Eval: 9 probe cells newly pass (baseline probes.txt); no regressions.
