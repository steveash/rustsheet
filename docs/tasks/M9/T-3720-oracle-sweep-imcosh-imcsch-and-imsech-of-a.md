---
id: T-3720
title: "Oracle sweep: IMCOSH, IMCSCH and IMSECH of a complex z with 0 < |x| <= 0.01, with SIN, COS, SINH and COSH companions"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
T-3721 has dense excel-graph evidence for IMCOSH, IMCSCH and IMSECH of a complex z = x+yi with 0 < |x| <= 0.01, the
region where three t3589 texts are still off and no candidate kernel fits them. Each IM cell sits next to Excel's
worksheet SIN(y), COS(y), SINH(x), COSH(x) and EXP(-x) at the same argument. The IM text shows only 15 digits; those
companions give the full double of every factor.

## Context
- Gap triage 2026-11-01h (`make eval` on main b3389661, excel-graph). T-3589's small-x fresh sweep (probes-numeric,
  owned cells listed in T-3721) has 14 failing texts. 11 of them are notation only: a 21-character part below 1 written in E notation. T-3560
  (PR #2735, `MAX_COMPLEX_FRACTION_PLAIN_LEN`) fixes that. The other 3 are 15th-digit misses at a complex x, and
  T-3721 owns them:
  - S!B15 `IMCOSH("-0.00908725305283191-0.897991007302082i")`: Excel 0.623208141637039+0.00710702505540606i, ours
    ...+0.00710702505540607i (the imaginary part, sinh x · sin y).
  - S!D15 `IMCSCH("-0.00908725305283191-0.897991007302082i")`: Excel real part -0.00925756264826299, ours
    -0.009257562648263.
  - S!D29 `IMCSCH("-0.00994266897763484-0.217145289213716i")`: Excel real part -0.208738033086954, ours ...955.
- docs/excel-semantics/engineering.md, "IMSECH and IMCSCH of a small real number" (T-3609), says: every pairing of the
  series, libm and form sinh with the form, libm and series cosh scores the same 105/111 complex IMCOSH and 71/75
  complex IMCSCH texts, and none fits these three. Status UNVERIFIED for B15, D15 and D29. So the x factor alone does
  not explain them. The open candidates are in the y factor (Excel's SIN/COS against libm's), the order of the
  complex product, and IMCSCH's complex reciprocal. The current sweep has only 75 complex rows, too few to rank them
  with a hold-out (decision 0188 R4.1), and it has no SIN/COS companions.
- Split with other tasks:
  - T-3593 (on unmerged branch agent/functions/tasks-gap-triage-opus-2026-11-01, PR #2676) samples IMSINH and IMCOSH
    with SINH/COSH/EXP companions of x. It has no IMCSCH or IMSECH, and no SIN(y)/COS(y).
  - T-2797 owns the real-x cut neighbourhood (0.0095 to 0.011).
  This sweep owns the complex reciprocal functions and the y-side companions. Its rows near the cut are extra evidence
  for T-2797, not a replacement for it.
- Generator precedent: docs/research/t3589/gen_sweep.py (`small_x`, seeded, 15-significant-digit literals). Extend it
  with a new function and seed, or import its helpers. Do not copy it (directive 3).
- Engine, for the hypotheses only (no change here): `inverse_exp_sinh_cosh`, `sinh_series`, `imsech`, `imcsch` in
  crates/rustsheet-engine/src/functions/engineering/complex/trig.rs.

## Acceptance criteria
- [ ] New case `eval/suites/probes-numeric/cases/t3720-im-reciprocal-small-x-complex.case.json` (`rustsheet-case/v1`,
      one sheet `S`, tags `M9`, `engineering`, `probe`, `T-3720`). It is written by docs/research/t3589/gen_sweep.py
      under a new function and seed, so it regenerates byte for byte. Grid, 400 rows:
      - |x| log-uniform in [1E-6, 0.01] with a random sign. Half the rows (200) have |x| in [0.003, 0.01], where all
        three misses lie. |y| uniform in [0.05, 1.5] with a random sign. Every value has 15 significant digits.
      - Columns: A `IMCOSH(z)`, B `IMCSCH(z)`, C `IMSECH(z)`, D `IMSINH(z)`, E `SIN(y)`, F `COS(y)`, G `SINH(x)`,
        H `COSH(x)`, I `EXP(-x)`.
      - Rows 401-404 hold the two t3589 arguments verbatim (the one of S!B15 and S!D15, and the one of S!D29) and
        their complex conjugates, in the same columns.
- [ ] Filled with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric` (excel-probe skill).
      The `.oracle.json` sidecar is committed with no pending cells.
- [ ] docs/excel-semantics/engineering.md, the "IMSECH and IMCSCH of a small real number" entry, cites the case and
      gives two counts: the complex IMCOSH and IMCSCH texts in it whose parts equal the current engine's, and those
      whose parts equal the product computed from the E:I companions (Excel's own factors) in plain doubles.
- [ ] T-3721 links the case.
- [ ] Fixtures hold only formulas and Excel values. No RUSTSHEET_ORACLE_* values in logs or commits.
- [ ] If the oracle is unavailable, the task is `status: blocked` with the reason.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Choosing or implementing a kernel (T-3721). Real x (T-3609, done). Where the 0.01 cut lies (T-2797). Notation of
  parts below 1 (T-3560).
