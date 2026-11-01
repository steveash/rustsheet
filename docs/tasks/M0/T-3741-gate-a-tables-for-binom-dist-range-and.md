---
id: T-3741
title: "Gate A tables for BINOM.DIST.RANGE and BINOM.INV (fresh sweep at n to 2^31 - 2)"
milestone: M0
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3444]
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
Committed, oracle-filled Gate A tables for BINOM.DIST.RANGE and BINOM.INV (and CRITBINOM), each with a cited mpmath
definition, and `make check` running them: the fresh-sweep criterion split out of T-3444.

## Context
- Split out of T-3444 (2026-11-01): T-3444's probe (probes/t3444-binom-range-inv-large-n, 756 cells, excel-graph)
  established Excel's forms, but its acceptance criterion "Gate A floor on a fresh sweep (decision 0188 R2): 0 points
  outside both 15 digits of Excel and 1E-9 of the math" needs Definitions these functions do not have yet, more than
  fits in T-3444 (as T-3553 split the discrete distributions out of T-3358).
- Add the Definitions to `tools/src/rustsheet_tools/accuracy/defs/discrete.py` beside BINOM.DIST: reuse `binom_cdf`,
  `binom_sf`, `binom_pmf` and `integer_quantile` (BINOM.INV is the smallest k with cdf(k) >= alpha, statistics.md
  "BINOM.DIST.RANGE and BINOM.INV at a large n sit on BINOM.DIST's own cdf (T-3444)"). The range needs
  number_s <= number_s2 <= trials in its strata. Generate with `cd tools && uv run rustsheet-accuracy generate
  <FUNCTION>`, fill with `uv run rustsheet-oracle fill -b excel-graph --suite accuracy`, check with
  `rustsheet-eval accuracy` (SPEC-0029).
- A failing point is carried by a `known-failures.tsv` line naming its owner task, only for points added in the same
  diff (T-3379); never by weakening ε or the 15-digit rule. BINOM.DIST's own last digits are T-3447; the upper-tail
  range rows T-3740.

## Acceptance criteria
- [ ] Gate A tables for BINOM.DIST.RANGE and BINOM.INV, oracle-filled (excel-graph), with cited definitions.
- [ ] 0 points outside both 15 digits of Excel and 1E-9 of the math, or each such point carried by a known-failures
      line naming an open owner task added in the same diff.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing BINOM.DIST.RANGE or BINOM.INV (T-3740, T-3447).
