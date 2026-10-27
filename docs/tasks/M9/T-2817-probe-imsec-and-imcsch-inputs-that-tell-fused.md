---
id: T-2817
title: "Probe IMSEC and IMCSCH inputs that tell fused from unfused denominators"
milestone: M9
lane: functions-engineering
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Know whether IMSEC's and IMCSCH's denominators fuse a product into their sum (as IMSECH's does, T-2750),
from oracle cells that tell the forms apart, and keep or change each kernel on that evidence.

## Context
T-2757 scored fused (`mul_add`) denominators on every oracle cell we have (docs/excel-semantics/engineering.md
"IM* reciprocal kernels"): fusing refutes IMCSC (sweep D33) and IMCOT (P31, P70), but for IMSEC
(`fma(cos x, cos x, (cosh 2y + 1)/2 − 1)`) and IMCSCH (`fma(cosh²x·sin y, sin y, sinh²x·cos y·cos y)` and
`fma(sinh²x·cos y, cos y, cosh²x·sin y·sin y)`) every form ties: no cell discriminates. A libm emulation
(Python 3.13 `math.fma`, `%.15g`, plain sin/cos) found inputs where the 15-digit text differs between the forms,
for example:
- IMSEC: `-1.85558-0.582736i` (real ...536377 plain, ...536378 fused), `-1.44771+0.0419247i`, `0.196417-1.11583i`,
  `-4.07921-1.12509i`.
- IMCSCH: `-0.0959678+0.00165849i`, `0.0466774-2.22015i`, `0.0853196+0.00680423i`, `-1.82906-2.23458i`.
Recompute the candidates with the engine's own kernels before probing (temporary `mul_add` switch, as T-2757 did),
then probe 20 or more discriminating inputs per function with the excel-probe skill.

## Acceptance criteria
- [x] A probes-numeric case with excel-graph values for 20 or more discriminating IMSEC and IMCSCH inputs.
- [x] Each kernel keeps or adopts the form that fits the most cells (all old cells included); engineering.md
      records the scores.
- [x] `make check` green; reviewers approve.

## Out of scope
- IMCSC and IMCOT (refuted by T-2757), IMSECH (T-2750).

## Outcome
probes-numeric `im-sec-csch-fused` (excel-graph, 2026-10-27): 34 IMSEC and 44 IMCSCH inputs (text, IMREAL and
IMAGINARY of each), picked with the engine's own kernels so that the plain and fused denominators print different
text, plus the task's 8 libm-emulation candidates. Fusing is not what Excel does for either function:
- IMSEC keeps its T-2743 plain kernel (23 of 34 texts; fused 10).
- IMCSCH moves to `t = sinh²x + sin²y` (33 of 44; the old four-product sum scored 25, its fused forms 30 and 18).
Every older oracle cell ties, and the gate suites show 0 regressions. No form fits all the cells, so T-2841 fits the
part kernels. Scores are in engineering.md "IM* reciprocal kernels". probes-numeric is not a gate suite, so the new
case's 11 IMSEC and 11 IMCSCH text misses are in no baseline.
