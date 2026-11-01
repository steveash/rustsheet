---
id: T-2914
title: "Probe IMCSCH of a large real number (y = 0) past the inverse_exp_sinh_cosh band: IMCSCH('400'), IMCSCH('710')"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
docs/excel-semantics/engineering.md records Excel's IMCSCH of a large real number with a zero imaginary part, past
`inverse_exp_sinh_cosh`'s band (|x| > 20), from excel-graph evidence, and the engine matches it or a task owns the gap.

## Context
Carried over from T-2841 and T-2884 (T-2884 fitted IMSEC and left this probe undone). Past the band IMCSCH's parts
are libm's, and only `IMCSCH("710.4+0i")` (probes/eng-im-overflow-edges A64) covers the overflowed `t`. Probe with the
`excel-probe` skill: `IMCSCH("400")`, `IMCSCH("-400")`, `IMCSCH("710")`, `IMCSCH("709.5")`, `IMCSCH("25")`,
`IMCSCH("100")`, and the same with `+0i`. Code: `imcsch` in crates/rustsheet-engine/src/functions/engineering/complex/trig.rs.

T-3642's grid (probes/im-trig-magnitude-grid J134, J153, J172, excel-graph) has the real rows: `IMCSCH(COMPLEX(700,0))`
1.97193530875195E-304, `(705,0)` 1.32867955959959E-306, `(709,0)` 2.43356150124685E-308; ours 0. The complex rows of
the same x are listed in T-2668.

## Acceptance criteria
- [ ] A probes case holds the inputs above with excel-graph values.
- [ ] engineering.md "IM* reciprocal kernels" records the result; any mismatch is fixed here (if size S) or has a task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- IMSEC, IMCSC, IMCOT and IMSECH kernels.
