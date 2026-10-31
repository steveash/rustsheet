---
id: T-3475
title: "BETA.DIST last digits near a shape of 1 (w2f-unit-shape) and at shapes 5 to 100 (m9-beta-dist): 26 visibly-off cells where Excel is 6 to 235 ulp off the math (Phase 1b)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
campaign: visible
defer: phase-1b
---

## Goal
The 26 BETA.DIST cells below (6 cdf cells at a shape one to five ulps below 1, and 20 densities and cdf cells at
shapes 5 to 100 on the m9-beta-dist probes) agree with Excel to 15 significant digits, by reproducing Excel's
arithmetic where Excel itself is off the math. No cell that agrees today is lost.

## Context
- Decision 0188 R5 (owner's 2026-10-31 rulings): parked as Phase 1b. Every cell is a dense probes-numeric cell inside
  Gate A's envelope (SPEC-0029: ours is within 1E-9 relative of the 50-digit math on all 26) and Excel is itself 6 to
  235 ulp off the math on every one, so this is reproducing Excel's last digits. A point that Gate A's tables show
  outside the envelope gets its own task without the key.
- Gap triage 2026-10-31i (`make eval` on origin/main 0c2fbf706, excel-graph; `scripts/exit_owners.py --all-off` and
  `--milestone M9`, `rate:BETA.DIST`): no open owner, deferred or not. Signed ulps from mpmath at 60 digits, Excel / ours:
  - t2464-w2f-unit-shape, cdf at x = 0.1 with a = 1 - k·2^-52 (k 1 to 5) and b = 2.5 or 5: A211
    `BETA.DIST(0.1,(67108863*2^-26+67108859*2^-52),2.5,TRUE)` Excel 0.23156652857908355 (-25.6), ours
    0.2315665285790843 (+1.4); A407 (b = 5) Excel 0.4095099999999998 (-14.1), ours 0.4095100000000008 (+3.9); A409,
    A411, A421, A423 Excel -6.7 to -13.2, ours +2.2 to +6.6. Excel drops well below the a = 1 closed form
    1 - 0.9^b as a leaves 1, ours stays near the math: this is the "shape of 1 outside the closed forms" region
    (statistics.md, T-3007; T-3192 owns 6 other shape-of-1 cells of T-3170).
  - m9-beta-dist-a5, -a10, -a100 at x 0.1, 0.5, 0.9: a10 A20 `BETA.DIST(0.1,10,2,FALSE)` Excel 9.899999999999996E-8
    (-6.7), ours 9.900000000000004E-8 (-0.7); a5 A54 `BETA.DIST(0.9,5,100,FALSE)` +234.9 / +220.9; a100 A26 +181.9 /
    +159.3. On a10 A54, a100 A5, A20, A38, A43 and A44 ours is 71 to 147 ulp off where Excel is 7 to 69: an accuracy
    lead for the density and cdf kernels (T-3473, T-3401), not a reason to fit.
  - T-3401 (branch agent/functions/tasks-refill-opus when this was filed) says not to move m9-beta-dist-a50/-a100 cells
    at x 0.1 with b 2 to 10 where Excel is inaccurate and the engine matches it; these are the ones that do not match.
- The 4 cells of the same probes where Excel is within 2.5 ulp of the math and ours is not are T-3472's (a100 A51)
  and T-3473's (a10 A32, a100 A34, a5 A52).
- This is a numeric fit. The w2f cells have the cached t2464-w2f-* sweeps for training; the m9-beta-dist shapes 5 to
  100 have only the 54-cell m9 probes. Needs an oracle sweep (shapes 5 to 100 by x 0.05 to 0.95, density and cdf, with
  a hold-out half) before any fit: file it as a research-lane task when the key is removed, after checking T-3400's
  sweep (bulk cdf at one shape 1.5 to 39 and the other 20 to 1E5) for overlap.
- Code: crates/rustsheet-engine/src/functions/statistical/bratio/, `beta_density.rs`, `large_shape_beta.rs`.

Owned cells:
- probes-numeric/t2464-w2f-unit-shape.case.json (6): Calc!A211, A407, A409, A411, A421, A423
- probes-numeric/m9-beta-dist-a5.case.json (4): Calc!A5, A31, A42, A54
- probes-numeric/m9-beta-dist-a10.case.json (4): Calc!A20, A31, A48, A54
- probes-numeric/m9-beta-dist-a100.case.json (12): Calc!A5, A20, A25, A26, A28, A30, A32, A38, A43, A44, A46, A48

## Visible-parity stage
Decision 0083 / 0114 (`visible`): met when the 26 cells above are within half a unit of the 15th significant digit
of Excel's value, and no other cell of the four cases is newly visibly off. Bit-level criteria still unmet then move
to a follow-up without the campaign key and with `defer: phase-1b`.

## Deferred (decision 0114, decision 0188 R5)
`defer: phase-1b`: these are dense-sweep last-digit cells inside Gate A. Remove the key only when the steward records
the Phase 1 exit, or when Gate A's tables show one of them outside the envelope (then that cell gets its own task).

## Acceptance criteria
- [ ] At least 22 of the 26 cells above are within half a unit of the 15th significant digit of Excel's value; the
      rest are listed in the Outcome with Excel / ours / exact values.
- [ ] R4 guards (decision 0188, reviewer-excel-fidelity check 7), each recorded in the Outcome:
  - [ ] hold-out: train and test counts side by side from the fit lab's deterministic ~30% split; the accepted
        candidate's test exact rate is within the stated margin of its train rate and beats the correctly rounded
        candidate on test;
  - [ ] fresh sweep: after the model is frozen, a new log-uniform oracle sweep over the full domain is filled and its
        result recorded before merge;
  - [ ] Gate A envelope result (`rustsheet-eval accuracy`) for BETA.DIST: no point outside the envelope;
  - [ ] every fitted constant, cut-point or table carries `/// FITTED(T-3475, <n> cells, <range>)` and is not applied
        outside that range (accurate math there); quality/fitted.json moves only through `make hygiene`'s ratchet;
  - [ ] no fitting below the oracle's reproducibility floor (decision 0112: web against desktop up to 4 ulp);
  - [ ] no keep-matched branch: no code path that exists only to return Excel's value at the listed points.
- [ ] No new known-failure line in any eval/baseline/<suite>.txt (`make eval-baseline` refuses regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The accuracy cells (T-3472, T-3473) and the kernel accuracy leads above.
- BETA.INV (T-3398, T-3457).
