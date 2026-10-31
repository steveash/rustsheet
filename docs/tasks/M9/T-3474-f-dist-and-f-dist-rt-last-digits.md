---
id: T-3474
title: "F.DIST and F.DIST.RT last digits on the t2220 d1-1, d2-1 and rt sweeps: 111 visibly-off cells where Excel is 2 to 620 ulp off the math (Phase 1b)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: []
campaign: visible
defer: phase-1b
---

## Goal
The 111 F.DIST and F.DIST.RT cells below (70 densities, 41 cdf or right-tail cells, on the T-2220 sweeps with one df
of 1 and on the right-tail sweep, plus one t1538 cell) agree with Excel to 15 significant digits, by reproducing the
arithmetic Excel uses where Excel itself is off the math. No cell that agrees today is lost.

## Context
- Decision 0188 R5 (owner's 2026-10-31 rulings): parked as Phase 1b. Every cell is a dense probes-numeric sweep cell
  inside Gate A's envelope (SPEC-0029: ours is within 1E-9 relative of the 50-digit math on all 111) and Excel is
  itself 2 to 620 ulp off the math on every one (at 1.9 to 3 ulp only at d1-1 A179 and d2-1 A278, where ours is 3 to 5
  ulp off the other way), so this is reproducing Excel's last digits. A point that Gate A's tables show outside the
  envelope gets its own task without the key.
- Gap triage 2026-10-31i (`make eval` on origin/main 0c2fbf706, excel-graph; `scripts/exit_owners.py --all-off` and
  `--milestone M9`, `rate:F.DIST`, `rate:F.DIST.RT`): no open owner, deferred or not. T-3173 (|c·LN x| >= 4) and
  T-3174 (the constant K) own the M-final density residue of other sweeps but name none of these cases' cells;
  T-2790 (d1·x large against d2) and T-2221 (closed forms, complement, tail choice on the T-2220 sweep) name other
  cells of the same sweeps. When one of them is worked, fold the matching cells in from here (record the move in both
  files) rather than fitting twice.
- Signed ulps from mpmath at 60 digits, Excel / ours, examples:
  - density, d1 = 1: d1-1 A899 `F.DIST(1000000,1,100,FALSE)` +160.9 / +253.1; A179 `F.DIST(1000000,1,2,FALSE)`
    -2.9 / +3.1. Density, d2 = 1: d2-1 A2 `F.DIST(1E-6,2,1,FALSE)` -4.3 / +5.6. Over the 70 densities Excel is 1.9 to
    619 ulp off and ours 0 to 564; ours is the further one on 25.
  - right tail: rt A1000 `F.DIST.RT(100,100,100)` Excel 1.9019500844145964E-72 (+123.1), ours 1.9019500844145517E-72
    (-54.8); A879 `F.DIST.RT(10,50,20)` -39.0 / +129.2. On rt A579, A589, A868, A878, A879, A885, A897 and A900 ours
    is 27 to 129 ulp off where Excel is 7 to 39: an accuracy lead for the kernel tasks (T-3383 bfrac and bup
    arithmetic), not a reason to fit.
  - t1538-beta-dist-at-excel-inv A15 `F.DIST.RT(7.177637399104892,5,1E10)`: Excel 9.999999738287662E-7 (+36.3), ours
    9.999999738287633E-7 (+22.6); a df of 1E10 (T-1538's region).
- The 6 cells of the same group where Excel is within 1.6 ulp of the math and ours is not are T-3472's (accuracy).
- This is a numeric fit. The training points are the cached T-2220 sweeps (t2220-f-dist-sweep-pdf, -cdf, -rt, -d1-1,
  -d2-1) and the r4/r5 M-final hold-out cases; findings in statistics.md "Dense F sweep (T-2220)", "F density is not
  the beta density, R's df or a GAMMALN formula (T-2513)" and the M-final entries. Needs an oracle sweep for R4.2's
  fresh log-uniform check once a model is frozen: file it as a research-lane task when the key is removed.
- Code: crates/rustsheet-engine/src/functions/statistical/f_density.rs, `f_density/`, `fisher.rs`, `bratio/`.

Owned cells:
- probes-numeric/t1538-beta-dist-at-excel-inv.case.json (1): Calc!A15
- probes-numeric/t2220-f-dist-sweep-d1-1.case.json (42): Calc!A176, A179, A338, A443, A501, A518, A530, A536, A596, A608, A614, A620, A626, A686, A689, A692, A701, A707, A710, A713, A716, A766, A768, A779, A785, A788, A794, A803, A809, A861, A867, A869, A872, A875, A878, A881, A884, A887, A890, A893, A896, A899
- probes-numeric/t2220-f-dist-sweep-d2-1.case.json (39): Calc!A2, A188, A278, A316, A318, A455, A458, A464, A473, A479, A545, A548, A554, A557, A566, A575, A632, A635, A638, A641, A644, A647, A650, A659, A662, A664, A673, A675, A722, A725, A728, A731, A734, A737, A740, A743, A746, A749, A760
- probes-numeric/t2220-f-dist-sweep-rt.case.json (29): Calc!A84, A97, A98, A399, A567, A579, A589, A594, A596, A669, A680, A697, A795, A800, A804, A868, A878, A879, A885, A890, A897, A900, A960, A978, A990, A994, A997, A999, A1000

## Visible-parity stage
Decision 0083 / 0114 (`visible`): met when the 111 cells above are within half a unit of the 15th significant digit
of Excel's value, and no other cell of the five cases is newly visibly off. Bit-level criteria still unmet then move
to a follow-up without the campaign key and with `defer: phase-1b`.

## Deferred (decision 0114, decision 0188 R5)
`defer: phase-1b`: these are dense-sweep last-digit cells inside Gate A. Remove the key only when the steward records
the Phase 1 exit, or when Gate A's tables show one of them outside the envelope (then that cell gets its own task).

## Acceptance criteria
- [ ] At least 90% of the 111 cells above are within half a unit of the 15th significant digit of Excel's value; the
      rest are listed in the Outcome with Excel / ours / exact values.
- [ ] R4 guards (decision 0188, reviewer-excel-fidelity check 7), each recorded in the Outcome:
  - [ ] hold-out: train and test counts side by side from the fit lab's deterministic ~30% split; the accepted
        candidate's test exact rate is within the stated margin of its train rate and beats the correctly rounded
        candidate on test;
  - [ ] fresh sweep: after the model is frozen, a new log-uniform oracle sweep over the full domain is filled and its
        result recorded before merge;
  - [ ] Gate A envelope result (`make eval-gates` / `rustsheet-eval accuracy`) for F.DIST and F.DIST.RT: no point
        outside the envelope;
  - [ ] every fitted constant, cut-point or table carries `/// FITTED(T-3474, <n> cells, <range>)` and is not applied
        outside that range (accurate math there); quality/fitted.json moves only through `make hygiene`'s ratchet;
  - [ ] no fitting below the oracle's reproducibility floor (decision 0112: web against desktop up to 4 ulp);
  - [ ] no keep-matched branch: no code path that exists only to return Excel's value at the listed points.
- [ ] No new known-failure line in any eval/baseline/<suite>.txt (`make eval-baseline` refuses regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The accuracy cells of the same sweeps (T-3472) and the kernel accuracy leads above (T-3383).
- F.INV and F.INV.RT (T-3398). df of 1E7 and above beyond A15 (T-1538's follow-ups).
