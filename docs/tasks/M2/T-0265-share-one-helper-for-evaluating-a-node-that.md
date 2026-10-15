---
id: T-0265
title: "Share one helper for evaluating a node that keeps INDEX/IF references"
milestone: M2
lane: quality
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
One shared helper evaluates a node while keeping INDEX/IF references.

## Context
`Node::FunctionKind{Function::Index} => index_impl(args, cell, true)` now appears in `evaluate_aggregate_arg` (functions/util.rs), `fn_isref` (information/mod.rs) and `lookup_index_arg` (lookup_and_reference/index_arg.rs, T-0251). Find the common shape and reuse it without changing behavior.

## Acceptance criteria
- [ ] The three call sites share one definition; no behavior change (eval and tests identical).
- [ ] `make check` green; reviewers approve.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Added `Model::index_reference` (lookup_and_reference/index.rs) and used it at all four sites that evaluated an INDEX node in reference form (`evaluate_aggregate_arg`, `eval_ref_or_index`, `range_operand`, the INDEX dispatch). The ISREF and lookup-index sites already went through `eval_ref_or_index`. No behavior change; eval delta 0 fixed, 0 regressed.
