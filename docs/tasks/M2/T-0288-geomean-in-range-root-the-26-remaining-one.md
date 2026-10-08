---
id: T-0288
title: "GEOMEAN in-range root: the 26 remaining one-ulp misses"
milestone: M2
lane: functions-stat
status: todo
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
refinement-probe sets are the correctly rounded root. Ideas not yet tried: a product formed in
another order or precision for n > 2, a correction applied only when |r^n - P| exceeds a bound,
and probing two-value sets (a, b) with a*b fixed to map where Excel leaves r alone.
Code: crates/rustsheet-engine/src/functions/statistical/geomean.rs.

## Acceptance criteria
- [ ] A model that fixes some of the misses with no regression over the 159 in-range cells, or a
      numbers.md note of the further models rejected.
- [ ] `make check` green; reviewers approve.

## Out of scope
The over/underflow path (numbers.md "GEOMEAN root rounding").
