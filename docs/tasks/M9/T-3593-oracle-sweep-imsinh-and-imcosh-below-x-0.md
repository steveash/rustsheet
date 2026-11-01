---
id: T-3593
title: "Oracle sweep: IMSINH and IMCOSH below |x| 0.0105 and IMSIN and IMCOS at |y| 20 to 709, outside T-2778's band"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
T-3594 has dense excel-graph evidence for the two regions where Excel's complex sinh and cosh are outside T-2778's
reciprocal-form band and no model is pinned yet: IMSINH and IMCOSH with |x| below 0.0105, and IMSIN and IMCOS with
|y| from 20 to 709. Next to each IM cell is Excel's worksheet SINH, COSH and EXP at the same argument. Those give the
full double that the 15-digit IM text hides.

## Context
- Gap triage 2026-11-01 (`make eval` on main 408444f20, excel-graph). probes-numeric/t3525-im-hyperbolic-fresh-sweep
  has 5 text misses that no open task names (995 of 1,000 pass since T-3545). T-3594 owns them. That sweep has only
  106 IMSINH/IMCOSH cells with |x| < 0.01 (3 miss) and 94 IMSIN/IMCOS cells with |y| > 20 (2 miss). The IM functions
  return text with 15 significant digits, so a cell only tells kernels apart near a 15-digit rounding tie. A fit needs
  more points than that, plus a hold-out (decision 0188 R4.1), so the cached workbooks are not enough.
- Rule and current model: docs/excel-semantics/engineering.md, the IMSINH/IMCOSH kernel entry (T-2778, T-2629, T-3525:
  "Below the band, IMSINH and IMCOSH miss 3 libm cells ... neither cut is pinned by any cell yet") and "IMSIN and IMCOS
  of a small imaginary part take the reciprocal form" (T-3545: "Still open:
  `IMSIN("0.981793650815856-183.167307387565i")`").
- Engine (for the hypotheses only, no change here): functions/engineering/complex/trig.rs `inverse_exp_sinh_cosh`
  uses libm `sinh`/`cosh` below `INVERSE_EXP_LOW` (0.01). `trig_sinh_cosh` takes `exp_sinh_cosh` (the EXP pair with
  Excel's EXP) above `INVERSE_EXP_HIGH` (20). The worksheet SINH and COSH use `math_and_trigonometry::hyperbolic`.
- Generator precedent: docs/research/t3525/gen_sweep.py (seeded, 15-significant-digit literals). Extend it with a new
  flag and seed. Do not copy it (directive 3).

## Acceptance criteria
- [ ] A new probes-numeric case `eval/suites/probes-numeric/cases/t3593-im-hyperbolic-outside-band.case.json`
      (`rustsheet-case/v1`, one sheet, one formula per cell, tags `M9`, `engineering`, `probe`, `T-3593`). It is
      written by `docs/research/t3525/gen_sweep.py` under a new flag and seed, so it regenerates byte for byte. Grid:
      - Rows 1-300, low band: |x| log-uniform in [1e-6, 0.0105] with a random sign, 15 significant digits. Even rows
        have y = 0. Odd rows have |y| uniform in [0.1, 1.4] with a random sign. A `IMSINH(x+yi)`, B `IMCOSH(x+yi)`,
        C `SINH(x)`, D `COSH(x)`, E `EXP(x)`, F `EXP(-x)`.
      - Rows 301-500, high band: |y| log-uniform in [20, 709] with a random sign. x is 0 on even rows and uniform in
        (-1.5, 1.5) on odd rows. A `IMSIN(x+yi)`, B `IMCOS(x+yi)`, C `SINH(y)`, D `COSH(y)`, E `EXP(y)`,
        F `EXP(-y)`.
      - Also the 5 t3525 arguments verbatim (S!A7, A12, B16, C117 and D117 of the fresh sweep), each with its
        SINH/COSH/EXP companions in the same columns.
- [ ] It is filled with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric` (excel-probe
      skill), and the `.oracle.json` sidecar is committed with no pending cells.
- [ ] docs/excel-semantics/engineering.md, the IMSINH/IMCOSH kernel entry, cites the case. For each band it gives the
      count of IM cells whose parts equal libm's, the EXP pair's and the reciprocal form's digits, and whether every
      IMSINH/IMCOSH real part equals SINH's or COSH's double rounded to 15 digits (and likewise IMSIN/IMCOS
      against SINH/COSH of y).
- [ ] T-3594 links the case.
- [ ] Fixtures hold only formulas and Excel values. No RUSTSHEET_ORACLE_* values in logs or commits.
- [ ] If the oracle is unavailable, the task is `status: blocked` with the reason.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Choosing or implementing a kernel (T-3594). The band's interior 0.0105 to 20 (T-2778, T-3545). |x| above 708
  (T-3546).
