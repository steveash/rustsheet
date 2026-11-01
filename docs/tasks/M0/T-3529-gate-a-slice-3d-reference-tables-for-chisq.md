---
id: T-3529
title: "Gate A slice 3d: reference tables for CHISQ.INV, CHISQ.INV.RT, GAMMA.INV, BETA.DIST and BETA.INV"
milestone: M0
lane: harness
status: todo
priority: P0
size: M
tier: opus
depends_on: [T-3357, T-3379]
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
Committed, oracle-filled Gate A tables for CHISQ.INV, CHISQ.INV.RT, GAMMA.INV, BETA.DIST and BETA.INV, each with a
cited mpmath definition, the decision 0188 R3 points marked, and `make check` running them.

## Context
- Split out of T-3358 (2026-10-31), which keeps the Student-t and F families (T.DIST, T.INV, T.INV.2T, TINV, F.DIST,
  F.INV); T-3553 has the discrete distributions and T-3535 the financial solvers.
- Domains and edges to cite: statistics.md "Chi-squared quantile at df 2 and gamma quantile at alpha 1 are closed
  forms (T-2860)", "BETA.INV solves Excel's own CDF to the crossing (T-1745)", "BETA.DIST's density is #NUM! at an x
  of 0 or 1 (T-2435)", "BETAINV is #NUM! where its quantile underflows ... (T-2435)", the inverse-CDF p range note
  ("NORM.INV, ... CHISQ.INV, CHISQ.INV.RT, GAMMA.INV and BETA.INV return a value (or #NUM!) for p from 1E-300 to
  1-1E-16"), and the A/B bounds of BETA.DIST/BETA.INV.
- R3 points: GAMMA.INV at a huge shape on the Wilson-Hilferty branch (statistics.md "GAMMA.INV at a huge shape,
  closed-form branch: Excel is about 2.7e-4 off"), and the deep-tail CHISQ.INV.RT and BETA.INV quantiles of the
  "Tiny-p quantiles" entry. `math-over-excel.tsv` is protected: a decision record citing 0188 R3's initial cases.
- Known failures on landing: CHISQ.INV and CHISQ.INV.RT `#NUM!` at df ≥ 2e7 (T-3350), BETA.INV `#NUM!` at p near
  1 - 1e-8 (T-3351); carried by `known-failures.tsv` lines naming them (T-3379's rules).
- Tools: as T-3358 (`rustsheet-accuracy generate`, `rustsheet-oracle fill --suite accuracy`, `rustsheet-eval
  accuracy`), then rerun `rustsheet-accuracy math --suite probes-numeric` (T-3380) and commit the sidecars.

## Acceptance criteria
- [ ] Definitions for the five functions, unit-tested against published values; tables generated, filled and
      committed; each failing point named with its owner task in the Outcome.
- [ ] The R3 points are marked and the MATH-OVER-EXCEL entries cite them.
- [ ] The probes-numeric math sidecars are regenerated with the new definitions and committed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The fixes for failing points (their owner tasks).
