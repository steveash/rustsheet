---
id: T-1108
title: "Reconstruct the pow Excel uses for GEOMEAN's root at near-halfway roundings"
milestone: M1
lane: research
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A documented, evidenced model of how Excel rounds the n-th root in GEOMEAN's over/underflow path,
precise enough that the engine can match the 8 near-halfway cells it misses today, or a recorded
finding that no reproducible model exists.

## Context
T-1099 found that the engine's `(m * 2^r)^(1/n) * 2^q` with glibc `powf` matches Excel on 21 of
the 29 probed over/underflow cells, and every miss is one ulp where the root `X^fl(1/n)` sits
0.39 to 0.50 ulp from a rounding boundary (Excel lands on the other side). Probes:
probes/geomean-product-order (Z6, Z7, Z8, Z10, Z11, Z13, Z14 fail) and
probes/geomean-overflow-many-values F4. Rejected models (simulated in Python against all 29
cells; see numbers.md, "GEOMEAN root rounding"): pairwise or multi-accumulator products,
decimal scaling, frexp-[0.5,1) remainders, truncated division, log-sum or exp/log2 forms of the
root, the correctly rounded root with exact or rounded 1/n, and x87-style 64-bit log/exp
compositions. Next ideas: probe single-pow behavior directly (`=(X)^(1/n)` and `POWER` on the same
X and n, to see whether Excel's `^` shares the rounding), Newton-iteration roots, and
Excel's own `exp`/`log` on the root's arguments.

## Acceptance criteria
- [ ] A probe that separates the candidate root models, filled from excel-graph, and the finding in
  `docs/excel-semantics/numbers.md`.
- [ ] Either a follow-up engine task (or the fix itself, size permitting) that makes the 8 cells
  pass with no regression, or a recorded "no reproducible model" finding that closes T-1099.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The in-range log-sum path of GEOMEAN.
