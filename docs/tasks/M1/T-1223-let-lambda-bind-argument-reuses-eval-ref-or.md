---
id: T-1223
title: "LET/LAMBDA bind_argument reuses eval_ref_or_index for INDEX instead of its own check"
milestone: M1
lane: quality
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The rule "an INDEX in reference context keeps its reference" has one owner: LET/LAMBDA binding uses it instead of an inline copy.

## Context
Steward spot-audit 2026-10-09 of PR #453 (T-1092, d92190e), reviewer-quality on opus: `bind_argument`
in `crates/rustsheet-engine/src/functions/logical/let.rs` (~line 196) checks
`matches!(node, Node::FunctionKind { kind: Function::Index, .. })` itself and routes INDEX through
`evaluate_reference_arg`. `Model::eval_ref_or_index` (`reference_operators.rs` ~line 318) already owns
this rule (it calls the same `index_impl(args, cell, true)`). If that helper later learns another
reference-returning form, LET/LAMBDA bindings silently miss it (directive 3). Suggested shape:
drop `is_index` and return `Binding::Value(self.eval_ref_or_index(node, cell))` when the node is not
multi-area. Check that `into_single_area` of an INDEX result gives the same `CalcResult` so nothing
changes (probes/let-bind-reference-forms, let-name-call-refs must still pass).
Same review, minor: the `#NAME?` from calling a number or boolean has an empty message
(`lambda.rs` ~59-61); give it one (e.g. "A value cannot be called") while there.

## Acceptance criteria
- [x] `bind_argument` has no INDEX special case of its own; it calls the shared helper.
- [x] Eval shows 0 regressions; the LET/LAMBDA probe cases still pass.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing which functions count as reference forms (T-1224 probes that).

## Outcome
`bind_argument` drops its inline INDEX check and reads a non-union argument through `eval_ref_or_index`. That helper also has an `@INDEX` arm (T-1247) that would have changed `LET(x,@INDEX(..),ISREF(x))` from FALSE to TRUE with no evidence, so `ImplicitIntersection` nodes keep the old path (unit test); probe filed as T-1503. Eval: 0 regressions, no cells fixed. The `#NAME?` message for calling a number was not changed: that result is built per element as an `ArrayNode::Error`, which carries no message.
