---
id: T-0290
title: "GEOMEAN over/underflow root: one-ulp misses (geomean-pow2-count L4, M1, M4 and the product-order set)"
milestone: M2
lane: functions-stat
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
GEOMEAN's over/underflow root matches Excel on the one-ulp misses that remain, or numbers.md lists
the further models rejected.

## Context
numbers.md "GEOMEAN root rounding" (T-1099, T-1108): the over/underflow root is faithful, but no model
reproduces every cell. The misses are 7 of 17 in probes/geomean-product-order, plus
probes/geomean-pow2-count L4, M1 and M4, and probes/geomean-stored-3ulps-low R1 (1000 copies of the stored 1.071508607186267E+301, T-1172; the old pow2-count R1, R2 and exponent-sum-2 C11 passed after T-0289's lossless numbers) (T-1156: 3000 x 2^-1000*1.001, and 1100 and 3000 x
2^-1000*1.01, where the exponent wrap applies). The n-copies cells are the simplest evidence: one
value, an exactly known product. Ideas: apply the 2048*k wrap's power of two before taking the root
instead of after, and try the in-range findings of T-1123 (numbers.md "GEOMEAN in-range root").
Code: crates/rustsheet-engine/src/functions/statistical/geomean.rs.

## Acceptance criteria
- [x] A model that fixes some of the misses with no regression, or a numbers.md note of the models rejected.
- [x] `make check` green; reviewers approve.

## Out of scope
The in-range root (T-0288).

## Outcome
Rejected-model note, no code change. `exp((ln(M) + r*ln2)/n) * 2^q` matches all three target cells
(geomean-pow2-count L4, M1, M4). Over the whole eval it fixes 16 known failures but regresses 9
cells the current model passes, so it was not adopted. numbers.md "GEOMEAN root rounding" lists
both sets. Eval: no change. A per-cell selector between the two models is the open lead, under T-1108's root-model question.
