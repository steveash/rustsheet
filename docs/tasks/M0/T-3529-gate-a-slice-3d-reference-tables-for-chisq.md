---
id: T-3529
title: "Gate A slice 3d: reference tables for CHISQ.INV, CHISQ.INV.RT, GAMMA.INV, BETA.DIST and BETA.INV"
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
- [x] Definitions for the five functions, unit-tested against published values; tables generated, filled and
      committed (BETA.INV's table split to T-3578, see the Outcome); each failing point named with its owner task in the Outcome.
- [ ] The R3 points are marked and the MATH-OVER-EXCEL entries cite them: split to T-3578 (no s1 point is in an R3 region).
- [x] The probes-numeric math sidecars are regenerated with the new definitions and committed.
- [x] `make check` green; reviewers approve.

## Out of scope
- The fixes for failing points (their owner tasks).

## Outcome
Done (2026-11-01) for four of the five tables; the rest is split to T-3578.
- **Definitions** (`tools/src/rustsheet_tools/accuracy/defs/`):
  - gamma.py: GAMMA.INV, CHISQ.INV and CHISQ.INV.RT. The quantile solves the smaller tail in log scale. The tails are
    the incomplete gamma's power series and continued fraction (A&S 6.5.29, 6.5.31), summed at the working precision,
    because mpmath's `gammainc` gives NoConvergence from a shape of about 1E5.
  - beta.py: BETA.DIST and BETA.INV on [0, 1]. The density is a domain error at an end with a shape ≤ 1 (T-2435). The
    quantile solves the smaller tail in log scale of x or of 1 - x.
  - Tests against Microsoft's published examples are in `tools/tests/test_accuracy_gamma_beta.py`.
- **Tables:** `s1` batches for GAMMA.INV (177 points), CHISQ.INV (112), CHISQ.INV.RT (107) and BETA.DIST (193), 589
  points, filled from excel-graph.
  - The batches stop at a shape of 1E7 (df 2E7), and log-uniform draws beyond it are dropped (the generator's 10 s
    cap). Points mpmath's incomplete beta does not converge on are dropped too.
- **Check:** 0 failing points and 0 table defects. The 4 points outside the envelope are known failures owned by the
  new **T-3577**. All are in accuracy/BETA.DIST/s1:
  - A184, A187 and A193: the cdf at tiny shapes, where the engine returns 0 and Excel matches the math.
  - A186: the cdf at a huge beta (1.8E+214), where Excel gives `#NUM!` and the engine and the math give 0.
- **Split to T-3578:**
  - BETA.INV's table. Its 194 points generate, but excel-graph fails at createSession on that workbook in three
    runs, while the other four tables filled in the same runs.
  - The huge-shape batch with the R3 points of "GAMMA.INV at a huge shape, closed-form branch". It needs a
    uniform-asymptotic incomplete gamma in the generator.
  - The tiny-p R3 points of BETA.INV and CHISQ.INV.RT. No point is marked here, so no decision record is needed;
    T-3578 fixes their regions.
- **probes-numeric math sidecars:** regenerated with the new definitions.
- **Eval delta:** none. Gate A gains 4 tables (1391 points in 10 tables), and the eval suites, baselines and gates
  are unchanged.

