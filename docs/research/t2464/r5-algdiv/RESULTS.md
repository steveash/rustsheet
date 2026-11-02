# T-3569 r5-algdiv: x87 storage variants of algdiv (b0 >= 8) against the r4-algdiv pins

Worker r5-algdiv, 2026-11-01. This tests whether a storage variant of TOMS algdiv explains the pinned density lnB
for min(a, b) < 1 < 8 < max(a, b). Inputs: r4-algdiv's per-shape pins (`../r4-algdiv/scan_a.json`,
`scan_hold.json`; Excel for the web values), r3-m1-residual's `algdiv_v.py` arithmetic switches and `m1lib`.
Run with a Python that has mpmath, numpy and scipy (bindings/python/.venv; README.md).

**Result: negative.** No x87 storage variant of algdiv reproduces most pinned shapes. The best frozen tier gets 36 of 78
shapes in-sample and 33 of 78 on the hold-out. That is no better than the correctly rounded lnB (33 and 35) and only a
little better than TOMS in doubles (26 and 30). Nothing is frozen, so there is no fit and no `FITTED` tag. The engine
keeps TOMS-d.

## Method (`score.py`)
- **Pins.** These are the shapes r4-algdiv pinned uniquely: all rows reproduced at one j and no tie. There are 78 of 80
  per probe. The pinned lnB is `nb(lnb_base(a, b), j)`. A variant scores one point per shape whose double lnB equals
  the pin exactly, per shape and not per row, as T-3569 asks.
- **Grid (65,792 variants).** It contains `ar` = all-double, all-x87-stored, and extended with every subset of the
  nine named stores (h c x d s w u v r; 512 subsets). Each is crossed with alnrel {TOMS (3,3) rational, r2-log1p's
  series, correctly rounded, LN(1 + q)}, log b {Excel's LN, FYL2X extended}, the final order {TOMS's u > v switch,
  (w - u) - v, (w - v) - u, w - (u + v)}, t² {(1/b)², 1/(b·b)} and the GL(a) + algdiv sum {stored, extended}. This is
  r3-m1-residual's 3,072-variant `algsearch.py` grid widened to all nine stores, t2 and the fourth order.
- **Positive control** (`score.py ctl`, `ctl.out`). The lnB of one variant (stores d w u, TOMS alnrel, FYL2X log, TOMS
  order) was planted as the pins on probe a's 78 shapes and scored over the 4,112-variant sub-grid that shares its
  alnrel, t2 and sum. It ranks first with 78 of 78. 256 variants tie with it, because many store choices round to
  the same double, so the method can rank families but not separate stores that make no difference.
- **Hold-out discipline.** The top tier was frozen on probe a (`score_a.json`: all 4,096 variants at the best
  score). Only that tier was then scored on the hold-out, once (`score.py hold --top 4096`, so the
  whole tier and not the default top 25).

## In-sample (probe a, `score_a.out`)
- 78 pinned shapes; TOMS-d 26.
- Histogram of variant scores: 36: 4,096; 35: 1,024; 34: 8,192; 33: 1,024; 32: 6,144; 31: 13,312; 30: 13,312; 28 and
  lower: 2,192.
- The 36 tier is every variant that stores u (= d·alnrel(a/b)) to double, uses the series alnrel and uses Excel's LN
  for log b. Its other switches make no difference on these shapes.

## Hold-out (`score_hold.out`)
- 78 pinned shapes; TOMS-d 30. Of the frozen 36-tier, 1,024 variants score 33, 1,024 score 31 and 2,048 score 28.

## Exact, within-4-ulp and within-15-digit counts per shape (`counts.py`, `counts.out`)
| candidate | a: exact | a: <= 4 ulp | a: 15 digits | hold: exact | hold: <= 4 ulp | hold: 15 digits |
|---|---|---|---|---|---|---|
| best storage variant (e, st=u, ser, xl, toms, sq, d) | 36 | 77 | 75 | 33 | 77 | 70 |
| TOMS algdiv in doubles (engine) | 26 | 76 | 68 | 30 | 77 | 69 |
| correctly rounded lnB | 33 | 78 | 73 | 35 | 77 | 71 |

(r4-algdiv's correctly rounded counts were 34 and 37 of 80 shapes, which include the two shapes left unpinned here.)

## Reading
The density's lnB is within 4 ulp of every candidate on all but one or two shapes. Exact agreement stays below half
for every arithmetic of TOMS algdiv tried, and every storage pattern stays below half too. Storing u is the only
switch that helps, and it adds 10 shapes in-sample but 3 on the hold-out, about what tuning over 65,792 variants would
give. This supports r4-algdiv's reading that the pins are the accurate value plus a few ulps of noise. It does not
show the noise comes from rounding inside TOMS algdiv. The pins are Excel for the web values, and their -4 to +3 ulp
spread is inside the web/desktop floor of up to 4 ulp (docs/research/oracle-calibration.md). So the next useful step
is desktop Excel cells for the same shapes (M14), not more web-pin scoring. The bpser L check that T-3569 planned
for a winner was not run, because no variant won.

Not done here: the wider rescan of (0.4, 8.6), whose r4 pin is at the scan's edge (j = -16). It would need a new
probe, and the result above does not depend on it. No new oracle probes were made: this round reuses r4-algdiv's.
