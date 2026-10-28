---
id: T-2743
title: "IMSEC near a pole and IMSECH of 1.34384-77.6742i print Excel's text (im-last-digit-sweep G29, G33, J34)"
milestone: M9
lane: functions-engineering
status: done
priority: P3
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
IMSEC of `23.7944+0.00977722i` and `956.615+0.00387493i` and IMSECH of `1.34384-77.6742i` print Excel's
15-digit text, and the IMREAL/IMAGINARY cells that read them back follow, with no other
im-last-digit-sweep cell lost.

## Context
Gap triage 2026-10-25. T-2044 fitted the IM* reciprocal kernels on probes-numeric
`im-last-digit-sweep` (excel-graph, T-2043) and left three rows unowned (its Outcome: "IMSEC/IMSECH below
99.9% of their sweep cells (time-boxed, misses recorded)"; docs/excel-semantics/engineering.md "IM*
reciprocal kernels, IMABS and IMARGUMENT (T-2044)"). No open task owns them: T-2537 is the gen-m9 IMSECH
cells, T-2534 real arguments, T-2629 IMCOSH/IMSINH/IMSECH at a large imaginary part.

Cells (probes-numeric/im-last-digit-sweep, sheet Calc; Excel, ours, exact from mpmath at 40 digits):
- G29 `IMSEC("23.7944+0.00977722i")`: Excel `4.33329584706818-0.178960935839407i`, ours
  `4.33329584706819-0.178960935839407i` (exact 4.333295847068187427-0.178960935839406811i, so Excel's real
  part is the less accurate one). H29 `IMREAL(G29)` follows.
- G33 `IMSEC("956.615+0.00387493i")`: Excel `-2.46276584105671+258.045027895372i`, ours
  `-2.4627658410385+258.045027893464i` (exact -2.462765841048564+258.045027894412i). Off in the 10th to 12th
  digit on both sides: cos x is about -3.7E-5 here, so the denominator t is a small difference.
  H33 `IMREAL(G33)` and I33 `IMAGINARY(G33)` follow.
- J34 `IMSECH("1.34384-77.6742i")`: Excel `-0.367433538630135+0.376675382306336i`, ours
  `...+0.376675382306335i` (exact imaginary part 0.37667538230633553, so here ours is the one off).
  L34 `IMAGINARY(J34)` follows. T-2044 notes the old `cosh²x cos²y + sinh²x sin²y` kernel got J34 right but
  missed rows 28, 49 and 79.

Lead (triage scratch check, Python + mpmath; not yet scored by the engine): keeping the x87 64-bit π
reduction of `trig_sin_cos` and libm `cosh`/`sinh`, the IMSEC denominator
`t = cos²x·cosh²y + sin²x·(cosh²y − 1)` reproduces both G29 and G33 to all 15 printed digits, where the
current `cos²x + (cosh 2y − 1)/2`, `cos²x + sinh²y`, `cos²x + (cosh 2y − 1)/2` with exp-based cosh, and the
double-angle `(cosh 2y + cos 2x)/2` do not. A rough re-implementation of the sweep scoring suggested it may
cost row 79 (`1+0.5i`); score it on every sweep row in the engine before taking it. Try the same shape for
IMSECH (`cos²y·cosh²x + sin²y·(cosh²x − 1)`) against J34 and rows 28, 49, 79.
This is fitting Excel's formulation where Excel is the inaccurate one (G29, G33), so the stage is
`visible` and the work is time-boxed (decision 0056).

Code: `imsec`, `imsech` in crates/rustsheet-engine/src/functions/engineering/complex/trig.rs. Re-check the
gated IM* cells that use these kernels: ironcalc-calc ENGINEERING/COMPLEXs.xlsx, probes eng-im-overflow-edges,
eng-argument-modes, and the gen-m9 IMSEC/IMSECH cells (`make eval` "Functions with failing cells").

## Acceptance criteria
- [x] probes-numeric/im-last-digit-sweep G29, H29, G33, H33, I33, J34 and L34 pass (`make eval-show
      CASE=probes-numeric/im-last-digit-sweep.case.json`), and no IMSEC or IMSECH cell of that sweep that passes
      on main fails; or, at the time-box, the candidates tried and their sweep scores are recorded in
      docs/excel-semantics/engineering.md "IM* reciprocal kernels" and the cells stay owned by a follow-up.
- [x] No new known-failure lines in any eval/baseline file (`make eval-baseline` refuses regressions); the
      gated IMSEC/IMSECH cells in ironcalc-calc, probes and gen-m9 pass at least as before.
- [x] Unit tests pin the two IMSEC inputs' 15-digit texts (J34 is pinned by T-2750).
- [x] `make check` green; reviewers approve.

## Out of scope
- IMSEC/IMSECH at a real argument (T-2534) and IMSECH in the gen-m9 sweep (T-2537).
- IMCOSH/IMSINH/IMSECH at a large imaginary part (T-2629).
- IMABS B33 of the same input (`956.615+0.00387493i`, 2 ulp; T-2538).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-25): G29, G33 and J34 are text results that differ from Excel's printed
digits (G33 in the 10th digit), so the cells are visibly off and the task is in the campaign at stage
`visible`. Its target is that every cell it owns prints Excel's text and the IMREAL/IMAGINARY readers are
within half a unit of the 15th significant digit of Excel's value; the first acceptance criterion states it.
Bit-level criteria still unmet then move to a follow-up with `defer: phase-1b`.

## Outcome
First criterion met through its time-box clause for J34/L34 (owned by T-2750); the IMSEC cells pass.
IMSEC's denominator is now `cos²x + ((cosh 2y + 1)/2 − 1)` (sinh²y as cosh²y − 1 by the half-angle identity):
all 70 finite IMSEC cells of im-last-digit-sweep, so G29, H29, G33, H33 and I33 pass (sweep 1399 of 1404
cells) and row 79 is kept. IMSECH hit the time-box: no candidate got J34 without losing another row; the
scores are recorded in docs/excel-semantics/engineering.md and J34/L34 are owned by T-2750. eval/baseline
unchanged (no gated case moved; probes-numeric is not baselined).
