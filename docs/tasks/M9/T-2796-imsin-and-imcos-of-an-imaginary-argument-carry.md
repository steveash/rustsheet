---
id: T-2796
title: "IMSIN and IMCOS of an imaginary argument carry IMSINH's digits (IMSIN('-4i') -27.2899171971278i)"
milestone: M9
lane: functions-engineering
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
IMSIN and IMCOS of an imaginary or near-imaginary argument print Excel's digits, starting with probes/t2778-imsinh-sign-sweep C2.

## Context
- Left by T-2629. `IMSIN("-4i")` is "-27.2899171971278i" in Excel (probes/t2778-imsinh-sign-sweep C2, excel-graph),
  the digits of `IMSINH("4")` (27.289917197127753, the `1/EXP(-x)` kernel, engineering.md "IMSINH, IMCOSH and IMSECH take
  sinh and cosh as (1/u ∓ u)/2"). Ours: "-27.2899171971277i", from `exp_sinh_cosh(-4)` = `(EXP(-4) - EXP(4))/2`.
- IMSIN's own sinh is the EXP pair elsewhere: `IMSIN("1E-200+1E-200i")` is "1E-200" and `IMCOS("1.5E+2-2E-3i")` needs
  the EXP form (engineering.md, T-2535), so a plain switch to `inverse_exp_sinh_cosh` may regress them. Hypothesis to
  test first: IMSIN(z) = -i·IMSINH(i·z) for a zero real part only, or the `1/EXP(-y)` kernel over |y| in its band.
- The other C/D cells of that probe (`IMSIN("4i")`, `"±12i"`, `"±0.01i"`, IMCOS) pass; probe more imaginary-axis and
  mixed arguments before refitting.
- Code: `imsin`, `imcos`, `exp_sinh_cosh` in crates/rustsheet-engine/src/functions/engineering/complex/trig.rs.

## Acceptance criteria
- [x] probes/t2778-imsinh-sign-sweep C2 leaves eval/baseline/probes.txt with no new known-failure lines in any
      baseline, or the candidates tried are recorded in engineering.md (time-box, decision 0056).
- [x] `make check` green; reviewers approve.

## Out of scope
- IMSINH/IMCOSH (T-2629, done).

## Outcome
IMSIN and IMCOS take `(sinh y, cosh y)` from IMSINH's kernel at `-y` (`trig_sinh_cosh`, complex/trig.rs; sin z =
-i·sinh(iz)) inside its band (0.01, 20], and the EXP pair outside, so `IMSIN("-4i")` is -27.2899171971278i. The first
hypothesis tried fitted: all 1,023 IMSIN/IMCOS cells of the suites still pass. engineering.md updated.
Eval: probes known failures 2585 -> 2584 (C2), no regressions.
