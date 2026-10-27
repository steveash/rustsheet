---
id: T-2841
title: "Fit the IMSEC and IMCSCH part kernels on the im-sec-csch-fused misses"
milestone: M9
lane: functions-engineering
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
IMSEC and IMCSCH print Excel's 15 digits on every cell of probes-numeric `im-sec-csch-fused`, with no old cell lost.

## Context
T-2817 probed 34 IMSEC and 44 IMCSCH inputs where denominator forms disagree (excel-graph, 2026-10-27). No
grouping or fusing of the denominator fits all of them (engineering.md "IM* reciprocal kernels" has the scores):
IMSEC's kernel fits 23 of 34 texts, IMCSCH's new `sinh²x + sin²y` 33 of 44. The misses are likely in the parts:
libm's `cosh`/`sinh` (try the x87 `excel_exp`-based forms such as `inverse_exp_sinh_cosh` or `exp_sinh_cosh` in
complex/trig.rs), IMCSCH's plain `y.sin()`/`y.cos()` (try `trig_sin_cos`, Excel's reduced SIN and COS, as IMSEC and
IMSECH use), or the numerators' grouping (`x_sinh * y_cos / t` vs `x_sinh * (y_cos / t)`). Score candidates on every
IMSEC and IMCSCH oracle cell (gen-m9, ironcalc-calc, probes, probes-numeric `im-*`), as T-2757 and T-2817 did, with a
scratch test outside the commit. Misses to start from: `IMSEC("-1.44771+0.0419247i")` and
`IMCSCH("0.0853196+0.00680423i")`, which no T-2817 form fits.
Also probe a large real part with y = 0, such as `IMCSCH("400")` and `IMCSCH("710")`. With T-2817's `sinh²x + sin²y`, an
overflowed `t` is `inf` (it was NaN), and only `IMCSCH("710.4+0i")` (probes/eng-im-overflow-edges A64) covers that path.

## Acceptance criteria
- [x] Each kernel fits more `im-sec-csch-fused` cells than today and loses no older oracle cell (or the task records
      that no candidate does, with the scores).
- [x] engineering.md "IM* reciprocal kernels" records the scores.
- [x] `make check` green; reviewers approve.

## Out of scope
- IMCSC, IMCOT, IMSECH.

## Outcome
- IMCSCH takes IMSINH's sinh x and cosh x (`inverse_exp_sinh_cosh`): all 44 `im-sec-csch-fused` texts fit (33 before),
  `IMCSCH("0.0853196+0.00680423i")` included. IMSEC divides each hyperbolic part by t before multiplying,
  `cos x·(cosh y/t)`: 27 of 34 (23 before). No older IMSEC or IMCSCH oracle cell is lost (gen-m9, ironcalc-calc,
  probes, probes-numeric `im-*`, scored with a scratch knob build outside the commit). engineering.md "IM* reciprocal
  kernels" has every candidate's score; two unit cases in `test_complex.rs`.
- Eval delta: probes-numeric `im-sec-csch-fused` 190 -> 220 of 234 cells; every other case unchanged; gated suites 0
  fixed, 0 regressions (no baseline change).
- The `IMCSCH("400")` / `IMCSCH("710")` probe was not run; that path is unchanged (`inverse_exp_sinh_cosh` is libm's
  outside its band, as before).
- Follow-up: T-2855 (IMSEC's last seven fused texts).
