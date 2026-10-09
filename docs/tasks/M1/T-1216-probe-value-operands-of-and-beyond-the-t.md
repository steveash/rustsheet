---
id: T-1216
title: "Probe value operands of : and # beyond the T-1111 forms"
milestone: M1
lane: research
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence on whether Excel loads value operands of `:` and of `#`/ANCHORARRAY that T-1111 did not probe, so the engine's inferred refusals (T-1117) are VERIFIED or corrected.

## Context
T-1117 made the parser refuse every operand of `:`, `#` and ANCHORARRAY that the intersection rule
(`is_reference_operand`, parser/reference_operators.rs) refuses. Only `=A1:(1/0)`, `=(1/0):(A1 B1)`,
`=(1/0):(A1 A2)` and `=ANCHORARRAY(1)` were probed (T-1111, one scratch workbook per formula). Unprobed and
marked UNVERIFIED in docs/excel-semantics/references.md ("FORMULATEXT keeps the parentheses of an `@`
operand..."): `=A1:"a"`, `=A1:#N/A`, `=#N/A:A1`, `=A1:SUM(B1)`, `=A1:IFERROR(B1:B2,0)`, `=ANCHORARRAY(SEQUENCE(3))`,
`=ANCHORARRAY(A1+0)`, and the negated references `=(-A1):B1`, `=A1:(-B1)` and `=(-A1)#` (the union accepts `(-A1,B1)`). Use the `excel-probe` skill; a refused workbook (`FileCorruptTryRepair`) is the expected
evidence for a refusal, one scratch workbook per formula.

## Acceptance criteria
- [ ] Each listed formula is recorded as loading (with its value, as a probes case) or refused, in references.md.
- [ ] Any form Excel loads that the engine refuses gets an engine follow-up task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Steward note (2026-10-09)
Raised to P2 by the weekly spot-audit of PR #464 (T-1117): T-1117 already refuses these forms in the
parser, so an Excel-authored workbook holding one that Excel loads imports here as `#ERROR!`. Probe
the negated operands first (`=(-A1):B1`, `=A1:(-B1)`, `=(-A1)#`): Excel loads negated union operands
(probes/union-neg-operand), so the inference is weakest there. Also add the forms the parser still
accepts without evidence: `=A1:LAMBDA(x,x)`, `=A1:(@1)`, `=ANCHORARRAY(@1)`.
