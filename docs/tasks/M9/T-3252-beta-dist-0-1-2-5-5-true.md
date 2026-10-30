---
id: T-3252
title: "BETA.DIST(0.1,2.5,5,TRUE) is visibly off: bpser for both shapes in (1, 8) where Excel is 16 ulp above the true value (m9-beta-dist-a2p5 A31, plus 12 m9 cdf last-digit cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
`BETA.DIST(0.1,2.5,5,TRUE)` (probes-numeric/m9-beta-dist-a2p5 Calc!A31) is within half a unit of the 15th significant
digit of Excel's value, with no new visibly-off cell. The 12 other bpser cdf cells of the m9 beta sweeps listed below,
all within 15 digits already, are matched to the bit where the same change reaches them.

## Context
- Gap triage 2026-10-30 (`make eval` on origin/main 6a26b992a, excel-graph). `python3 scripts/exit_owners.py --all-off`,
  run over the task files of origin/main and of every pushed branch, left this one visibly-off cell with no open
  owner. T-3007 (done) owned it through T-2981's routing ("BETA.DIST cdf, ported BpserLarge"), and its Outcome handed
  its open items to T-3170 and T-3192, neither of which names it. The 12 bit-level cells had no owner either.
- Every cell is on TOMS bratio's bpser: the tiny and half shapes (b = 0.0001, 0.5) go there directly, and for both
  shapes > 1 here b0 < 40 with b0·x0 <= 0.7 (A31: b0 = 5, b0·x0 = 0.5). statistics.md "Incomplete beta bpser" has the
  model of record: L is the extended GAMMALN sum for 1 < b0 < 8 and algdiv for b0 >= 8 (A37 of both sweeps, a2 A49).
- Stage `visible`, not `accuracy`: against a 60-digit mpmath value (this triage, `mpmath.betainc(a, b, 0, x,
  regularized=True)`), Excel is the inaccurate one. A31: true 0.055333818608509839059, Excel +16.1 ulp, ours +8.1 ulp.
  For the rest the two are within a unit of each other: m9-beta-dist-a2p5 A3 Excel +6.5 / ours +5.5, m9-beta-dist-a2
  A1 +6.2 / +7.2, m9-legacy-betadist A18 -2.4 / -1.4. So reaching Excel's digits means reproducing its bpser sum (its
  term count, stop, or the stored z), not a more accurate kernel (decision 0114).
  The finding is recorded in docs/excel-semantics/statistics.md "Incomplete beta bpser" (this triage).
- The shapes probe (t2464-r2-bpser-small-shapes) is within 15 digits on all 4,602 cells at 1 < b0 < 8, so A31 is an
  isolated miss of the model: start by running the research model (`docs/research/t2464/w3-series/model.py`) at A31's
  arguments. If it gives Excel's value, the engine's port differs from the model there; if not, the model's L or series
  stop is wrong at this point, and a dense oracle sweep around (x, 2.5, 5) is the next step (research lane, file it
  and depend on it rather than fitting blind).
- T-3007's Outcome lists m9-legacy-betadist A17 and A18 among the 15 bit-exact cells its betaln change moved off.
- Code: `crates/rustsheet-engine/src/functions/statistical/bratio/bpser.rs` (`bpser`), dispatched from
  `beta_dist_lower_tail` in `bratio/mod.rs`.

Excel vs ours (`make eval-show CASE=probes-numeric/<case>.case.json`):
- m9-beta-dist-a2p5 Calc!A31 `=BETA.DIST(0.1,2.5,5,TRUE)`: 0.05533381860850995 vs 0.055333818608509895 (visibly off)
- m9-beta-dist-a2p5 Calc!A5 `=BETA.DIST(0.9,2.5,0.0001,TRUE)`: 0.00011702917767280959 vs 0.0001170291776728098
- m9-beta-dist-a2 Calc!A37 `=BETA.DIST(0.1,2,10,TRUE)`: 0.30264311979999997 vs 0.3026431198
- m9-legacy-betadist Calc!A17 `=BETADIST(0.3,10,1.5)`: 1.8630179360888307e-05 vs 1.863017936088831e-05

Owned cells:
- probes-numeric/m9-beta-dist-a2p5.case.json (6): Calc!A3, A5, A7, A9, A31, A37
- probes-numeric/m9-beta-dist-a2.case.json (5): Calc!A1, A3, A5, A37, A49
- probes-numeric/m9-legacy-betadist.case.json (2): Calc!A17, A18

## Acceptance criteria
- [ ] m9-beta-dist-a2p5 Calc!A31 is within half a unit of the 15th significant digit of Excel's value in `make eval`
      (probes-numeric is ungated: show it with `make eval-show`), and `make unowned-off ALL=1` lists no new cell.
- [ ] No gated known-failure line is added to any `eval/baseline/` file, and the report's BETA.DIST, BETADIST,
      F.DIST, F.DIST.RT, BINOM.DIST and NEGBINOM.DIST visibly-off counts do not rise (bpser is shared).
- [ ] The Outcome reports the owned cells bit-exact before and after, and docs/excel-semantics/statistics.md
      "Incomplete beta bpser" records what A31 showed (model result at A31, the form adopted or rejected).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The bpser items T-3192 owns (b0 >= 8 L diff on the shapes probe, the 987 cells T-2505's port moved off).
- BETA.DIST densities (T-3220, T-2586 and the Loader residue follow-ups).

## Visible-parity stage
Target (decision 0114): m9-beta-dist-a2p5 Calc!A31 within half a unit of the 15th significant digit of Excel's value.
The 12 bit-level cells and any bit-level residue of A31 move to a follow-up with `defer: phase-1b` if this task does not
reach them.
