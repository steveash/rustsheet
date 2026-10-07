---
id: T-1102
title: "Print an @ operand of : and # so it re-parses to the same tree"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
An `@` (ImplicitIntersection) that is an operand of `:` or `#` prints so the text re-parses to the same tree.

## Context
From the T-1098 reviews: `tight_operand` (parser/reference_operators.rs) wraps an intersection but not an `ImplicitIntersection`, so `A1:(@B1)` prints `A1:@B1` (a parse error) and `(@(A1 B1)):C3` prints `@A1 B1:C3` (a different tree); `@(@(A1 B1))` prints `@@A1 B1`. Both printers (stringify.rs, move_formula.rs) use it. These fail on main already. Excel's text for them is not probed; keep the engine's choice minimal (parenthesize).

## Acceptance criteria
- [ ] Round-trip cases for each of these formulas in test_intersection.rs: print then reparse gives the same tree.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`tight_operand` now also parenthesizes an `ImplicitIntersection` operand of `:` and `#`, and the new `at_operand` parenthesizes an `@` operand of `@` (both printers: stringify.rs, move_formula.rs). Round-trip cases `A1:(@B1)`, `(@(A1 B1)):C3` and `@(@(A1 B1))` added to test_intersection.rs. Excel's own text for these is unprobed; parenthesizing is the minimal engine choice. No eval change.
