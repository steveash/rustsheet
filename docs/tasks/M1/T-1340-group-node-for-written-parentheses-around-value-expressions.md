---
id: T-1340
title: "Group node for written parentheses around value expressions (SPEC-0007 step 1)"
milestone: M1
lane: engine
status: in-progress
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

## Progress (opus run 2026-10-11, not started in code)
Claimed late in a session and released for lack of time: no code on this branch yet. Sizing for the
next run (same session found it is a full M, budget at least 2-3 hours):
- Dispatch entry points to peel (`let node = node.ungrouped();`): model.rs `evaluate_node_in_context`
  (593) and `evaluate_node_with_reference` (265); functions/util.rs `evaluate_reference_arg` (210);
  reference_operators.rs `eval_ref_or_index` (373), `spill_anchor_operand` (406), `range_operand` (432).
- Shape-match candidates in functions/, model.rs, reference_operators.rs, units.rs (grep of
  `Node::<shape>Kind` in `matches!`/`=>`/`if let`; mark each "peeled" or "not an operand" in the PR):
  - model.rs:342  `Node::ImplicitIntersection { child, .. } => {`
  - model.rs:356  `Node::DefinedNameKind((name, scope, _)) => {`
  - reference_operators.rs:342  `let error = if matches!(child, Node::ImplicitIntersection { .. }) {`
  - reference_operators.rs:451  `Node::GroupedRootKind { child } => self.range_operand(child, cell),`
  - reference_operators.rs:452  `Node::DefinedNameKind(..) if self.defined_name_formula(node).is_some()`
  - reference_operators.rs:600  `Node::DefinedNameKind(_) => self`
  - reference_operators.rs:603  `Node::OpIntersectKind { left, right, .. } => {`
  - reference_operators.rs:711  `Node::OpUnionKind { left, right, .. } => {`
  - reference_operators.rs:733  `Node::OpUnionKind { left, right, .. } => self.union_areas(left, right,`
  - reference_operators.rs:734  `Node::OpIntersectKind { left, right, .. } => {`
  - reference_operators.rs:837  `Node::FunctionKind { kind, args } => self.evaluate_reference_form(kind`
  - units.rs:181  `Node::FunctionKind { kind, args } => self.compute_function_units(kind,`
  - units.rs:190  `Node::WrongReferenceKind { .. } => None,`
  - units.rs:193  `Node::OpRangeKind { .. } => None,`
  - units.rs:194  `Node::OpIntersectKind { .. } | Node::OpUnionKind { .. } => None,`
  - units.rs:200  `Node::ArrayKind(_) => None,`
  - units.rs:201  `Node::DefinedNameKind(_) => None,`
  - units.rs:206  `Node::ImplicitIntersection { .. } => None,`
  - units.rs:208  `Node::GroupedRootKind { child } => self.compute_node_units(child, cell`
  - functions/math_and_trigonometry/product.rs:39  `} => &Node::RangeKind {`
  - functions/util.rs:216  `Node::OpUnionKind { .. } | Node::OpIntersectKind { .. } if self.is_mul`
  - functions/util.rs:226  `| Node::OpRangeKind { .. } => self.evaluate_node_with_reference(node, `
  - functions/util.rs:230  `Node::DefinedNameKind(_) => return self.defined_name_reference(node, c`
  - functions/util.rs:280  `if let Node::FunctionKind { kind, args } = node {`
  - functions/text/concatenate.rs:345  `if matches!(arg, Node::ReferenceKind { .. })`
  - functions/information/mod.rs:141  `Node::ReferenceKind { .. } | Node::RangeKind { .. } => true,`
  - functions/information/mod.rs:144  `Node::OpRangeKind { left, right } => {`
  - functions/information/mod.rs:148  `Node::GroupedRootKind { child } => self.is_reference(child, cell),`
  - functions/information/mod.rs:165  `Node::FunctionKind { kind, args } => {`
  - functions/information/mod.rs:176  `Node::DefinedNameKind((name, scope, _)) => {`
  - functions/information/mod.rs:376  `Node::DefinedNameKind((name, scope, _)) => {`
  - functions/logical/let.rs:37  `Node::FunctionKind { kind, args } => {`
  - functions/logical/let.rs:77  `| Node::GroupedRootKind { child } => {`
  - functions/logical/let.rs:82  `| Node::OpUnionKind { left, right, .. } => {`
  - functions/logical/let.rs:206  `let is_picked = matches!(node, Node::FunctionKind { kind, .. } if kind`
  - functions/aggregate.rs:172  `if !matches!(k, Node::RangeKind { .. }) {`
  - functions/logical/and_or_xor_not.rs:147  `if !matches!(arg, Node::ReferenceKind { .. }) {`
  - functions/statistical/count.rs:23  `if !matches!(arg, Node::ReferenceKind { .. }) =>`
  - functions/statistical/count.rs:34  `if !matches!(arg, Node::ReferenceKind { .. })`
  - functions/statistical/mode_functions.rs:37  `error @ CalcResult::Error { .. } if matches!(arg, Node::OpUnionKind { `
  - functions/statistical/mode_functions.rs:43  `CalcResult::String(_) if !matches!(arg, Node::ReferenceKind { .. }) =>`
  - functions/statistical/mode_functions.rs:50  `CalcResult::EmptyCell if matches!(arg, Node::ReferenceKind { .. }) => `
  - functions/statistical/chisq.rs:355  `let by_reference = matches!(node, Node::ReferenceKind { .. });`
  - functions/statistical/count_and_average.rs:33  `if !matches!(arg, Node::ReferenceKind { .. }) =>`
  - functions/statistical/count_and_average.rs:38  `if !matches!(arg, Node::ReferenceKind { .. }) =>`
  - functions/statistical/count_and_average.rs:117  `if matches!(arg, Node::ReferenceKind { .. }) =>`
  - functions/statistical/count_and_average.rs:262  `if let Node::ReferenceKind { .. } = arg {`
  - functions/statistical/variance.rs:97  `Node::OpUnionKind { .. } => self.evaluate_reference_arg(arg, cell),`
- Not yet listed: the parser predicates (`is_reference_operand`, `is_union_operand`, `has_negated_operand`,
  `is_constant`, `chain_operands`, `cuts_written_union`) and the exhaustive walkers (static_analysis.rs,
  bound_variables.rs, units.rs, rename walkers, stringify, move_formula.rs).
