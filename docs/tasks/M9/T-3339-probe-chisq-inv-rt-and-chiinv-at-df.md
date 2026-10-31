---
id: T-3339
title: "Probe CHISQ.INV.RT and CHIINV at df 1 from p = 1E-4 to 1E-16, with Excel's CHISQ.DIST at the doubles beside each quantile"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
An oracle-filled probe case gives T-3330 enough Excel points to fit where Excel's df-1 right-tail quantile lands. At
df 1, `CHISQ.INV.RT(p,1)` is the lower quantile of 1 - p (statistics.md, "CHISQ.INV.RT and CHIINV quirks"), and for
tiny p Excel lands a whole step of its double cdf away from the crossing the engine returns.

## Context
Gap triage 2026-10-31 (`make eval` on 1171772e, `python3 scripts/exit_owners.py --all-off`, excel-graph). The cells
probes-numeric/m9-legacy-chiinv Calc!A1 `CHIINV(1e-09,1)` and A2 `CHISQ.INV.RT(1e-09,1)` are Excel
37.324892889999234 and ours 37.324893106518715. That is 6E-9 relative, so they are visibly off. A 50-digit mpmath
check (gap triage):
- the true quantile is 37.32489305136233;
- the double 1 - 1E-9 is 0.99999999900000002828;
- erf(sqrt(x/2)) at ours rounds to that double, and at Excel's it rounds to 0.9999999989999999, one ulp below;
- ours minus Excel's is 2.16519E-7, which is 2^-53 over the density 5.128E-10 at the quantile: exactly one band of
  x whose double cdf is constant.

The eval/ cells at df 1 with a p small enough for that band to matter are only 1E-9 (off) and 1E-10
(`CHISQ.INV.RT(1E-10,1)` = 41.82145620298276, passing), so the landing rule (which band edge, which side, from where
Excel starts) cannot be fitted from them. Needs an oracle sweep: this task. Probe design follows the `excel-probe`
skill. Formulas are literal calls (no cell references) so `rustsheet-fitlab` can read the arguments. Excel truncates
literals to 15 significant digits (T-3146's caution); at x near 40 one band is about 1E-8 relative, so 15-digit x
literals resolve it.

## Acceptance criteria
- [ ] A spec case `probes-numeric/t3339-chisq-inv-rt-df1-tiny-p`, oracle-filled (excel-graph), no pending cells,
      about 300 literal cells:
  - `CHISQ.INV.RT(p,1)` and `CHIINV(p,1)` for p = m E-k, k = 4 to 16, m in 1, 1.5, 2, 3, 5, 7;
  - `CHISQ.INV(q,1)` with q the literal 1 - p for k = 4 to 14 (`0.999999999` for 1E-9), to check whether
    "the right tail is the left one of 1 - p" holds bit for bit where the band is wide;
  - the same p at df 1.5 (truncated to 1, a check that the rule follows the truncation) and at df 2 (the closed
    form, a control);
  - for p = 1E-6, 1E-9, 1E-12 and 1E-15 at df 1: `CHISQ.DIST(x,1,TRUE)` and `CHISQ.DIST.RT(x,1)` at x = Excel's
    quantile moved by -3 to +3 bands (band = 2^-53 over the density, computed for each p and written as a 15-digit
    literal), and at the midpoint of each band.
- [ ] docs/excel-semantics/statistics.md names the case under "CHISQ.INV.RT and CHIINV quirks" and states, from the
      filled values: from which p Excel is more than half a unit of the 15th digit off the crossing of 1 - p, and
      where its quantile sits relative to the band whose Excel cdf rounds to 1 - p (inside, at an edge, or one band
      below).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting or implementing the landing rule (T-3330).
- p near 0.999 at other df (T-3146) and the `#NUM!` edge below 2^-54 (already in statistics.md).
