---
id: T-3528
title: "BETA.DIST density at b < 1 < 8 < a: the bit-level residue on probe t2512-r3-bgrat-bupdens (4,838 cells within 15 digits, 29 beyond 4 ulp)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-3383]
defer: phase-1b
---

## Goal
BETA.DIST(x, a, b, FALSE) at a shape of about 0.7 against about 30.7 (both orientations), the points of probe
t2512-r3-bgrat-bupdens, takes Excel's double wherever a structural model of Excel's density lnB does so. In
particular, the 29 cells 7 to 10 ulp from Excel's double reach 4 ulp or better. Cells within 4 ulp are not targets
unless the same model fixes them.

## Context
- Gap triage 2026-10-31 (`make eval` on origin/main 73241cb4c, excel-graph; `scripts/exit_owners.py`'s `owned` rule
  over every failing cell and the task files of origin/main and every open task-filing branch): T-2512 committed the
  probe (5,816 cells, docs/research/t2464/r3-bgrat/RESULTS.md table, `make_probe3.py`) to measure bup's prefactor. It
  went to research only, and no open task names it. Today 978 cells are bit-exact. The other 4,838 are 1 to 10 ulp off
  (median 3), none visibly off.
- The probe holds x runs and shape runs of consecutive doubles around (x, a, b) = (0.8117, 30.7, 0.7) in column A and
  the mirrored (y, b, a) in column B: 2,419 failing cells each. Examples, Excel vs ours:
  - A1 `BETA.DIST(0.8117,30.699999999999,0.7,FALSE)` 0.028367050675885373 vs 0.028367050675885387
  - A97 `BETA.DIST(0.8117,(128765132*2^-22+53686975*2^-48),0.7,FALSE)` 0.02836705067588226 vs
    0.02836705067588229 (8 ulp)
  - B1345 `BETA.DIST((25273198*2^-27+12240173*2^-53),0.7,30.7,FALSE)` 0.02836705067593831 vs 0.028367050675938345
    (10 ulp)
  The 29 cells beyond 4 ulp: A68, A89, A97, A193, A201, A246, A254, A255, A300, A305, A314, A407, A474, A725, B68,
  B89, B97, B193, B201, B246, B254, B255, B300, B305, B314, B407, B474, B725, B1345.
- Suspected root cause: the density's lnB for min < 1 < 8 < max. M2 takes fl(GL(min) + algdiv(min, max)), and Excel's
  algdiv is -8 to +6 ulps from TOMS algdiv in doubles (T-3383 item 2; T-2513 round 3 r3-m1-residual, `lnbfit.py`).
  RESULTS.md finding 2 also says the two orientations' densities differ by up to ±8 ulp on 1,355 of 2,908 points
  (`orient.out`), so the x-dependent part (LN or log1p by x) is in play too.
- Related: T-2821 owns the min(a,b) <= 1 density residue on the probes it lists, not this one. T-3382 uses this
  density as bup's prefactor in the L140 cdf, so a fix here moves T-3382's W term.
- Code: crates/rustsheet-engine/src/functions/statistical/beta_density.rs (`beta_density_small_shape`), bratio/.
- Cells: the whole case. Every failing cell of probes-numeric/t2512-r3-bgrat-bupdens is owned here.

## Acceptance criteria
- [ ] Each of the 29 cells beyond 4 ulp listed above is within 4 ulp of Excel's double, or the Outcome lists it with
      Excel, new and 50-digit values and the reason.
- [ ] The probe's bit-exact count does not drop below 978, and no cell of it or of any other BETA.DIST, F, T, BINOM or
      NEGBINOM case moves from within 15 digits to visibly off (`make eval` per-case counts in the Outcome). No new
      known-failure line in any baseline (`make eval-baseline`).
- [ ] Any fit follows decision 0188 R4 (reviewer-excel-fidelity check 7), each guard shown in the Outcome:
      hold-out counts (train and hold-out side by side, bit-exact / 4 ulp / 15 digits, chosen candidate against the
      correctly rounded one, margin stated before scoring); a fresh sweep of other (x, a, b) points with
      min < 1 < 8 < max oracle-filled after the freeze (this probe sits at a single shape pair, so it cannot be its own
      hold-out); the Gate A envelope result on that sweep with every point outside listed; each fitted piece tagged
      `FITTED(T-3528, n_cells, range)` and applied only inside its range; no step fits differences below the
      reproducibility floor (4 ulp web against desktop, docs/research/oracle-calibration.md); no keep-matched branch.
- [ ] statistics.md's "Beta density for min(a,b) <= 1" entry records the result.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The algdiv model itself (T-3383 item 2, research). This task ports what that finds.
- The L140 cdf composition (T-3382).

## Deferred (decision 0114)
`defer: phase-1b`: every cell this task owns is already within 15 significant digits of Excel, so it waits until
Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells
visibly off (then give it a `campaign:` stage instead).
