---
id: T-2790
title: "F.DIST density to 15 digits where d1·x is large against d2: form 1 - y without cancellation (224 t2220 sweep cells, ours up to 2.4E-10 off)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
campaign: accuracy
depends_on: []
---

## Goal
`F.DIST(x,d1,d2,FALSE)` agrees with Excel to 15 significant digits on the 224 T-2220 sweep cells below.
At each of them a 60-digit reference shows Excel within half a unit of the 15th digit and the engine
not. On these cells y = d1·x/(d1·x + d2) is above 1/2, and the engine forms 1 - y by subtraction, which
cancels. Its error grows with d1·x/d2: 1E-14 at x = 100, 1E-12 at x = 1E4 and 2.4E-10 at x = 1E6.
Excel has no such loss: its worst density error on the whole sweep is 7.8E-14.

## Context
Gap triage 2026-10-26 (`make eval` on `main` a8e23e57). No open task owns these cells. T-2219 owns 16
F cells whose loss it puts down to the Gamma-ratio prefactor growing with a + b, which is a different
root cause. T-2221 (deferred) owns the sweep's last digits. T-2586 (research) is cracking Excel's own
density formula. T-2642 (huge df) and T-2647 (tiny x) own other corners.

Engine: `f_pdf_accurate` in `crates/rustsheet-engine/src/functions/statistical/fisher.rs`. Where
statrs' direct form is finite it returns `beta_density(y, d1/2, d2/2) * y * (1.0 - y) / x` with
`y = scaled / (scaled + df2)`. `beta_density` (`incomplete_beta.rs`) then raises `(1.0 - x)` to the
power `b - 1` again. When d1·x is much larger than d2, y rounds close to 1, so `1 - y` keeps only
about log10(d2/(d1·x)) + 16 correct digits and both factors inherit the error. The exact complement is
`df2 / (scaled + df2)`, the y = 1/(1 + d1·X/d2) form that statistics.md "Incomplete beta: ..." (T-2502)
already records for F's cdf arguments. A double-precision simulation of the engine's formula with that
complement brings the worst error on the sweep grid from 1.4E-9 down to 4.0E-15. That is still not
enough everywhere, because the powers y^(a-1) and q^(b-1) and the Gamma ratio contribute too, so the
change has to be measured, not assumed.

Excel's own form (statistics.md "F density is not the beta density, R's df or a GAMMALN formula
(T-2513)") takes its x-dependence through ln(1 + r) with r = min(d1·x/d2, d2/(d1·x)), the smaller
ratio. It never forms 1 - y. That explains why Excel stays accurate at large x. Excel's exact bits stay
T-2586's problem: this task only removes the engine's own cancellation.

Evidence (probes-numeric, excel-graph; `gate = false`, so there are no baseline lines). Each cell was
checked against a 60-digit mpmath reference: the closed-form density at the double x Excel receives
(`docs/research/t2220_f_dist_sweep_stats.py` builds the same reference):

| cell | formula | Excel | ours | Excel err | ours err |
|---|---|---|---|---|---|
| t2220-f-dist-sweep-d2-1 A629 | `F.DIST(1000000,20,1,FALSE)` | 3.9398837886748487E-10 | 3.939883787716676E-10 | 7.7E-16 | -2.4E-10 |
| t2220-f-dist-sweep-d2-1 A539 | `F.DIST(1000000,10,1,FALSE)` | 3.8910816995648966E-10 | 3.891081698696332E-10 | 6.1E-16 | -2.2E-10 |
| t2220-f-dist-sweep-d2-1 A626 | `F.DIST(386000,20,1,FALSE)` | 1.6428658130243037E-09 | 1.6428658132864497E-09 | 7.3E-16 | 1.6E-10 |
| t2220-f-dist-sweep-d1-1 A173 | `F.DIST(149000,1,2,FALSE)` | 4.504210918920901E-11 | 4.504210918904454E-11 | -1.4E-16 | -3.7E-12 |
| t2220-f-dist-sweep-d1-1 A89 | `F.DIST(1000000,1,1,FALSE)` | 3.1830956787422307E-10 | 3.1830956788216106E-10 | 8.6E-16 | 2.5E-11 |
| t2220-f-dist-sweep-d2-1 A704 | `F.DIST(8530,50,1,FALSE)` | 5.038360347565045E-07 | 5.038360347516473E-07 | -2.8E-17 | -9.6E-12 |
| t2220-f-dist-sweep-pdf A740 | `F.DIST(100,20,4,FALSE)` | 4.295760082684452E-06 | 4.2957600826846765E-06 | -1.9E-16 | 5.2E-14 |

The 224 target cells (Excel within 15 digits of the reference, ours not, y > 1/2):
- probes-numeric/t2220-f-dist-sweep-d1-1 (38): A59, A65, A68, A71, A74, A80, A83, A86, A89, A155, A158,
  A161, A164, A167, A170, A173, A248, A251, A263, A335, A341, A344, A350, A353, A356, A359, A422, A425,
  A428, A434, A440, A446, A512, A524, A629, A698, A704, A806
- probes-numeric/t2220-f-dist-sweep-d2-1 (97): A59, A62, A65, A68, A71, A74, A77, A80, A83, A86, A89,
  A149, A152, A155, A161, A164, A167, A170, A173, A176, A179, A233, A236, A239, A242, A245, A248, A251,
  A254, A257, A260, A263, A266, A269, A326, A329, A332, A335, A338, A341, A344, A347, A350, A353, A356,
  A359, A416, A419, A422, A425, A428, A431, A434, A437, A440, A443, A446, A449, A506, A509, A512, A515,
  A518, A521, A524, A527, A530, A533, A536, A539, A596, A599, A602, A605, A608, A611, A614, A617, A620,
  A623, A626, A629, A665, A683, A686, A689, A692, A695, A698, A701, A704, A707, A752, A755, A761, A770,
  A773
- probes-numeric/t2220-f-dist-sweep-pdf (89): A20, A90, A110, A160, A170, A210, A220, A240, A310, A319,
  A320, A327, A330, A350, A400, A410, A419, A430, A440, A460, A490, A510, A520, A529, A530, A540, A610,
  A619, A620, A630, A649, A660, A668, A710, A718, A719, A720, A727, A729, A730, A739, A740, A747, A760,
  A768, A769, A778, A807, A809, A810, A817, A818, A819, A820, A827, A828, A829, A830, A833, A838, A839,
  A847, A848, A859, A860, A868, A869, A903, A907, A908, A909, A917, A919, A928, A929, A937, A938, A947,
  A949, A951, A958, A959, A963, A966, A967, A968, A978, A988, A989

Another 114 sweep density cells with y > 1/2 have both Excel and ours more than half a unit off the
reference (for example d1-1 A797, `F.DIST(22100,1,50,FALSE)`: Excel +2.8E-14, ours -3.7E-13). They
are not targets here. The task records how far they move.

## Acceptance criteria
- [ ] The 224 cells above agree with Excel to 15 significant digits (`target/release/rustsheet-eval show
      probes-numeric/t2220-f-dist-sweep-{d1-1,d2-1,pdf}.case.json`). A script scores them against the
      60-digit reference. Commit it under `docs/research/` or extend `t2220_f_dist_sweep_stats.py`.
- [ ] No F.DIST, F.DIST.RT, FDIST, F.TEST or BETA.DIST cell in any suite moves from bit-exact or
      15-digit-equal to visibly off. That includes the T-2586 F density probes (t2513-r3-fdensity-*,
      t2586-*), which are the bulk of the F density evidence. Ulp-level moves among cells that stay
      within 15 digits are allowed (decision 0083). The Outcome gives counts per case before and after,
      plus the 114 "both off" cells' error before and after.
- [ ] `make eval-check` shows 0 regressions in gated suites. ironcalc-calc F_DIST.xlsx F3
      (`F.DIST(0.7,4,2.4,FALSE)`) stays where it is: y = 0.538 there, so the change of form reaches it.
- [ ] statistics.md's "F density is not the beta density ..." entry, in its "Ours:" line, says where
      the engine now forms the complement directly, with these cells as evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Excel's own density formula and its last bits (T-2586 research, T-2221 deferred).
- The Gamma-ratio prefactor loss at large a + b (T-2219), huge df (T-2642), tiny x (T-2647).
- The cdf and upper tail (the bratio port, T-2504 to T-2511).

## Visible-parity stage
Decision 0083: this task is done when every cell it owns (the 224 above) is within half a unit of the
15th significant digit of Excel's value, with no cell newly visibly off. Bit-level matches are not
required.

## Stage (decision 0114)
`campaign: accuracy`: a 60-digit reference shows Excel within 15 digits on these cells and the engine
not. Aim for an accurate kernel scored against the reference and the oracle. Do not fit Excel's last
bits. Cells left within 4 ulp afterwards go to a `defer: phase-1b` follow-up.

Tier: opus. The complement fix alone leaves 4E-15 errors on the grid. Reaching 15 digits on all 224
cells without moving any of the roughly 66,000 F density probe cells is accuracy work at the ulp level,
measured against a reference.
