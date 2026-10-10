---
id: T-1226
title: "ANCHORARRAY(@1) loads in Excel as #VALUE! (prints =@1#); the parser refuses it (follow-up of T-1216)"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`=ANCHORARRAY(@1)` evaluates to `#VALUE!` and prints as `=@1#`, as Excel does, instead of being refused by the parser.

## Context
T-1117 made the parser refuse operands of `#`/ANCHORARRAY that the intersection rule refuses, so `=ANCHORARRAY(@1)` is a syntax error.
Excel loads it: probes/colon-hash-value-operands D3 is `#VALUE!` and E3 (FORMULATEXT) is `=@1#` (excel-graph, 2026-10-09, T-1216).
Failing today (known failures in eval/baseline/probes.txt): Sheet1!D3 and Sheet1!E3. D1 (`=A1:LAMBDA(x,x)`) and D2 (`=A1:(@1)`) already match.
Code: `is_reference_operand` in crates/rustsheet-engine/src/expressions/parser/reference_operators.rs. See docs/excel-semantics/references.md, the T-1216 note.

## Acceptance criteria
- [ ] probes/colon-hash-value-operands D3 and E3 pass; the baseline lines are removed with `make eval-baseline`.
- [ ] The refused forms listed in T-1216's note (`=ANCHORARRAY(1)`, `=ANCHORARRAY(SEQUENCE(3))`, `=ANCHORARRAY(A1+0)`) stay refused.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`SpillRangeOperator` over an `@` of a non-reference now evaluates to `#VALUE!` (was `#ERROR!`) and `@<number>` prints bare, `=@1#`; the refused forms stay refused. probes/colon-hash-value-operands D3 and E3 pass (2 known failures removed from eval/baseline/probes.txt). The wider print rule is UNVERIFIED; follow-up T-1240.
