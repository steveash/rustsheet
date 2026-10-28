---
id: T-2560
title: "DDB with an FV-derived life of 1E86 is one ulp off (gen-m9 1dce4774f5c0e736 A366)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
gen-m9/1dce4774f5c0e736 Calc!A366 `=DDB(46469,651,FV(100,42,-935),9,2)` returns Excel's bits
(6.544613631714984e-81; ours 6.544613631714983e-81, one ulp low).

## Context
Left over from T-2033 (DB and DDB operation order), which fixed every other DB/DDB cell of the gen-m9
sweep. Here the life is FV(100,42,-935), about 1E86, so the rate `2/life` is about 2E-86, the book value
`cost*(1-rate)^8` rounds to `cost`, and the charge is `cost*rate`. The ulp therefore most likely comes
from the FV argument (or from how Excel forms `factor/life` at that magnitude), not from DDB's order;
unverified. First step: get Excel's `FV(100,42,-935)` and `2/FV(100,42,-935)` (excel-probe skill) and
compare with ours (functions/financial/annuity.rs FV; financial_depreciation.rs `ddb_depreciation`).
Known failure line: `F gen-m9/1dce4774f5c0e736.case.json Calc!A366` in eval/baseline/gen-m9.txt.

## Acceptance criteria
- [ ] Probe cells for FV(100,42,-935) and the DDB parts land with oracle values; the cause is recorded
      in docs/excel-semantics/financial.md.
- [ ] The A366 line leaves eval/baseline/gen-m9.txt (`make eval-baseline`) with no new failure lines,
      or the task is closed with the evidence showing why it cannot be matched yet.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Re-ordering DB/DDB again (T-2033 fits every other cell).

## Deferred (decision 0114)
`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells visibly off (then give it a `campaign:` stage instead).
