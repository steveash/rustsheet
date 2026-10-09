---
id: T-0288
title: "GEOMEAN in-range root: the 26 remaining one-ulp misses"
milestone: M2
lane: functions-stat
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Find the root model that matches Excel on the in-range GEOMEAN cells still one ulp off after
T-1123: probes/geomean-in-range-many A2 (= geomean-single-and-copies Copies!A35) and 25 of the
48 sets of probes/geomean-root-refinement.

## Context
numbers.md "GEOMEAN in-range root" lists the evidence and every rejected model. Since T-1123 the
engine uses `r = exp(ln(P)/n)`, snapped to its 15-digit value when that value multiplied n times
gives P back. Excel's LN and EXP are glibc's. Excel is faithful but not always correctly rounded
(GEOMEAN(1,3) and GEOMEAN(1,2,5,1) are r, 0.55 and 0.81 ulp from the exact root), while most
refinement-probe sets are the correctly rounded root. The clearest lead: GEOMEAN(0.01,1) is exactly 0.1 (geomean-root-refinement A7, A8) although
0.1*0.1 != 0.01 in doubles, so Excel's real condition is not the snap's. Ideas not yet tried: a product formed in
another order or precision for n > 2, a correction applied only when |r^n - P| exceeds a bound,
and probing two-value sets (a, b) with a*b fixed to map where Excel leaves r alone.
Code: crates/rustsheet-engine/src/functions/statistical/geomean.rs.

## Acceptance criteria
- [x] A model that fixes some of the misses with no regression over the 159 in-range cells, or a
      numbers.md note of the further models rejected.
- [x] `make check` green; reviewers approve.

## Out of scope
The over/underflow path (numbers.md "GEOMEAN root rounding").

## Outcome
No model fixes misses without breaking cells r matches, so the engine is unchanged and numbers.md "GEOMEAN in-range
root" records the new rejected models. Scored on 167 cells (harness: the in-range probes plus ironcalc C23/G27), the
current r with the snap gets 140. glibc `pow(P,1/n)` with the snap gets 157 but breaks C23, G27 and refinement A20.
Nearest-power neighbour searches get 152 or 155, and the `exp(ln P*(1/n))`, log10 and log2 forms 104 to 149. The new
probe probes/geomean-same-product (23 cells, excel-graph 2026-10-09) shows the root depends only on the double product P
and n: every factor order and split of P = 3, 10.704 and 10 gives the same value. C23 (1,3) and refinement A36
(6.69,1.6) have the same n, residual pattern and near-identical r position, yet Excel rounds them differently. So the last
bit comes from Excel's own (non-glibc) root, which no double formula tried reproduces. Side lead for T-0221: Excel's
`P^(1/n)` cells match `exp((1/n)*ln P)` on 32 of 35. Eval delta: probes +1 case, 23 cells, 15 pass, 8 new-coverage known
failures (the existing A36/A29 one-ulp class); fixed 0, regressions 0.
