---
id: T-3358
title: "Gate A slice 3: first reference tables for the numeric functions in scope, oracle-filled, with the MATH-OVER-EXCEL points marked"
milestone: M0
lane: harness
status: done
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
- **Split 2026-10-31** (18 functions with 3 definitions is more than an M task): this task keeps the continuous
  Student-t and F families: T.DIST, T.INV, T.INV.2T, TINV, F.DIST and F.INV, with the deep-tail T.INV R3 points.
  T-3529 takes CHISQ.INV, CHISQ.INV.RT, GAMMA.INV, BETA.DIST and BETA.INV (with GAMMA.INV's huge-shape R3 points),
  T-3543 the discrete distributions (BINOM.DIST, NEGBINOM.DIST, POISSON.DIST) and T-3535 the financial solvers (IRR,
  XIRR, NPER, RATE, with NPER's R3 points).
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
- [x] Tables for this task's functions (the split above) are committed and filled; each failing point is named with its owner task in the Outcome.
- [x] The `MATH-OVER-EXCEL` entries cite their committed evidence points.
- [x] Further slices for the remaining numeric functions are filed.
- [x] `rustsheet-accuracy math --suite probes-numeric` (T-3380) is rerun after the new definitions land, and its
      math sidecars are committed, so R1(c) sees the math value of every probes-numeric cell of those functions.
- [x] `make check` green; reviewers approve.

## Out of scope
- Fixing the failing points (their own tasks).

## Outcome
Done (2026-10-31), on T-3379's governance (built on its branch).
- **Split:** this task keeps the Student-t and F families. The rest are filed as T-3543 (BINOM.DIST, NEGBINOM.DIST,
  POISSON.DIST), T-3535 (IRR, XIRR, NPER, RATE, with NPER's R3 points) and T-3529 (CHISQ.INV, CHISQ.INV.RT, GAMMA.INV,
  BETA.DIST, BETA.INV, with GAMMA.INV's R3 points). The missing-table count lists the other 317 numeric functions;
  the family slices of SPEC-0029 rollout step 4 cover them as they are filed.
- **Definitions** (`tools/src/rustsheet_tools/accuracy/defs/`):
  - T.INV.2T and TINV in student_t.py. Both use one definition, with p in (0, 2) as Excel takes it; p in (1, 2) is
    `T.INV(1 - p/2)`.
  - F.DIST and F.INV in the new fisher.py. Both tails come from their own incomplete beta, and the quantile solves
    the smaller tail in log scale. F.INV(0) is 0, as Excel gives it.
  - Tests against Microsoft's published examples are in `tools/tests/test_accuracy_student_f.py`.
- **Tables:** `s1` batches for T.DIST (118 points), T.INV (109), T.INV.2T (104), TINV (102), F.DIST (198) and F.INV
  (171), 802 points in all, filled from excel-graph.
  - The generator dropped 11 to 31 points per inverse where mpmath does not converge at a huge df, the weakness
    T-3425 owns.
- **Check:** 788 points are inside the envelope (11 on the Excel side only, 94 on the math side only), with 0 failing
  points and 0 table defects. The 14 points outside it are known failures with owners:
  - **T-1871**, 2 points: F.INV deep tail, accuracy/F.INV/s1 A154 and A168. The engine is far off where Excel
    matches the math.
  - **T-3363**, 4 points: T.INV and T.INV.2T at df from 7E7 to 1E10 (T.INV/s1 A75, A102; T.INV.2T/s1 A75, A91). The
    engine and Excel are both off the math, differently.
  - **T-3386**, 4 points: T.INV.2T and TINV at p = 2.2E-308, where Excel gives `#NUM!` (T.INV.2T/s1 A68, A78;
    TINV/s1 A68, A78).
  - **T-3533** (new), 4 points: T.DIST's density is `#NUM!` in Excel at a huge x where the engine returns 0
    (T.DIST/s1 A85, A95, A105, A110).
- **R3 marks (decision 0213):** 8 points inside the tiny-p case's approved region (p < 1e-150):
  - T.INV/s1 A89, A93 and A95;
  - F.INV/s1 A112, A126, A146, A160 and A171.

  statistics.md's "Tiny-p quantiles" entry now cites them and states their p and df ranges. The tables have 17
  points with Excel more than G off the math. The other 9 are left for an owner ruling and pass Gate A through its math
  side:
  - T.INV A91, A96 and F.INV A164, A169, at p ≥ 1e-150 (new T-3544);
  - T.INV.2T A98, A99, A103 and TINV A102 (T-3386);
  - F.INV A167, inside the region but with a true quantile (3.0E-377) below the double range, where the engine's 0
    passes only through the 1e-300 floor (T-3544).
- **New quirks** (statistics.md, "T.DIST's density is #NUM! at a huge x, and T.INV.2T and TINV at p = 2.2E-308"):
  - T.DIST's density is `#NUM!` at |x| from 1.0E+236 and 0 up to 9.9E+137.
  - The two-tailed quantiles are `#NUM!` at p = 2.2E-308.
  - F.INV(0) is 0, while the legacy FINV(0) is `#NUM!`.
  - The table also settles the T-2474 entry's open question: the cumulative T.DIST is `#NUM!` at a huge negative x
    too, with the cut in (4.0E+136, 5.4E+186).
- **probes-numeric math sidecars,** regenerated with the new definitions: 109,909 cells in 67 cases, 16 MB, in about
  2 minutes.
  - 32,184 of the 70,286 visibly-off numeric cells now have a math value, up from 588.
  - Of those 32,184, 32,067 have both the engine and Excel within ε of the math: Phase 1b evidence under R1(c)
    (T-3427). Most are F.DIST.
- **Eval delta:** none. Gate A gains tables, and the eval suites, baselines and gates are unchanged.

