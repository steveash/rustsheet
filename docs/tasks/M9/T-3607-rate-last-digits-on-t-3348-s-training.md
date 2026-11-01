---
id: T-3607
title: "RATE last digits on T-3348's training sweep: 5 t3348-rate-gate-a-sweep cells 1 to 3 ulp from Excel in the noise floor (Phase 1b)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-3579]
defer: phase-1b
---

## Goal
On the 5 owned cells, RATE returns Excel's bits. Each cell already has the right type and is within half a unit of
the 15th significant digit of Excel's value, so this is last-digit work. The fix must come from the residual model
that T-3579's evidence pins down for RATE's noise floor, not from a rule for these 5 cells.

## Context
- Gap triage 2026-11-01c (`make eval` on origin/main 4f55f1fbf, excel-graph). The sweep has 300 cells, 293 passing.
  Of its 7 failures, T-3424 owns A272 and A296 (type misses). No open task, on main or in an open task PR, named
  the other 5. T-3591 (main) and T-3596 (#2676) own the four noise-floor digit cells of the *fresh* sweep
  (probes-numeric/t3348-rate-gate-a-fresh). They do not own this one.
- This sweep is T-3348's training evidence. A171 and A231 helped build the amended model (T-3348 task, financial.md
  "RATE holds on a flat secant and runs on below -1"). So these 5 cells cannot be the hold-out for a fit; the fresh
  sweep and a newer one are.
- Excel, ours, the 50-digit root (mpmath `findroot` on pv·(1+r)^n + pmt·(1+r·type)·((1+r)^n - 1)/r + fv at 50
  digits, computed in this triage), then each side's relative error against the root:
  - Calc!A228 `RATE(102,2166497722.99291,-207421358353.562,2596415.8989201,0)`: 0.001243745130470952 /
    0.0012437451304709525 / 0.0012437451304698744749; 8.7e-13 / 8.7e-13 (1.6 ulp apart).
  - Calc!A269 `RATE(147,1182658570.6986,-155237025269.246,387558.719282563,0)`: 0.0015611674850550617 /
    0.0015611674850550608 / 0.0015611674850543476217; 4.6e-13 / 4.6e-13 (2.5 ulp).
  - Calc!A271 `RATE(2,-875955339.667909,-10743.2012310885,2915489978.03367,1)`: 0.391647652294285 /
    0.3916476522942853 / 0.39164765229428521326; 5.5e-16 / 1.6e-16 (3.2 ulp). Here the engine is closer to the
    root than Excel.
  - Calc!A273 `RATE(3,16880559841.4338,117.162591922508,-57092918821.1684,1)`: 0.06116288981935156 /
    0.06116288981935157 / 0.061162889819351039741; 8.5e-15 / 8.7e-15 (1.0 ulp).
  - Calc!A285 `RATE(33,-5043229791.46827,159357373389.657,24379537.2625102,0)`: 0.002565758866566991 /
    0.00256575886656699 / 0.0025657588665648835178; 8.2e-13 / 8.2e-13 (1.5 ulp).
- Every value is within 1e-9 of the root, so decision 0188 R1(c) counts these cells as Phase 1b evidence (R5), hence
  `defer: phase-1b`. They get no `campaign:` key: no cell is visibly off (decision 0114). The terms run from 1e9 to
  2e11, so the secant ends in the residual's rounding noise. Excel's A228, A269 and A285 are 4.6e-13 to 8.7e-13 off
  the root, about 1e3 ulp. That is the same kind of miss as the 7 flat-secant cells of t3348-rate-fv-nonzero, and
  T-3389 could not reproduce those (financial.md "Not reproduced: in the noise floor", which now cites these cells).
  So a more accurate kernel will not match Excel here; only Excel's residual noise will.
- Needs Excel evidence first: T-3579 probes Excel's residual bits at the guess and near the root. This task waits on
  it, because the residual and step forms T-3389 tried (96 residual roundings among them; financial.md) do not
  give Excel's value on the cached workbooks.
- Code: `compute_rate`, `rate_residual` (crates/rustsheet-engine/src/functions/financial/financial_util.rs); fit lab
  `rate` (tools/src/rustsheet_tools/fitlab/kernels/solvers.py). Engine code must not read docs/ (decision 0066).
- Cells owned:
  - probes-numeric/t3348-rate-gate-a-sweep.case.json (5): Calc!A228, A269, A271, A273, A285

## Acceptance criteria
- [ ] Each owned cell is bit-exact against Excel. Or the Outcome names it with the residual and step forms tried, and
      it stays a known failure.
- [ ] These 5 cells score as passing in `make eval` for probes-numeric (the suite is `gate = false`, so they have
      no eval/baseline line), and `make eval-baseline` records no regression elsewhere. RATE's fit set stays bit-exact (960 cells, as T-3348 counts it), and so does every other RATE
      oracle cell that passes today.
- [ ] Any constant or residual form fitted or refitted here follows decision 0188 R4 (reviewer-excel-fidelity check
      7), shown in the Outcome:
      - hold-out counts against the correctly rounded root (R4.1), with t3348-rate-gate-a-fresh as the test split
        (never this training sweep);
      - a fresh sweep oracle-filled after the freeze (R4.2);
      - its Gate A envelope result: 0 points outside and 0 error-against-number cells (R2);
      - a `FITTED(T-3607, n_cells, range)` tag on each new or refitted constant, with the fallback to accurate math
        outside the range (R4.3);
      - no step below the reproducibility floor (R4.4);
      - no keep-matched branch (decisions 0056, 0082).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The sweep's type misses A272 and A296 (T-3424). The fresh sweep's digit cells (T-3591, T-3596). The 7
  t3348-rate-fv-nonzero noise-floor cells (T-3389's Outcome; T-3579).
