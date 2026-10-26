---
id: T-2794
title: "GEOMEAN in-range root: replace exp(ln P/n) where it is not faithful (16 gen-m9 cells, 2 visibly off)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
GEOMEAN's in-range root is within one ulp of the exact n-th root of the product, as Excel's is, for products far from 1.
The two visibly-off cells below reach 15-digit parity, and the other 14 cells of this list move to Excel's value or
within one ulp of it, with no GEOMEAN cell that passes today regressing.

## Context
Gap triage of the T-2676 sweep (gen-m9 round 2, excel-graph, 2026-10-26). When the plain product P is a normal double,
`fn_geomean` (crates/rustsheet-engine/src/functions/statistical/geomean.rs) returns `r = exp(ln(P)/n)` (with the
15-digit snap of numbers.md "GEOMEAN in-range root"). The rounding error of `ln(P)/n` is multiplied by the size of
`ln(P)/n` when `exp` is applied, so for a product far from 1 r is several ulps from the exact root: up to 14 ulps here,
and 1.7 units of the 15th digit at A371. Excel stays faithful on every cell (at most 0.94 ulp from the exact root, and
correctly rounded on 13 of the 16), so this is an accuracy fix, not a fit (docs/excel-semantics/numbers.md "GEOMEAN in-range
root", the paragraph on products far from 1).

Each cell's P and n were read with PRODUCT and COUNT of the same arguments on the case's Data sheet (engine), and the
exact root of that double P was taken to 60 digits. The engine's inputs and Excel's agree (Excel's value is within one
ulp of that root in every cell). Distances are in ulps of Excel's value (Excel / ours, from the exact root):
- gen-m9/290180acf017709c (9): A16 P=4E+15 n=4 (+0.34 / +5.34); A156 P=8100 n=5 (-0.22 / -1.22); A191 P=2E+15 n=2
  (+0.13 / +2.13); A236 P=1E+15 n=2 (-0.43 / -5.43); A241 P=2.5125E+28 n=4 (-0.41 / +11.59); A281 P=1.02E+26 n=4
  (-0.53 / +1.47); A341 P=1.57E-13 n=4 (-0.06 / -1.06); **A371** `=GEOMEAN(ROUND(2^53+1,0),(SECOND(3.14159)-(Data!$A3+Data!B$6)),COLUMNS(Data!A11:B11))`
  P=9.5476E+17 n=3 (-0.45 / -14.45; Excel 984687.7727545286, ours 984687.7727545269, visibly off); A396 P=5E-16 n=2
  (+0.12 / -2.88)
- gen-m9/f81bd3b6dc373e5c (7): A24 P=1.037E+10 n=3 (-0.05 / +2.95); A34 P=7.407E+07 n=3 (-0.26 / +3.74); A54
  P=4.844E+22 n=7 (-0.60 / +3.40); **A139** `=GEOMEAN(Data!B11:B11,{0.02,0.027},LARGE((Data!E2*Data!A$6),1))` P=1.62E+11
  n=3 (+0.02 / -5.98; Excel 5451.361778496419, ours 5451.3617784964135, visibly off); A204 P=9.145E+17 n=5 (+0.82 /
  +1.82); A234 P=9.191E+17 n=6 (-0.15 / -4.15); A299 P=7.407E+09 n=4 (-0.22 / -1.22)

A suggested kernel, to be checked against every GEOMEAN probe: keep r (and the snap) when it is within one ulp of the
exact root (an error-free check, e.g. r^n against P in double-double), otherwise return the correctly rounded root.
Since Excel is faithful on every probed in-range cell (numbers.md), a cell that passes today has r within one ulp, so
that rule leaves it unchanged; on this list it gives Excel's bits on 13 cells and one ulp on A281, A54 and A204. The
one-ulp cells where r is already faithful but on the other side from Excel (12 more cells of the same sweep) are
T-1284's, not this task's.

## Acceptance criteria
- [ ] gen-m9/290180acf017709c A371 and gen-m9/f81bd3b6dc373e5c A139 are within half a unit of the 15th significant
      digit of Excel's value (`make eval` no longer lists them as visibly off).
- [ ] At least 13 of the 16 known-failure lines above disappear from eval/baseline/gen-m9.txt, with no new line in any
      baseline (`make eval-baseline` reports 0 regressions; ironcalc-calc GEOMEAN C23 and G27 and the
      probes/geomean-* cells keep passing).
- [ ] The cells still failing are listed in this task's Outcome with their ulp distance (each at most 1 ulp).
- [ ] numbers.md "GEOMEAN in-range root" says which kernel ours uses for these products.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The over/underflow path (T-1099, T-1108) and the one-ulp misses where r is faithful (T-1284).

## Visible-parity stage
`accuracy`: Excel is within one ulp of the exact root and ours is up to 14 ulps off; the task is done when A371 and A139
are within 15 digits and no listed cell is more than one ulp from Excel.
