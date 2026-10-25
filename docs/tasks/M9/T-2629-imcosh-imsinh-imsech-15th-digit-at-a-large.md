---
id: T-2629
title: "IMCOSH/IMSINH/IMSECH 15th digit at a large imaginary part (eng-im-overflow-edges A9, A17, A58)"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
probes/eng-im-overflow-edges A9, A17 and A58 print Excel's 15th digit.

## Context
Left by T-2231. Each differs in the last printed digit of the imaginary part, and each exact value sits within
about 5E-19 of the 15-digit rounding midpoint, so the cells only tell kernels apart through their last ulp:
- A9 `IMCOSH("1.5+710.4i")`: Excel 0.82929871219349, ours 0.829298712193489 (exact 0.8292987121934895003).
- A17 `IMSINH("1.5+710.4i")`: Excel 0.916202079442849, ours 0.916202079442848 (exact 0.9162020794428484950,
  so Excel's double is above the exact value).
- A58 `IMSECH("1.5+711i")`: Excel -0.371302652691887, ours ...888 (exact -0.3713026526918874831).
Tried by T-2231 (none fits A9 and A17): libm sin/cos, Excel's SIN/COS reduction (`trig_sin_cos`, x87 π),
`(e^x ± e^-x)/2` and complex-exp sums for sinh/cosh. Next candidates: x87 extended products, or a check
whether IM* text rounds a halfway shortest-repr string up (`0.9162020794428485` → ...849) before refitting
kernels. Code: `imcosh`, `imsinh`, `imsech` in crates/rustsheet-engine/src/functions/engineering/complex/trig.rs.

## Acceptance criteria
- [ ] The three lines leave eval/baseline/probes.txt (`make eval-baseline`) with no new known-failure lines,
      or the candidates tried are recorded in docs/excel-semantics/engineering.md (time-box, decision 0056).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
