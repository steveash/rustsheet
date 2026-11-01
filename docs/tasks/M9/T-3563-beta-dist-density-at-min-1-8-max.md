---
id: T-3563
title: "BETA.DIST density at min < 1 < 8 < max: the 132 visibly-off t3383-r4-algdiv cells, from Excel's algdiv lnB"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3569]
campaign: visible
---

## Goal
`BETA.DIST(x,a,b,FALSE)` with min(a, b) < 1 < 8 < max(a, b) is within half a unit of the 15th significant digit of
Excel on the 132 visibly-off cells below. The fix takes the density's lnB from T-3569's model of Excel's algdiv. The
cells are a few units past the 15th digit (6E-16 to 1.9E-15 relative). They are visibly off only because of the
cut, but they count in the M9 probes-numeric row and against BETA.DIST's Phase 1 rate.

## Context
- Gap triage 2026-11-01 (`make eval` on b98b3c00d, `scripts/exit_owners.py --all-off`): these are the only visibly-off
  BETA.DIST cells with no open owner, deferred or not. 131 sit in the hold-out case at a = 0.75 over ten b from 20.6
  to 7000, and one in the in-sample case at (0.5, 711.406).
- Examples (Excel vs ours):
  - t3383-r4-algdiv-hold D1096 `BETA.DIST(0.00029871520097802168,0.75,3908.43,FALSE)`: 954.7840819759201 vs
    954.7840819759209;
  - t3383-r4-algdiv-hold D1055 (x 0.012964170899592254, same shape): 8.586061897833868E-20 vs 8.586061897833876E-20;
  - t3383-r4-algdiv-hold D151 (x 0.22541738244076043, b 20.613): 0.07610161716548895 vs 0.07610161716548888;
  - t3383-r4-algdiv-a D843 (x 0.0002126070177298964, a 0.5, b 711.406): 887.1908992352588 vs 887.1908992352594.
- Against 50-digit math (mpmath on the exact two-piece literals, this triage), over the 132 cells Excel is closer
  than ours on 70 and ours is closer on 62. The median relative error is 1.05E-15 for Excel and 1.01E-15 for ours.
  Neither side is correctly rounded, so this is `visible`, not `accuracy`: the target is Excel's own lnB.
- Root cause (T-3383, docs/research/t2464/r4-algdiv/RESULTS.md; statistics.md "Beta density for min(a,b) <= 1"):
  with one free lnB per shape, T-2513's model M2 reproduces Excel on 78 of 80 shapes of each probe. So the whole
  density gap here is the double lnB = fl(GL(min) + algdiv(min, max)). Excel's pinned lnB is the correctly rounded
  value -4 to +3 ulps off, with no structure in b. TOMS algdiv in doubles, the engine's today, is 2,008 and 1,696
  of 4,000 rows bit-exact. At these shapes |lnB| is about 3 to 9, so each ulp of lnB is about 1E-15 relative in the
  density.
- Needs the model first. T-3569 (research; T-3468's item 2, split out) scores x87 storage variants of TOMS algdiv
  per shape against the 156 pinned lnB values, in-sample a first and the hold-out once at the end, and spot-checks
  the winner on desktop Excel. Without it there is nothing to port: tuning lnB per shape against these cells is the
  keep-matched branch decision 0056 forbids.
- Code: `beta_density.rs` (the M2 exponent path and its lnB), bratio/ln_beta.rs (algdiv), in
  crates/rustsheet-engine/src/functions/statistical/.
- Sibling tasks: T-3528 (the mirror region b < 1 < 8 < a, on its own probe, deferred) and T-3564 (the
  bit-level cells of these two cases, deferred).

Cells (the visibly-off cells of the two cases; T-3564 owns the rest of each case):
- probes-numeric/t3383-r4-algdiv-hold.case.json (131): Calc!D151, D164, D175, D182, D186, D194, D651, D658, D664:D667,
- probes-numeric/t3383-r4-algdiv-hold.case.json (cont.): D675, D684, D688, D694:D696, D752, D754:D755, D759:D760, D763,
- probes-numeric/t3383-r4-algdiv-hold.case.json (cont.): D768:D769, D771:D772, D777:D779, D781, D793, D800:D802, D804,
- probes-numeric/t3383-r4-algdiv-hold.case.json (cont.): D807:D810, D814, D816:D819, D821, D825, D831:D832, D838, D842,
- probes-numeric/t3383-r4-algdiv-hold.case.json (cont.): D844, D848:D850, D953, D955, D961:D962, D964, D967, D970,
- probes-numeric/t3383-r4-algdiv-hold.case.json (cont.): D974, D988, D1055, D1058, D1064, D1067:D1068, D1074, D1078,
- probes-numeric/t3383-r4-algdiv-hold.case.json (cont.): D1080, D1089:D1090, D1093, D1096:D1097, D1104:D1105,
- probes-numeric/t3383-r4-algdiv-hold.case.json (cont.): D1107:D1108, D1112, D1118, D1121:D1123, D1132, D1136, D1138,
- probes-numeric/t3383-r4-algdiv-hold.case.json (cont.): D1148, D1160, D1162:D1163, D1167, D1173, D1177:D1178, D1184,
- probes-numeric/t3383-r4-algdiv-hold.case.json (cont.): D1192, D1197, D1205, D1208, D1213, D1218, D1224, D1227:D1228,
- probes-numeric/t3383-r4-algdiv-hold.case.json (cont.): D1232, D1236, D1238, D1243, D1801, D1805:D1806, D1808, D1816,
- probes-numeric/t3383-r4-algdiv-hold.case.json (cont.): D1818:D1821, D1823:D1825, D1830, D1832:D1833, D1837,
- probes-numeric/t3383-r4-algdiv-hold.case.json (cont.): D1839:D1840, D1850
- probes-numeric/t3383-r4-algdiv-a.case.json (1): Calc!D843

## Acceptance criteria
- [ ] The engine's lnB for min < 1 < 8 < max is T-3569's model, ported, with the model's counts quoted. On the 132
      cells above, each is within half a unit of Excel's 15th digit (`make eval` per-case deltas in the Outcome), or
      is listed in statistics.md with the model's value and why it misses.
- [ ] Decision 0188 R4 guards (reviewer-excel-fidelity check 7), in the Outcome: hold-out counts (bit-exact, within
      4 ulp, within 15 digits) for the chosen lnB model and for the correctly rounded lnB, by T-3569's split
      (the in-sample and hold-out cases above, fixed before scoring); a fresh probes-numeric sweep
      over min < 1 < 8 < max filled after the model is frozen, with its result; the Gate A envelope result on that
      sweep, with any point outside it listed; each fitted piece tagged `FITTED(T-3563, n_cells, range)` and applied
      only inside min < 1 < 8 < max, with accurate math outside; no target below the reproducibility floor (the pins'
      -4 to +3 ulp offsets are inside the 4-ulp web/desktop floor, so a model that only reaches the floor is
      recorded, not forced); and no per-shape or keep-matched branch.
- [ ] No cell of any suite goes from within 15 digits to visibly off, and gated suites lose no bit-exact cell, or a
      regression decision record covers the loss (decision 0082). Main-against-HEAD deltas in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The bit-level cells of the same cases (T-3564), the mirror region (T-3528), and a = 1 or b = 1, where lnB is
  -LN(the other shape).

## Visible-parity stage
Decisions 0083 and 0114 (`visible`): done when every cell this task lists is within half a unit of the 15th
significant digit of Excel's value, or is an UNVERIFIED entry with its ruled-out hypotheses. The bit-level residue is
T-3564 (`defer: phase-1b`).
