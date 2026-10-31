---
id: T-3393
title: "Explain AMORDEGRC left-over state around dynamic-array formulas (probes -ag, -ah)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel's behaviour for AMORDEGRC's left-over state around dynamic-array formulas is explained and the engine matches it.

## Context
T-2844 left two known failures: probes/amordegrc-fraction-state-ag (a fraction above `=SEQUENCE(1,2)` above the 485
charge is 0 in Excel, 485 ours; emptying `amor_left_over` at a dynamic-array formula's place in `run_excel_order` did not
change it, so how the pass reaches a spilling anchor needs finding) and -ah (`=SEQUENCE(1,2)+AMORDEGRC(..485..)` in A2
above an AMORLINC in A3 leaves 485 for A1, ours 132; -ai, the same formula with `*0` and no A3, leaves 0). See
financial.md "AMORDEGRC below period 1".

Cell lists (gap triage 2026-10-31: the short names -ag and -ah are not case names `exit_owners.py` reads):
- probes/amordegrc-fraction-state-ag.case.json (1): Sheet1!A1
- probes/amordegrc-fraction-state-ah.case.json (1): Sheet1!A1

Related: T-3499 (gen-m9 d912671baa1364aa A76, another left-over value the order table does not explain); if one rule
explains both, one PR may close both tasks.

## Acceptance criteria
- [ ] A probe of an error element in a lifted call (`{1,-1,0.5}`) is added. More probes tell where a dynamic-array formula (and the charges inside it) sits in the order; -ag and -ah pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
