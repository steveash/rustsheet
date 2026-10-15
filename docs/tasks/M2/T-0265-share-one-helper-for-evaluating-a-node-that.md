---
id: T-0265
title: "Share one helper for evaluating a node that keeps INDEX/IF references"
milestone: M2
lane: quality
status: in-progress
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
