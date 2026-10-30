---
id: T-3290
title: "BETA.DIST(0.1,2.5,5,TRUE) is visibly off: bpser for both shapes above 1 and at most 8 (m9-beta-dist-a2p5 A31, left by T-3007)"
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
`BETA.DIST(0.1,2.5,5,TRUE)` gives Excel's value, or at least a value within half a unit of its 15th significant
digit, and no other cell of the incomplete-beta family moves away from Excel.

## Visible-parity stage
`visible`: a 50-digit reference (mpmath `betainc(2.5, 5, 0, 0.1, regularized=True)` = 0.05533381860850983906)
puts Excel's 0.05533381860850995 18.2 ulp above the exact value and ours, 0.055333818608509895, 9.1 ulp above it.
Excel is the inaccurate side, so the fix reproduces Excel's own bpser arithmetic, not a more accurate kernel.
Target: the cell below within half a unit of the 15th significant digit (bit-exact expected), with no new line in any
eval/baseline/*.txt and no probes-numeric cell of the family newly visibly off.

## Context
- Gap triage 2026-10-30 (`make eval` on origin/main 9efff2847; `scripts/exit_owners.py --all-off` lists the cell as the
  one visibly-off BETA.DIST cell with no open owner). It was T-3007's, routed by T-2981 as "probes-numeric/
  m9-beta-dist-a2p5: BETA.DIST cdf, ported BpserLarge 1". T-3007 closed and handed its open items to T-3170 and then
  T-3192, and none of them names this cell. The other BpserLarge cells T-2981 routed have a shape above 8. That is
  the TOMS betaln item T-3007 ported. This one has max(a, b) = 5, so T-3007's port did not reach it.
- Branch: a = 2.5, b = 5, x = 0.1. TOMS bratio has a0 = 2.5 > 1, lambda = a - (a + b)x = 1.75 >= 0 (no swap), and
  b0·x0 = 0.5 <= 0.7, so the result is bpser(a, b, x) (`bratio/mod.rs` `ported_tails_with`, `Branch::BpserLarge`),
  with L the extended GAMMALN sum for min >= 1, max <= 8 (`bratio/bpser.rs` `bpser_neg_ln_beta`; statistics.md
  "Incomplete beta bpser: a forward series times EXP(fl(a LN x) - lnB) (T-2464)").
- The research model is within 15 digits on all 4,602 probe cells with 1 < b0 < 8
  (t2464-r2-bpser-small-shapes, same statistics.md entry), and the engine's GAMMALN is Excel's to the bit at 2.5 and
  7.5 (probes-numeric/t1042-gamma-half-int A76 and A81 pass). So first confirm that `dispatch` really gives
  `BpserLarge` here and that `bpser` returns a value, not `None`, which would leave the cell on today's kernel
  (`incomplete_beta::beta_reg_tails`). Then score the cell on the model (`docs/research/t2464/w3-series/model.py`,
  `docs/research/t2464/r2-bpser-small/lib.py` `bpser_w3`; `docs/research/t3007/cell_diff.py` for a full-eval A/B)
  and find the step where the engine and the model part.
- The case's other cdf cells that fail at the bit level, all within 15 digits: A3 `BETA.DIST(0.5,2.5,0.0001,TRUE)`, A5
  `(0.9,2.5,0.0001,TRUE)`, A7 `(0.1,2.5,0.5,TRUE)`, A9 `(0.5,2.5,0.5,TRUE)` (b < 1, another branch) and A37
  `(0.1,2.5,10,TRUE)` (b·x = 1, not bpser). Report in the Outcome whether the change moves them. They are not this
  task's cells.

Cells this task owns:
- probes-numeric/m9-beta-dist-a2p5.case.json (1): Calc!A31

## Acceptance criteria
- [ ] probes-numeric/m9-beta-dist-a2p5 Calc!A31 is within half a unit of Excel's 15th significant digit
      (`make eval-show CASE=probes-numeric/m9-beta-dist-a2p5.case.json`; probes-numeric is ungated, so the Outcome
      gives the before and after value), and no BETA.DIST, F.DIST, BINOM.DIST or NEGBINOM.DIST cell of
      probes-numeric goes from within 15 digits to visibly off (report.md "Visibly off" counts, main against HEAD).
- [ ] No new line in any eval/baseline/*.txt (`make eval-baseline` refuses regressions).
- [ ] The Outcome names the branch the engine took for A31 before and after the change, and the step where it
      differed from the research model. If no change reaches Excel, statistics.md's bpser entry records the
      candidates tried (decision 0056).
- [ ] `make check` green; reviewers approve.

## Out of scope
- bpser at b0 >= 8 and the 987 cells T-2505's port moved off (T-3192); the densities of this case (T-3220, T-3244).
