---
id: T-3358
title: "Gate A slice 3: first reference tables for the numeric functions in scope, oracle-filled, with the MATH-OVER-EXCEL points marked"
milestone: M0
lane: harness
status: todo
priority: P0
size: M
tier: opus
depends_on: [T-3356, T-3357, T-3379]
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
Committed, oracle-filled Gate A tables for the first batch of numeric functions the Gate A spec names (T-3355), with
the decision 0188 R3 points marked, and `make check` running them.

## Context
- Decision 0188 R1(a), R2, R3. The first batch must include the functions of the owner's 2026-10-31 assessment: IRR,
  RATE, XIRR, NPER, T.INV, T.INV.2T, TINV, F.INV, CHISQ.INV, CHISQ.INV.RT, BETA.INV, GAMMA.INV, BINOM.DIST,
  NEGBINOM.DIST, POISSON.DIST, F.DIST, T.DIST, BETA.DIST. The spec splits the rest into further slices.
- R3's initial `MATH-OVER-EXCEL` cases (statistics.md and financial.md) need committed evidence points: the deep-tail
  quantiles (for example `T.INV(1.87E-297,3937)`, Excel -156.96, math -40.26), NPER at rates below 1e-14, GAMMA.INV at
  a huge shape. Add them as R3-marked points, filled, and cite them from the semantics entries.
- Points that fail on landing are known bugs: the P0 tasks T-3347 to T-3354 and T-3367 own them. The spec says how a
  failing point is carried until its task lands (never by weakening ε or the 15-digit rule).
- SPEC-0029: a failing point is carried by a `known-failures.tsv` line naming its owner task, allowed only for points
  added in the same diff (T-3379); R3 points are lines of `math-over-excel.tsv` (protected: a decision record citing
  0188 R3's initial cases). Functions with no table yet are listed by the check's missing-table count; the follow-up
  slices by family are in SPEC-0029's rollout step 4.

## Acceptance criteria
- [ ] Tables for the batch are committed and filled; each failing point is named with its owner task in the Outcome.
- [ ] The `MATH-OVER-EXCEL` entries cite their committed evidence points.
- [ ] Further slices for the remaining numeric functions are filed.
- [ ] `rustsheet-accuracy math --suite probes-numeric` (T-3380) is rerun after the new definitions land, and its
      math sidecars are committed, so R1(c) sees the math value of every probes-numeric cell of those functions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing the failing points (their own tasks).
