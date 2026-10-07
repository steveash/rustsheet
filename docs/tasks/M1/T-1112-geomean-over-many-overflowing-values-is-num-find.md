---
id: T-1112
title: "GEOMEAN over many overflowing values is #NUM!: find Excel's cut"
milestone: M1
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A probed, documented rule for when GEOMEAN over many overflowing values is `#NUM!`, and the engine giving `#NUM!` exactly there.

## Context
Found in T-1108. docs/excel-semantics/numbers.md, "GEOMEAN over about 1100 overflowing values is #NUM!":
- These are `#NUM!` in Excel: probes/geomean-root-models Fallback!D1-D2 (1100 x 5E+204 or 3E+200, then one
  half-size value) and probes/geomean-overflow-count D1 (1100 x 5E+204).
- These are finite: 1001, 1024 and 1025 x 5E+204, 1000 x 1E+300, 1001 x 1E-300 and 1101 x 2.
- It is neither the count alone nor the size of the product's exponent. Probe the boundary between
  1025 and 1100 copies of 5E+204, the same count with other magnitudes, and underflowing counterparts.
  Excel's scaled product may hold a bounded exponent counter.
- The equal-value ulp misses in A1-C1 of geomean-overflow-count (1001, 1024, 1025 copies) may come
  from the same mechanism.
- The engine's path is `fn_geomean`, crates/rustsheet-engine/src/functions/statistical/geomean.rs.
  Its remainder-over-1000 fallback (`product.powf(1/n) * 2^(r/n)`) is the branch these cells reach.

## Acceptance criteria
- [ ] A probe that brackets the cut, filled from excel-graph; the rule goes in numbers.md.
- [ ] The engine gives `#NUM!` for geomean-root-models Fallback!D1-D2 and geomean-overflow-count D1, with no regression.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
