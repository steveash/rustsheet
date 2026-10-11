---
id: T-1340
title: "Group node for written parentheses around value expressions (SPEC-0007 step 1)"
milestone: M1
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: [T-1339]
spec: docs/specs/SPEC-0007-written-parentheses.md
---

## Goal
Written parentheses around value expressions survive parse → print (FORMULATEXT, formula text, moved text, export), through a `Node::Group { levels, child }` that evaluation sees through.

## Context
SPEC-0007 Rollout step 1. Add the variant, `Node::ungrouped()`, the dispatch peel in `evaluate_node_in_context` (its exhaustive `Group` arm re-peels, never recurses or panics), a peel at operand dispatch and at every operand/argument shape test (SPEC-0007 "Seeing through groups": every match on any `Node` variant applied to an operand, including the `ReferenceKind`/`RangeKind` argument tests; list each site in the PR); chain primitives never peel, the walker arms (static_analysis.rs, bound_variables.rs, units.rs, rename walkers), the printer (stringify and to_string_moved; a `Group` is an atom; a left `^` operand prints bare) and a `size_of::<Node>()` const assertion. The parser emits `Group` only where it drops the group today and no special case records it. The five special cases (`GroupedRootKind`, union/cut `written`, the `@` fields) stay as they are. This unblocks T-1331 (probes/power-chain-text I11, I26, I41, I46 on its branch).

## Acceptance criteria
- [ ] The plain-operator FORMULATEXT cells of probes/at-operand-nested-groups pass and leave the baseline (`=A1+(B1)`, `=-(A1)`, `=((A1))`, `=IF((A1>0),(B1),C1)`, ...). No `@` text changes.
- [ ] Merge T-1331's branch (probe case probes/power-chain-text, baseline lines, references.md rule) in; its cells I11, I26, I41, I46 pass and leave the baseline; close T-1331 as done by T-1340.
- [ ] T-1110/T-1169 stack tests pass unchanged plus an alternating `SUM((SUM((…))))` shape (debug times reported); `clone()` and print round-trip `=((A1+B1)+C1)*D1`; `make eval` wall time within noise (reported in the PR); 0 regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`Node::Group { levels, child }` (one node per run of directly nested parentheses) with `Node::ungrouped()`, a
`size_of::<Node>()` const assertion (80 bytes), a dispatch peel in `evaluate_node_in_context`,
`evaluate_node_with_reference`, `evaluate_reference_arg`, `eval_ref_or_index`, `spill_anchor_operand`, `range_operand`
and `compute_node_units`, and a peel at every operand/argument shape test (site list in the PR). The parser emits a
`Group` in `parse_parenthesized` except where a special case records the levels (whole formula, union/cut `written`,
the `@` fields), so `@` bookkeeping is untouched. The printer treats a `Group` as an atom and prints a left `^`
operand bare. T-1331's branch (probes/power-chain-text and its references.md rule) is merged; T-1331 is done by this
task.

Eval delta: probes 2699 -> 2679 known failures (20 fixed, 0 regressions), plus the new power-chain-text case passing
all 16 cells: at-operand-nested-groups F10, F18, F30-F35, F37; at-group-leftmost-shapes E35;
at-group-range-corner-whole E34; at-power-text I26; written-parens-reference-context H10, H11, H14;
written-parens-reference-only F30, F40, F50, F60, F75. Still failing, T-1343's: at-operand-nested-groups F11, F13,
F14 and at-group-leftmost-shapes E34 (a group that starts an `@` operand).

Timing: the T-1110/T-1169 stack tests (now with `SUM((SUM((…))))`) take 0.20-0.27 s in a debug build (main
0.20-0.28 s); `rustsheet-eval run` 1.4-2.1 s (main 1.7-2.1 s), within noise.
