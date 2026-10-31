# T-3383 r4-algdiv: Excel's algdiv for b0 >= 8 in the beta density (item 2)

Worker r4-algdiv, 2026-10-31. Item 2 of T-3383 (open since T-2464), through the density only. The density's lnB for
min(a, b) < 1 < 8 < max(a, b) is the only open piece of T-2513's model M2 (statistics.md "Beta density for
min(a,b) <= 1"). Counts are bit-exact rows unless stated otherwise. Run with a Python that has mpmath, numpy and scipy.

## Probes (built and filled together, before any scoring)
| case | rows | shapes |
|---|---|---|
| `t3383-r4-algdiv-a` (in-sample) | 4,000 | a = 0.5 and 0.25; b = 24 geometric points on [8.25, 5000] and 16 fine steps 16 + k/8 |
| `t3383-r4-algdiv-hold` (hold-out) | 4,000 | a = 0.75 and 0.4; 24 other geometric b on [8.6, 7000] and 16 fine steps 40 + k/4 |
| `t3383-r4-algdiv-gl` | 203 | Excel's GAMMALN at every a, b and a + b of both probes |

50 x per shape (r3-m1-residual's `xs_for`), with exact two-piece literals read back from A:C (no row was dropped by the
15-digit literal cut). `mkalgprobe.py`, `mkgl.py`.

## 1. With one free lnB per shape, M2 reproduces Excel (`scan.py`, `scan_a.out`, `scan_hold.out`)
For each shape, M2's exponent path is scored with L = TOMS algdiv in doubles (r2-bratio's `lnb_base`) moved by j ulps,
|j| <= 16:
| probe | TOMS-d (j = 0) | best j per shape | shapes at 100%, unique j |
|---|---|---|---|
| a | 2,008 of 4,000 | 3,956 | 78 of 80 |
| hold | 1,696 of 4,000 | 3,952 | 78 of 80 |

The 4 shapes that do not pin: (0.25, 176.694), where lnB is about 1e-14 (its ulp is tiny, 16 j tie); (0.4, 8.6),
whose pin lies at the scan's edge (j = -16, 3 of 50; unexplained, T-3468 rescans it wider); and (0.5, 16) and
(0.4, 43.75) at 49 of 50. This confirms
T-2513's finding on a fresh sweep at four new a: **for min < 1 < 8 < max the whole density gap is the double lnB; tx,
ty, the split exponent and EXP are right.**

## 2. The pinned lnB is the accurate value plus noise of a few ulps, with no structure in b (`dist.py`, `dist.out`)
Pinned minus correctly rounded lnB, in ulps of lnB, over the 156 pinned shapes:
- probe a: -4: 1, -3: 3, -2: 5, -1: 12, **0: 33**, +1: 16, +2: 6, +3: 2.
- hold: -3: 1, -2: 6, -1: 15, **0: 35**, +1: 16, +2: 3, +3: 1, +6: 1.

The implied algdiv, fl(pinned lnB - GL(a)), lies within -1.7 to +2.2 ulp(algdiv) of the accurate
ln Γ(b) - ln Γ(a+b) on every shape. TOMS algdiv in doubles is within -1.6 to +1.3 ulp(algdiv) of it too, so the two
differ by a unit or two in either direction. Along the fine b steps (spacing 1/8 and 1/4) the offset has
lag-1 autocorrelation -0.20, -0.12 (probe a) and +0.21, -0.24 (hold), on 15-16 points per a (standard error
about 0.26). **No large smooth error is visible.** (An approximation error of the series or of the Stirling
difference moves continuously with b.) The likely cause is rounding noise from a few stored intermediates, which
changes from one b to the next. This answers T-3383's "next" for item 2 as an inference, not a proof: the offset
looks like a storage pattern, not an approximation, until a storage model reproduces the pins (T-3468).

## 3. Fixed candidates, no per-shape freedom (`cands.py`, `cands_a.out`, `cands_hold.out`)
Shapes whose pinned lnB the candidate equals (and rows):
| candidate | probe a | hold |
|---|---|---|
| TOMS algdiv in doubles (`lnb_base`, the frozen choice) | 26 (2,008) | 30 (1,696) |
| correctly rounded lnB | **34** (2,376) | **37** (2,062) |
| fl(GL(a) + CR(algdiv)) | 22 (1,862) | 35 (1,919) |
| GL form carried past 8: fl(GL(a) + fl(GL(b) - GL(a+b))) | 0 (18) | 1 (62) |
| fl(fl(GL(a) + GL(b)) - GL(a+b)) | 0 (25) | 1 (51) |

Negatives: Excel's GAMMALN difference is not the density's lnB past 8, in either association (0 and 1 of 80 shapes;
the cancellation of GL(b) - GL(a+b) costs many ulps). The correctly rounded lnB is the best single candidate on both
probes, but it gets fewer than half of the shapes. No candidate is frozen, so there is no fit and no `FITTED` tag.
The engine keeps TOMS-d until a model reaches most shapes. The hold-out is used only to show that no candidate
reaches a majority on either probe.

## 4. Next (for the follow-up)
- The pins are Excel for the web values (excel-graph). Pinned offsets of -4 to +3 ulp are inside the documented
  web/desktop floor of up to 4 ulp (docs/research/oracle-calibration.md), so a model frozen from them needs a
  desktop spot-check first.
- The noise is the rounding of TOMS algdiv's stored intermediates, or of some other routine with the same
  accuracy. Its sign and size change at every b. The next probe is therefore x87 storage variants of algdiv scored
  against these 156 pinned lnB values, *per shape*, as r3-m1-residual's `lnbfit.py` does for its own 52 shapes.
  `scan_<name>.json` holds the pins (a, b, n, hits at j = 0, best j, best hits, ties, accurate j, ...). The 3,072
  variants of r3-m1-residual's `algsearch.py` were scored on rows, not on pins, and against fewer shapes.
- Reference: the accurate algdiv needs a + b summed exactly. Excel's GAMMALN(a + b) receives the double sum, but the
  accurate value does not (an earlier version of `scan.py` summed in doubles and showed false offsets of hundreds of
  ulps at a = 0.4 and 0.75).
