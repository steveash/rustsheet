---
id: T-1284
title: "GEOMEAN(5,7) is 1 ulp off Excel (5.916079783099616, engine ...615)"
milestone: M1
lane: functions-stat
status: blocked
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`GEOMEAN(5,7)` returns Excel's exact value 5.916079783099616.

## Context
`=GEOMEAN(5,7)` is 5.916079783099616 in Excel (probes/subtotal-reference-operands-d H28; excel-graph) and the engine gives 5.916079783099615. Look at `functions/statistical/geomean.rs` for how the product/log is accumulated and find the order of operations that reproduces Excel's bits across the existing GEOMEAN cases.

## Acceptance criteria
- [ ] `GEOMEAN(5,7)` matches Excel bit-exactly, with a probe case added (or the -d case extended) as evidence and no existing GEOMEAN case regressing.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Blocked because
GEOMEAN(5,7) is one of the known in-range misses with no reproducible root model
(docs/excel-semantics/numbers.md, "GEOMEAN in-range root"). The engine's r = exp(ln(35)/2) is
5.916079783099615, and Excel gives the correctly rounded sqrt(35) = 5.916079783099616. Any fix
that picks the correctly rounded root for n = 2 breaks GEOMEAN(1,3) (ironcalc-calc GEOMEAN C23):
there Excel keeps r = 1.7320508075688774 over the correctly rounded ...772, although both cases are
n = 2 with r 0.5-0.6 ulp high. The same contradiction appears between geomean-root-refinement A36
and C23. T-0288 scored the models already tried (Newton steps, glibc pow, neighbour search,
scaling) on 167 cells, and each breaks cells that r matches. Unblock when a root model fits C23,
A36 and this cell together. That likely needs Excel's own power or root routine, which is
desktop-only evidence (M14, decision 0039) or a new idea from T-0221 (`^` is not correctly
rounded). No probe was added: the existing evidence (probes/subtotal-reference-operands-d H28) is
enough to state the conflict.
