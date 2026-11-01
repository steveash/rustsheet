---
id: T-3755
title: "IMCOSH and IMCSCH of a complex z with |x| <= 0.01: the 3 t3589 texts one unit off in the 15th digit (attempt 2 after T-3609)"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
The three complex-argument texts of T-3589's small-x sweep that T-3609 left UNVERIFIED (cells under Context)
show Excel's text, or engineering.md records the new forms tried and their scores. Every IM* text that passes
today keeps passing.

## Visible-parity stage
`visible`: each cell below differs from Excel by one unit in the 15th significant digit, and Excel is the inaccurate
one. The 50-digit values below come from mpmath on the double inputs (gap triage, 2026-11-01). The engine's digits are
the correctly rounded ones, so the fix reproduces Excel's operation order. A more accurate kernel would not help. Excel's
error is about 1e-15 relative, far below decision 0188 R3's 1e-4, so Excel's digits are the target. Time-boxed
(decision 0056). Target: these 3 cells within half a unit of Excel's 15th digit (their text equal to Excel's), with no
IM* text cell newly failing. If no candidate does that, the Outcome lists the candidates and their scores and the cells
stay UNVERIFIED.

## Context
- Gap triage 2026-11-01 (`make eval` on origin/main 26e0bd00b, `scripts/exit_owners.py --all-off`): these 3 cells are
  visibly off and no open task owns them. T-3609 (done) took them on and left them UNVERIFIED in
  docs/excel-semantics/engineering.md ("IMSECH and IMCSCH of a small real number", last sentences). Its scorer found
  that every pairing of the series, libm and `1/u` sinh with the `1/u`, libm and series cosh gives the same
  105/111 complex IMCOSH and 71/75 complex IMCSCH texts. None of those pairings fits these three, so the miss is not
  in sinh x or cosh x. The other misses in those counts are notation (T-3560).
- Cells (`make eval-show CASE=<case>`):
  - probes-numeric/t3589-im-hyperbolic-small-x-sweep.case.json (3): S!B15, D15, D29
  - B15 `=IMCOSH("-0.00908725305283191-0.897991007302082i")`: Excel `0.623208141637039+0.00710702505540606i`, ours
    `...+0.00710702505540607i`. The 50-digit imaginary part is 0.0071070250554060662.
  - D15 `=IMCSCH("-0.00908725305283191-0.897991007302082i")`: Excel `-0.00925756264826299+1.27852743654627i`, ours
    `-0.009257562648263+...`. The 50-digit real part is -0.0092575626482629984.
  - D29 `=IMCSCH("-0.00994266897763484-0.217145289213716i")`: Excel `-0.208738033086954+4.63196641538457i`, ours
    `-0.208738033086955+...`. The 50-digit real part is -0.20873803308695452.
- What differs from T-3609's search: the factors and operations other than sinh x and cosh x. Candidates not yet scored:
  - The engine's worksheet SIN/COS kernels (`excel_sin_cos`, `trig_sin_cos`) for sin y and cos y. Today `imcsch` and
    `imcosh` call libm's `y.sin()` and `y.cos()` (functions/engineering/complex/trig.rs).
  - Other orders for IMCSCH's quotient, which today is `x_sinh·cos y/t` and `-x_cosh·sin y/t` with
    `t = sinh²x + sin²y` (T-2817, T-2841): the textbook `|sinh z|²`, a hypot-based modulus, Smith's division, and
    IMDIV's own kernel applied to `1` and IMSINH's parts.
  - For IMCOSH, a product order, or a fused multiply-add (`mul_add`) in `sinh x·sin y`.
  - A rough off-engine check (Python, libm exp) rules one thing out. The complex-exponential form
    `(e^z + e^-z)/2` happens to give B15's ...606, but it fits only about 6 of the 111 complex IMCOSH texts, because
    the imaginary part cancels. In the same check, the naive, Smith and hypot reciprocals miss different IMCSCH cells.
    So the cached texts already tell these candidates apart. Score them in-crate with the engine's own `excel_exp`
    and text formatting, as T-3609 did, not with Python's libm.
- Evidence: about 186 cached complex texts with 0 < |x| <= 0.01 (111 IMCOSH, 75 IMCSCH, plus IMSINH and IMSECH) across
  the IM* hyperbolic sweeps of T-3589, T-3525 and T-2535 and the IM* last-digit sweep (T-2044). Score on these first. Draw a fresh sweep only after the choice is frozen (R4.2 below).
- Fit-lab fits that must not lose a cell: `im-hyperbolic`, `im-hyperbolic-fresh`, `im-hyperbolic-t3589-fresh`,
  `im-small-imag`, and the 44 IMCSCH texts of T-2817's fused-form case (all passing).
- Code: crates/rustsheet-engine/src/functions/engineering/complex/trig.rs (`imcosh`, `imcsch`,
  `inverse_exp_sinh_cosh`, `reciprocal_result`). Semantics: docs/excel-semantics/engineering.md, entries "IMSECH and
  IMCSCH of a small real number" and "IM* reciprocal kernels".

## Acceptance criteria
- [ ] probes-numeric/t3589-im-hyperbolic-small-x-sweep S!B15, D15 and D29 pass (`make eval-show`), no other
      probes-numeric IM* cell starts failing, and no baseline under eval/baseline/ gains a line. probes-numeric is
      ungated, so it has no baseline file; the before/after counts come from the report. Otherwise engineering.md lists
      every candidate scored, with its counts over the cached complex texts, and the cells stay UNVERIFIED.
- [ ] Before/after counts in the Outcome for each fit-lab fit named in Context and for T-2817's 44 IMCSCH texts.
      No count drops.
- [ ] Decision 0188 R4 guards (reviewer-excel-fidelity check 7), as the change's kind requires: (a) a fit, or (c) a
      structural model chosen from ranked candidates:
  - [ ] Hold-out (R4.1): train and hold-out counts side by side (bit-exact, within 4 ulp, within 15 digits) for the
        chosen form and the correctly rounded one, on a split and margin fixed before any scoring.
  - [ ] Fresh sweep (R4.2): after the freeze, a new probes-numeric case with complex z, |x| log-uniform in
        [1e-6, 0.01] and |y| in [0.05, 1.5], oracle-filled on excel-graph, with its result in the Outcome.
  - [ ] Gate A envelope (R2): the fresh sweep's count inside the envelope, against attached mpmath values, with every
        point outside it listed.
  - [ ] `FITTED(T-3755, n_cells, range)` tag and range on any fitted constant or cut. The code takes accurate math
        outside that range. A kind (c) structural model needs no tag.
  - [ ] Reproducibility floor (R4.4): no step targets differences below the oracle's floor (web against desktop up to
        4 ulp; bits that vary between fills).
  - [ ] No keep-matched branch: no band or special case whose domain is the set of previously matching cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Notation of small parts (`0.0000060902323796872` against `6.0902323796872E-06`): T-3560.
- The real-argument IMSECH and IMCSCH kernels (T-3609, verified), the 0.01 band edge (T-2797), and extreme magnitudes
  (T-3642).
