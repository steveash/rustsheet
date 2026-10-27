---
id: T-2750
title: "IMSECH of 1.34384-77.6742i prints Excel's last digit (im-last-digit-sweep J34, L34)"
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
`IMSECH("1.34384-77.6742i")` prints Excel's `-0.367433538630135+0.376675382306336i`, and IMAGINARY of it
follows, with no other im-last-digit-sweep IMSECH cell lost.

## Context
Follow-up of T-2743, which fitted IMSEC (all 70 sweep cells) and time-boxed IMSECH. Cells
(probes-numeric/im-last-digit-sweep, sheet Calc): J34 `IMSECH("1.34384-77.6742i")`, Excel
`...+0.376675382306336i`, ours `...+0.376675382306335i` (exact imaginary part 0.37667538230633553, so ours is
the one off); L34 `IMAGINARY(J34)` follows.

Candidates T-2743 scored on the 49 IMSECH sweep rows with a nonzero imaginary part (y the trigonometric
argument, SIN/COS by `trig_sin_cos`, libm cosh/sinh; docs/excel-semantics/engineering.md "IM* reciprocal
kernels"):
- current `t = cos²y + sinh²x`: misses only J34 (and row 42, which the overflow path answers).
- `t = cos²y·((cosh 2x + 1)/2 − sinh²x) + sinh²x`: gets J34, misses row 79 (`1+0.5i`).
- `(cosh x·cos y)² + (sinh x·sin y)²` with the result as `(a/t)·b`: gets J34, misses rows 49, 54.
- IMSEC's winning `cos²y + ((cosh 2x + 1)/2 − 1)`: 41 of 49. Smith's division of cosh z: 44 of 49.
  `cos²y·cosh²x + sin²y·(cosh²x − 1)`: 37 of 49.
Next ideas: another cosh/sinh pair (`(e^x ± e^-x)/2` scored 38 of 49 with the current form and 36 with
Smith's), or a reciprocal `1/IMCOSH(z)` through IMDIV's Smith form with IMCOSH's own parts.

Code: `imsech` in crates/rustsheet-engine/src/functions/engineering/complex/trig.rs. Re-check the gated IMSECH
cells: ironcalc-calc ENGINEERING/COMPLEXs.xlsx, probes eng-im-overflow-edges, and gen-m9.

## Acceptance criteria
- [x] probes-numeric/im-last-digit-sweep J34 and L34 pass and no IMSECH cell of that sweep that passes on main
      fails (`make eval-show CASE=probes-numeric/im-last-digit-sweep.case.json`); or, at the time-box, the
      candidates tried are recorded in docs/excel-semantics/engineering.md and the cells stay owned.
- [x] No new known-failure lines in any eval/baseline file.
- [x] A unit test pins the input's 15-digit text.
- [x] `make check` green; reviewers approve.

## Out of scope
- IMSECH in the gen-m9 sweep (T-2537), at a real argument (T-2534), at a large imaginary part (T-2629).

## Visible-parity stage
Decision 0083: J34 is a text result that differs from Excel's printed digits, so the task is in the campaign
at stage `visible`. Its target is that J34 prints Excel's text and L34 is within half a unit of the 15th
significant digit of Excel's value. This fits Excel's formulation where ours is the inaccurate one, and the
work is time-boxed (decision 0056).

## Outcome
IMSECH's denominator is now `fma(sinh x, sinh x, cos²y)` (sinh²x fused into the sum): all 62 finite IMSECH cells of
probes-numeric/im-last-digit-sweep print Excel's digits, J34 included (`-0.367433538630135+0.376675382306336i`).
Scored 29 candidate forms on the 61 sweep rows with a nonzero imaginary part; only the fused form fits all of them
(engineering.md "IM* reciprocal kernels" lists the runners-up). Eval delta: no known-failure line changes in any baseline
(J34/L34 were inside the bit-level gate already); J34 now prints Excel's text, which the visible-parity count picks up.
Unit test: `im_reciprocal_kernels_match_the_sweep` pins J34's text.
