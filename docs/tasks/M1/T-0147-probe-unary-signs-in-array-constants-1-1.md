---
id: T-0147
title: "Probe unary signs in array constants: {--1}, {+1}, {-+1}"
milestone: M1
lane: research
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Know what Excel does with sign runs inside array constants (`={--1}`, `={+1}`, `={-+1}`, `={- 1}`)
and record it in `docs/excel-semantics/syntax.md`.

## Context
Since T-0132 the parser keeps every minus sign, so `={--1}` reaches `parse_array_item` in
`expressions/parser/lists.rs` as `Minus(Minus(1))` and is rejected ("Invalid value in array");
before T-0132 the signs were folded and it parsed as `{1}`. Neither behaviour has evidence
(fidelity review of T-0132). Use the `excel-probe` skill: one probe case
(`eval/suites/probes/cases/array-constant-signs.case.json`) with these formulas, filled by the
excel-graph oracle.

## Acceptance criteria
- [ ] Probe case with oracle sidecar committed; the rule recorded in `docs/excel-semantics/syntax.md`.
- [ ] If the engine disagrees, a follow-up implementation task filed (or the fix, if size S).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Unary signs outside array constants (T-0132).
