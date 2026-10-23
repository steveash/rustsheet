---
id: T-2440
title: "Legacy @: defined-name operands take the automatic @; probe the remaining dynamic-array-only functions"
milestone: M2
lane: engine
status: todo
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
- [ ] A defined-name operand takes the `@` as the probes show, even for a scalar or SWITCH body; a SWITCH reached only through a name does not remove the host's `@`s.
- [ ] SORTBY, XMATCH, VSTACK, DROP, TEXTSPLIT and the like are probed (`excel-probe`) and added to `NO_AUTOMATIC_AT` where they remove the `@`.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
