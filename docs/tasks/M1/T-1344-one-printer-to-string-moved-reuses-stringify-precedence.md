---
id: T-1344
title: "One printer: to_string_moved reuses stringify precedence (SPEC-0007 step 5)"
milestone: M1
lane: engine
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-1340]
spec: docs/specs/SPEC-0007-written-parentheses.md
---

## Goal
Cut/paste and move print the same text as FORMULATEXT: `to_string_moved` reuses `stringify`'s precedence rules through a reference-rewriting hook.

## Context
SPEC-0007 Rollout step 5. move_formula.rs has its own `moved_operator_text` (its Power, Sum, Concat and Compare arms never wrap) and unary table, ignores `OpIntersectKind.written`, and prints `SpillRangeOperator` and `LambdaCallKind` callees differently from stringify.

## Acceptance criteria
- [ ] move_formula.rs has no precedence table of its own.
- [ ] A test over the probe corpus: moving a formula by (0,0) prints the same text as `to_english_string`.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
