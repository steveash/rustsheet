---
id: T-3357
title: "Gate A slice 2: the mpmath reference generator, the stratified sampler and the oracle fill in tools/"
milestone: M0
lane: harness
status: done
priority: P0
size: M
tier: opus
depends_on: [T-3355]
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
A tool in tools/ writes Gate A reference tables (T-3355's format): it draws stratified points per function (about 40%
typical, 30% tails and domain edges, 20% extreme-but-valid, 10% log-uniform; decision 0188 R2), computes each point's
50-digit mpmath value, and has the oracle fill Excel's value.

## Context
- Decision 0188 R2; design in the Gate A spec (T-3355).
- Reuse the oracle client (`rustsheet-oracle`), the spec→xlsx builder and `rustsheet-eval snapshot`; Python is ruff
  and mypy strict; never re-implement xlsx value parsing (CLAUDE.md conventions).
- SPEC-0029 decides the design: `tools/src/rustsheet_tools/accuracy/`, CLI `rustsheet-accuracy` (`generate`,
  `verify`; `math` is T-3380), `Definition`s per family with cited valid and extreme ranges, dps 60 and 80 with a
  1e-45 convergence check, seeds from `sha256(FUNCTION/batch)`, the five strata and their shares, 120/200/300 points by
  arity, literals of at most 15 significant digits below 1E+308, and the fit lab's refusal of `accuracy/*` sweeps.
- The oracle fill needs nothing new once T-3356's `Suite::load_named` resolves `--suite accuracy`; until then, test the
  fill on a temporary suite.

## Acceptance criteria
- [x] The generator is deterministic (same seed, same points), and its mpmath definitions for at least three functions
      (one distribution, one inverse, one financial solver) are unit-tested against published values.
- [x] The fill writes Excel's values into a table and leaves unfilled points pending, as the spec says.
- [x] `make check` green (ruff, mypy, tests); reviewers approve.

## Out of scope
- The make check gate (T-3356); generating every function's table (T-3358).

## Outcome
- **What changed.** `tools/src/rustsheet_tools/accuracy/` and the CLI `rustsheet-accuracy list|generate|verify`:
  - `literal`: 15-digit literals, half-even, below 1E+308, never subnormal, and the 15-digit neighbours of a bound.
  - `model`: `Param`, `Definition`, `DomainError`, `NoValueError`, the registry, and `solve_bracket` (Illinois, keeps its bracket).
  - `sampler`: seed `sha256(FUNCTION/batch) % 2**63`; strata 40/15/15/20/10 by largest remainder; 120/200/300 points; tails round-robin
    over the tail map and the bounded reals; edges are each bound, its inside literal, the invalid literal outside, and a parameter's extra edges.
  - `tables`: dps 60 and 80 must agree to 1e-45, 10 s per point (SIGALRM). Non-converged, too slow and mpmath `NoConvergence` points
    are dropped and logged; overflow and several roots are redrawn up to 20 times. It writes rustsheet-case/v1 (hypotheses
    `gate-a/<stratum>/<args>`) and rustsheet-math/v1, refuses to overwrite a batch or take a path as a batch name, and `verify` recomputes.
  - `defs/student_t.py` defines T.DIST and T.INV from the incomplete beta. The quantile is solved in ln/ln space; domains are cited
    from statistics.md (T-1043, T-1531): df ≥ 1, cumulative and T.INV cut at 1E10, the density finite above it.
  - `defs/annuity.py` defines RATE as the unique root when the flows change sign exactly once: no change is a domain error, several
    are never drawn. The annuity is drawn as a whole and pmt is set from a drawn rate.
  - mpmath 1.3.0 is pinned in tools/ only. The fit lab refuses `accuracy/*` sweeps and `calls` cases unless `--holdout-only` is given
    (docs/architecture/fit-lab.md).
- **Tests** (`tools/tests/test_accuracy.py`):
  - Determinism (identical bytes). Strata counts. Literals and domain bounds of drawn points; tail-map points are in the tail.
  - T.DIST against the df 1 and df 2 closed forms at 1e-45, and Excel's T.DIST(2,1,TRUE).
  - T.INV against the df 2 closed form, t(0.975,10), -cot(πp) at df 1, and SPEC-0029's `T.INV(1.87E-297,3937)` ≈ -40.26.
  - RATE against Microsoft's 0.77% example and decision 0188's 0.0431054691928255, with the root's residual below 1e-40.
  - Dropped and logged points (unconverged, NoConvergence, overflow). `verify` catching a changed value.
  - Parameter order matching the catalog. The fit-lab refusal.
  - The fill end to end: a fake Excel backend fills 2 of 4 points through `rustsheet-oracle fill --suite accuracy` (the real
    `rustsheet-eval cases accuracy` listing); the sidecar holds Excel's values and `rustsheet-eval accuracy` reports `pending 2`.
- **Trial batches** (not committed; T-3358 commits real tables):
  - T.INV s1: 109 of 120 kept in 21 s. The 11 drops are all df ≥ 5.4E6, mpmath hypergeometric non-convergence: T-3425.
  - T.DIST s1: 118 kept. RATE s1: 300 kept.
  - Run through `rustsheet-eval accuracy` while pending, T.DIST shows 4 engine `#NUM!` at |x| ≥ 1E186 where the math is 0 or 1.
    That is for T-3358 to confirm against Excel once the batch is filled.
- **Choices where the spec is silent.** `source.dps` records 80, the precision of the written value; dps 60 is the convergence pass.
  The tail probability is log-uniform in [1E-300, 0.1], the tail distance relative to max(|bound|, 1). The batch case carries
  tags `accuracy`, `gate-a`, FUNCTION. RATE writes five arguments (no guess).
- **Eval delta:** none (tools only).
- **Follow-ups:** T-3425.
