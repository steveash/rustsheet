---
id: T-1240
title: "Spec: keep written parentheses in parsed formulas so FORMULATEXT prints them back"
milestone: M1
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A spec (docs/specs/) for keeping written parentheses in the parsed formula, so the engine's formula text and FORMULATEXT print every level back as Excel does, with the evaluation unchanged; then implementation tasks cut from it.

## Context
T-1178's probe probes/at-operand-nested-groups (excel-graph, 2026-10-09) shows Excel keeps every redundant parenthesis
as written, in plain formulas (`=A1+(B1)`, `=-(A1)`, `=((A1))`, `=IF((A1>0),(B1),C1)`) and inside `@` operands
(`=@(A1+(B1))`); see docs/excel-semantics/syntax.md "FORMULATEXT keeps every written parenthesis". The engine's
`Node` tree keeps groups only in special cases (`ImplicitIntersection { written, lead }`, `GroupedRootKind`, cut/union
shapes from T-1176, the parser's `at_group`/`root_group` bookkeeping in expressions/parser/mod.rs), and the printer
re-derives the rest from precedence, so 14 FORMULATEXT cells of that probe fail (values all match). A general `Group`
node (or a written-levels count on every node) would replace those special cases, but it touches the parser, printer,
`to_string_moved`, static analysis, the evaluator (groups are transparent; `(A1)(2)` calls and `(1)` vs `1` in
intersections already depend on them, references.md), structural edits and xlsx export, so it needs a spec first
(CLAUDE.md directive 4). The spec must cover: memory per node, the nesting bound (T-1169), whether the stored xlsx text
keeps the parentheses (Excel writes them; check an Excel-authored file), and which existing special cases fold into it.
T-1200 (an `@` group that is not the whole operand) becomes one case of it.

## Acceptance criteria
- [ ] docs/specs/SPEC-XXXX (template docs/specs/TEMPLATE.md) is written and approved by the architecture and performance reviewers.
- [ ] Implementation tasks are filed from it; the end state makes probes/at-operand-nested-groups pass and flips the T-1167 test `at_operand_adds_no_level_for_a_nested_inner_group` (E40, E41, E43, E44) to Excel's text.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
