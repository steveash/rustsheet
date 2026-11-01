---
id: T-3042
title: "GAMMALN at the incomplete-beta kernel arguments: ship T-2122's kernels for the bpser and bratio GAMMALN probes (1,883 cells within 15 digits)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-2122]
defer: phase-1b
---

## Goal
The engine's GAMMALN gives Excel's bits at the shape arguments the incomplete-beta research measured. It does this
by shipping the K3, (1.5, 2] and K2 coefficient sets that T-2122 cracks, so the two GAMMALN probes' last-digit
cells pass.

## Context
Gap triage 2026-10-28 (`make eval` on origin/main 4b20263be). Neither case has an open owner. T-3036 owns only
t2464-r2-bpser-small-gl Calc!A3188, the one visibly-off cell, and keeps it. T-2122 (research, deferred) is cracking
the kernels but ships nothing. T-2128 shipped the current best sets.
- probes-numeric/t2464-r2-bpser-small-gl: every failing cell except Calc!A3188, which is 1,371 of 5,843 `=GAMMALN(x)`
  cells at each bpser shape's a, b and a + b. All are NumericUlp. Examples
  (`make eval-show CASE=probes-numeric/t2464-r2-bpser-small-gl.case.json`):
  - Calc!A1000 `GAMMALN(0.4508)`: Excel 0.675302165846252, ours 0.6753021658462519.
  - Calc!A1003 `GAMMALN(0.4516)`: Excel 0.6735210010144506, ours ...504.
- probes-numeric/t2464-r2-bratio-gammaln: every failing cell, which is 512 of 5,094 `=GAMMALN(exact literal)` cells
  at the bratio port's kernel arguments (docs/research/t2464/r2-bratio/RESULTS.md item 6, `check_gl.out`). Example:
  Calc!A1002 `GAMMALN((5033164*2^-23+53687317*2^-49))`: Excel 0.39823385806861694, ours 0.3982338580686169.
- Also gen-m9/40425341c5034eed Calc!A115 `GAMMALN.PRECISE((WRAPCOLS(1.005,2,7)^0.3))`: Excel
  -0.0008624698309035655, ours ...657 (K3 region).
- Why the misses happen: statistics.md "GAMMALN is Excel's own x87 routine (T-1941, T-2128)" gives the routine's
  structure exactly. The four rationals' coefficient doubles are not known to the bit. The engine's exact rates are
  0.322 in K3 [0.7, 1.5] and 0.428 in (1.5, 2]. Arguments below 0.7 inherit K3 through fl(GAMMALN(x + 1) - LN(x)).
  The same entry counts the engine model exact on 274 of 771 bpser-gl arguments in [1, 2) and 1,738 of 2,585
  below 1.
- Downstream: T-3041 (BETA.DIST bpser runs) depends on this, because Excel's L = ln B is built from these GAMMALN
  values. The r2-bratio study (item 6) found that GAMMALN error does not limit the bratio kernel itself, so expect
  smaller gains in BETA.DIST than here.
- Engine: `crates/rustsheet-engine/src/functions/statistical/gamma_kernels.rs` (coefficient arrays),
  `gammaln.rs`. Research restart: `docs/research/t1941/HANDOFF2.md` §12 on branch
  agent/research/T-1941-crack-gammaln-gamma-ln.

## Acceptance criteria
- [ ] With T-2122's sets in `gamma_kernels.rs`, `make eval` shows fewer failing cells in both probes-numeric cases
      (below 1,371 and 512), with no visibly-off GAMMALN or GAMMALN.PRECISE cell anywhere.
- [ ] No new F line in any eval/baseline/*.txt. ironcalc-calc STATISTICAL/GAMMALN_GAMMALNPRECISE.xlsx still passes
      all 91 cells, and the GAMMA, BETA.DIST and density lines only shrink.
- [ ] If T-2122 ends without a better set for a region, that region keeps today's set. The task records the
      per-region counts on the two probes in statistics.md's GAMMALN entry.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Calc!A3188 of t2464-r2-bpser-small-gl (T-3036).
- Finding the coefficients (T-2122) and LN's near-tie bit (T-2651).

## Deferred (decision 0114)
`defer: phase-1b`: every cell this task owns is already within 15 significant digits of Excel, so it waits until
Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells
visibly off (then give it a `campaign:` stage instead).
