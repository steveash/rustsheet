---
id: T-2668
title: "Probe IMCSCH tiny results and subnormal IM* parts past the denominator overflow"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for two rules T-2335 left inferred: whether IMCSCH keeps normal tiny parts past its denominator overflow as IMSECH does, and whether a subnormal part of other IM* results prints as 0.

## Context
T-2335 made IMSEC, IMCSC and IMSECH keep normal tiny parts (trig parts times `2e^-|v|`) where the
denominator overflows (probes-numeric im-last-digit-sweep E55:I55, K42:L42), and flushed every
subnormal IM* part to 0 in the complex number-to-text (`Complex`'s Display in
functions/engineering/complex/mod.rs; evidence: IMEXP, gen-m9/b5aef3645cec1b95 A154). IMCSCH still
answers 0 (`reciprocal_result` in complex/trig.rs) and the flush is UNVERIFIED for other functions
(engineering.md "Tiny results" and the IM* last-digit sweep entry). Use the excel-probe skill.

T-3642's grid (probes/im-trig-magnitude-grid, excel-graph, 2026-11-01) answers the IMCSCH question for x = 700,
705 and 709: Excel keeps normal tiny parts where ours prints 0, in 41 cells (J134:J149, J153:J168, J172:J176, J180, J183, J184 and J186).
`IMCSCH(COMPLEX(700,0))` and `(700,1E-308)` are 1.97193530875195E-304, but `(700,1E-300)`, `(700,1E-200)` and
`(700,1E-20)` are -1.97193530875195E-304; `(700,1)` is -1.06544119434148E-304+1.65932634623297E-304i and `(700,20)`
-8.04711426558125E-305+1.80026897486721E-304i (the negation of 2e^-x(cos y, -sin y)), while `(700,709.78)` is
-1.92443257888109E-304-4.30218678399417E-305i; `(709,0)` 2.43356150124685E-308, `(709,705)` 2.33364281104964E-308i.
The sign pattern is recorded in engineering.md "IM* trig and hyperbolic functions past SIN's argument limit (T-3642)".

## Acceptance criteria
- [ ] A probe case with Excel values for `IMCSCH("-524.561+5.3113i")`, `IMCSCH("400+1i")`,
      `IMCSCH("-724.227-562.884i")` and subnormal-part cases of other functions (e.g.
      `IMDIV("1E-300+1i","1E+10")`, `IMPRODUCT("1E-200+1i","1E-120")`, `COMPLEX(1,1E-310)`).
- [ ] engineering.md records the rules as VERIFIED (or corrected); implementation follow-ups filed if ours differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing a changed IMCSCH kernel (file a follow-up).
