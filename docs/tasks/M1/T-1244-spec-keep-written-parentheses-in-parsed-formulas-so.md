---
id: T-1244
title: "Spec: keep written parentheses in parsed formulas so FORMULATEXT prints them back"
milestone: M1
lane: engine
status: done
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
It also covers probes/at-group-leftmost-shapes E34 (`=@((A1 B1) C1) D1`, a group nested in the leading `@` group) and E35
(`=@(A1):(B1) C1`, a group around a range's right corner; also probes/at-group-range-corner-whole E34, `=@(A1):(B1)`), which the engine prints without those groups (T-1251 left them here:
`LeadGroup` records one group only, and extending it would add another special case this spec should fold away).
Special cases to fold in also include `LeadGroup` itself (`of_range`: a group that is a range's left corner, `@(A1):B1 C1`
and the whole-operand `@(A1):B1`, T-1200, T-1251).

## Acceptance criteria
- [x] docs/specs/SPEC-XXXX (template docs/specs/TEMPLATE.md) is written and approved by the architecture and performance reviewers.
- [x] Implementation tasks are filed from it; the end state makes probes/at-operand-nested-groups pass and flips the T-1167 test `at_operand_adds_no_level_for_a_nested_inner_group` (E40, E41, E43, E44) and the F50, F51 asserts of `at_operand_group_leftmost_shapes_match_the_probe` (probes/at-group-leftmost-shapes E34, E35) to Excel's text.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
SPEC-0007 (docs/specs/SPEC-0007-written-parentheses.md) proposes a `Node::Group { levels, child }`.
It is transparent to evaluation through `Node::ungrouped()` and printed back as written, and it
replaces `GroupedRootKind`, the union and cut `written` counts, and the `@` `written`/`LeadGroup`/
`grouped` fields along with their parser state. New evidence (excel-graph, 2026-10-11): the stored
xlsx text keeps every written parenthesis, and `=@(Z1+(Z2))` is stored `_xlfn.SINGLE((Z1+(Z2)))`
(xlsx-storage.md). Implementation tasks filed: T-1339 (probe group transparency), T-1340 (Group
node, unblocks T-1331), T-1341, T-1342, T-1343 (fold the special cases) and T-1344 (one printer).
