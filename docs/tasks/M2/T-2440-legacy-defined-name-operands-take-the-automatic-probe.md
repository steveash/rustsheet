---
id: T-2440
title: "Legacy @: defined-name operands take the automatic @; probe the remaining dynamic-array-only functions"
milestone: M2
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A legacy cell's automatic `@` matches Excel for a defined name as an operand, and the other dynamic-array-only functions are classified.

## Context
Split from T-2419. Failing cells: probes/legacy-at-defined-name-dynamic-funcs A20, D20, G11, G12, G20, J10, J20, M20, P20, V11, V12, V20; semantics in docs/excel-semantics/references.md "Dynamic-array-only functions and LET, LAMBDA and names"; code in `static_analysis/legacy.rs` (`NO_AUTOMATIC_AT`) and `static_analysis.rs` (`DefinedNameKind`). Needs a name's body shape, hence M8 name handling.

## Acceptance criteria
- [x] A defined-name operand takes the `@` as the probes show, even for a scalar or SWITCH body; a SWITCH reached only through a name does not remove the host's `@`s.
- [x] SORTBY, XMATCH, VSTACK, DROP, TEXTSPLIT and the like are probed (`excel-probe`) and added to `NO_AUTOMATIC_AT` where they remove the `@`.
- [x] Probe the shapes T-2419 implements without evidence: a top-level legacy `=LET(x,Z30:Z32,x)`, a range bound through a nested LET, a grouped LET body, and a LET-bound or named lambda call (`=ABS(Z30:Z32)+LET(f,LAMBDA(x,x),f(1))`, which `removes_automatic_at` does not cover).
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
A defined name in a legacy value position takes the automatic `@` whatever its body (the old test only matched a body ending in
`:A9`-style text, so `$`-anchored ranges were missed too), stays whole as an IF arm, and a defined-name lambda's call takes the `@`
itself; a LET-bound lambda's call removes every `@` (`calls_bound_variable`). A new probe, probes/legacy-at-dynamic-funcs-2 (116 cells,
excel-graph, 2026-10-23), classified 30 dynamic-array-era functions: 18 join `NO_AUTOMATIC_AT`, and TEXTBEFORE, TEXTAFTER and REGEXTEST
over scalars now have a scalar static shape (no `@` on the call). It also confirmed T-2419's LET shapes. references.md "Dynamic-array-only
functions and LET, LAMBDA and names" records it all.
Eval delta: probes fixed 12 (legacy-at-defined-name-dynamic-funcs, every owned cell), new case legacy-at-dynamic-funcs-2 116/116,
regressions 0.
