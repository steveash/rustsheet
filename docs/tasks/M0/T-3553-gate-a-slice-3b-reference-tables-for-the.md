---
id: T-3553
title: "Gate A slice 3b: reference tables for the discrete distributions (BINOM.DIST, NEGBINOM.DIST, POISSON.DIST)"
milestone: M0
lane: harness
status: done
priority: P0
size: M
tier: opus
depends_on: [T-3357, T-3379]
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
Committed, oracle-filled Gate A tables for BINOM.DIST, NEGBINOM.DIST and POISSON.DIST (both the cumulative and the
mass forms), each with a cited mpmath definition, and `make check` running them.

## Context
- Split out of T-3358 (2026-10-31): that task named 18 functions in one batch, more than an M task, and only T.DIST,
  T.INV and RATE had definitions (T-3357). T-3358 keeps the continuous distributions and quantiles.
- SPEC-0029 ("The generator and the sampler", "Point fields", "The check"); decision 0188 R2 and R3. Generate with
  `cd tools && uv run rustsheet-accuracy generate <FUNCTION>` after adding a `Definition` under
  `tools/src/rustsheet_tools/accuracy/defs/`, fill with `uv run rustsheet-oracle fill -b excel-graph --suite accuracy`,
  and check with `rustsheet-eval accuracy`.
- A failing point is carried by a `known-failures.tsv` line naming its owner task, only for points added in the same
  diff (T-3379); never by weakening ε or the 15-digit rule.
- After the definitions land, rerun `uv run rustsheet-accuracy math --suite probes-numeric` (T-3380) and commit the
  sidecars, so R1(c) sees these functions' probes-numeric cells.
- The decision 0188 assessment found BINOM.DIST at n ≥ 6e5 and NEGBINOM.DIST at a large number of failures 1e-9 to
  1e-8 off (T-3352 owns them); POISSON.DIST and BINOM.DIST generalize poorly to fresh points (T-3366). Their failing
  points are carried with those owners.

## Acceptance criteria
- [x] Definitions for the three functions, unit-tested against published values; tables generated, filled and
      committed; each failing point is named with its owner task in the Outcome.
- [x] The probes-numeric math sidecars are regenerated with the new definitions and committed.
- [x] `make check` green; reviewers approve.

## Out of scope
- The fixes for failing points (their owner tasks).

## Outcome
Done (2026-11-01). Renumbered from T-3543, an id collision with main (decision 0215).
- **Definitions** (`tools/src/rustsheet_tools/accuracy/defs/discrete.py`):
  - BINOM.DIST, NEGBINOM.DIST and POISSON.DIST, both forms, from the regularized incomplete beta and gamma.
  - Each upper tail is taken directly, so a 1E-300 tail keeps its digits.
  - Excel's domain rules: trials below 2^31 - 1; the pmf at k = 0 with p = 1 is `#NUM!`; POISSON at x = 0 with a
    negative mean is EXP(-mean).
  - The tail stratum sets the count from its integer quantile (doubling then bisection).
  - Tests against Microsoft's published examples are in `tools/tests/test_accuracy_discrete.py`.
- **Generator:** a count has no Excel upper limit, so its domain is unbounded.
  - A first draft bounded the counts at 1E+15, so its edge points expected `#NUM!` where Excel computes.
  - `Param.draw_hi` now caps whole-domain draws (1E+12) without adding edge points. mpmath hangs, fails to converge or
    returns a wrong 0 at a count of 1E+300.
- **Tables:** `s1` batches, filled from excel-graph:
  - BINOM.DIST, 197 points (39 domain errors);
  - NEGBINOM.DIST, 197 points (10);
  - POISSON.DIST, 120 points (15).
- **Check:** all 514 points are inside the envelope, with 0 failing and 0 table defects. No known failures are added,
  and `rustsheet-accuracy verify` shows 0 differences. The T-3352 and T-3366 weaknesses did not show on these points.
- **probes-numeric math sidecars:** 22 cases, 40,464 cells, regenerated with the new definitions.
- **Excel findings** (statistics.md):
  - "NEGBINOM.DIST's cdf is #NUM! at a huge number_s": `#NUM!` from number_s 1E+13 at 28 failures. Probed in
    probes-numeric/t3553-negbinom-huge-s (7 cells in eval/baseline), owned by the new T-3576.
  - "POISSON at x 0 with a negative mean" gains the oracle limit: `POISSON.DIST(2,-2.09435621420062E+249,TRUE)` keeps
    excel-graph from opening its workbook (bisected), so the table caps the mean at 1E+12.
- **Carried fix:** `known-failures.tsv`'s four T.DIST density lines now name T-3554, following T-3358's renumber; that
  fix missed #2648's merge.
