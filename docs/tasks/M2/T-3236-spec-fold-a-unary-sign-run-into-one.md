---
id: T-3236
title: "SPEC: fold a unary sign run into one parse-tree node so its depth cost is constant (for T-3193)"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A spec in docs/specs/ (docs only, no engine code) that says how the parser
represents a run of unary signs (`---x`, `+-+-A1`, up to Excel's 1,024 minus / 1,023 plus or
alternating signs) so that the run costs a constant amount of parse-tree depth, and how every consumer
of the tree keeps today's behavior. T-3193 then implements it.

## Context
T-3193 (engine, opus) was resumed on 2026-10-30 and judged bigger than size M, which its own Context
foresaw ("This may need a spec first if the node change is bigger than M"). `Parser::finish_power`
(crates/rustsheet-engine/src/expressions/parser/mod.rs) wraps one `Node::UnaryKind` (minus) or
`Node::UnaryPlus` per sign and calls `grow` for each, so `MAX_FORMULA_DEPTH` (256, depth.rs) refuses
the eighteen probes/t3172-minus-*, -plus-* and -alt-* cases pinned in probes/deviations by T-3189
(decision 0168). The two variants are matched at 63 sites in 27 files, 22 of them production code in
rustsheet-engine (node_frames.rs, stringify and its sheet-edit/rename/bound-variable walkers,
static_analysis and its legacy/redundant/operand_array passes, reference_operators.rs,
contains_function.rs, cast.rs, call_arguments.rs, spill_shape.rs, tailed_array.rs, units.rs,
let.rs, read_once.rs, t_function.rs, lifted_options.rs, model.rs). Dropping the tree and the
derived `PartialEq`/`Debug` also recurse per node (depth.rs on chains), so a folded node must hold
the signs flat, not as nested boxes. `Node` must stay within `NODE_SIZE` (80 bytes, mod.rs).

The spec should settle at least: the node's shape (e.g. the signs in written order with their
whitespace runs, or a minus count plus kept pluses and runs, behind one `Box`); how evaluation
applies it (each minus negates and coerces, so `--x` coerces `x` to a number; an even run is not the
identity for text or errors) without recursion; how `Node::ungrouped`/`Node::lowered` and the
`UnaryPlus` transparency rules (T-2723 plus-before-number folding, T-2776 run placement) carry over;
whether the single-sign case keeps today's variants (smaller diff) or every run uses the new node;
how the binding/printing in stringify.rs (`Binding::Minus`) treats a run; and how the run counts
toward Excel's caps (refuse 1,025 minus / 1,024 plus as Excel does, typed and stored; syntax.md
"Formula nesting depth cap", T-3172), with a release-stack test plan (test_nesting_depth.rs, 1 MiB).
Decision 0173 (T-3191) is the nearest precedent: directly nested parentheses are parsed in a loop
(`parse_parenthesis_run`) and one `Group` carries their count (`levels`) under Excel's own per-run
cap (`MAX_PARENTHESES`). The spec should weigh that shape (a count on the existing variants) against
a new node, with alternatives considered. It should also split T-3193's implementation into size-M
steps if one PR would be too big.

## Acceptance criteria
- [x] docs/specs/SPEC-XXXX-*.md (next free number, docs/specs/TEMPLATE.md) answers each question above,
      lists every match site to change, and names the evidence (probes/t3172-*, syntax.md).
- [x] T-3193's Context links the spec; any implementation steps beyond one M PR are filed with
      `make new-task` and T-3193 depends on what it needs.
- [x] `make check` green; reviewers approve.

## Out of scope
- Implementing the node (T-3193).
- Bare parentheses (T-3191) and function-call nesting (T-3097, T-3143).

## Outcome
Closed with no spec, because the task it was written for no longer needs one. T-3193 merged on
2026-10-30 (#2426, decision 0183) without a new node: a unary sign stopped costing a level of
`MAX_FORMULA_DEPTH`, `MAX_SIGNS` (1,023, depth.rs) bounds the open signs instead, the evaluator
already walked a run in a loop (`apply_unary_chain`) and the printer now does too
(`stringify::wrapped_text`). The eighteen probes/t3172 sign-run cases load and match Excel, so
none of the 63 match sites had to change and there is no implementation left to split.

The one question this spec would still have answered, whether to fold a run into one node to
widen the stack margin of the derived `Clone`/`Drop`/`PartialEq`/`Debug` and the rewrite walks
(about 2x headroom on a 1 MiB release stack), is T-3253's, which already weighs a folded node
against iterative `Clone`/`Drop` and asks for a measurement first. Which signs count toward
Excel's cap is T-3266's. The acceptance criteria above are met vacuously: no spec number was
taken, T-3193 needs no link, and this change is docs only.
