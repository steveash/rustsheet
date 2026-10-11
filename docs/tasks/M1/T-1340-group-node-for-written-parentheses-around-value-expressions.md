---
id: T-1340
title: "Group node for written parentheses around value expressions (SPEC-0007 step 1)"
milestone: M1
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-1339]
spec: docs/specs/SPEC-0007-written-parentheses.md
---

## Goal
Written parentheses around value expressions survive parse → print (FORMULATEXT, formula text, moved text, export), through a `Node::Group { levels, child }` that evaluation sees through.

## Context
SPEC-0007 Rollout step 1. Add the variant, `Node::ungrouped()`, the `evaluate_node_in_context` arm, `ungrouped()` at every shape-matching site (list them in the PR), the walker arms (static_analysis.rs, bound_variables.rs, units.rs, rename walkers), the printer (stringify and to_string_moved; a `Group` is an atom; a left `^` operand prints bare) and a `size_of::<Node>()` const assertion. The parser emits `Group` only where it drops the group today. The five special cases (`GroupedRootKind`, union/cut `written`, the `@` fields) stay as they are. This unblocks T-1331 (probes/power-chain-text I11, I26, I41, I46 on its branch).

## Acceptance criteria
- [ ] The plain-operator FORMULATEXT cells of probes/at-operand-nested-groups pass and leave the baseline (`=A1+(B1)`, `=-(A1)`, `=((A1))`, `=IF((A1>0),(B1),C1)`, ...). No `@` text changes.
- [ ] T-1331's power-chain-text cells pass (merge T-1331's probe case in, or land after it).
- [ ] T-1169 depth tests pass unchanged; `make eval` wall time within noise (reported in the PR); 0 regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
