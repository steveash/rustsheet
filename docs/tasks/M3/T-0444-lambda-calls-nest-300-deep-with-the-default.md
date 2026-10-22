---
id: T-0444
title: "LAMBDA calls nest 300 deep with the default stack budget"
milestone: M3
lane: engine
status: in-progress
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
With the default evaluation stack budget, LAMBDA calls nest at least 300 deep in a release build, as Excel
evaluates them (probes/lambda-call-depth H3, H6, I3, I6 pass and leave `eval/suites/probes/deviations`), and
nested calls still stop as `#NUM!` before they can overflow the stack (decision 0080).

## Context
Follow-up of T-0439 (Excel-fidelity review). Nested LAMBDA calls may use `LAMBDA_STACK_SHARE` (4) times
the evaluation's stack budget (`Model::lambda_stack_base`, functions/logical/lambda.rs; decision 0080), which
holds about 200 levels in a release build and 50 in a debug build: each call takes some 5 KB of stack in
release (20 KB in debug). Excel evaluates 300 levels of both a self-call through a parameter and a REDUCE
closure chain (docs/excel-semantics/arrays.md "How deep LAMBDA calls nest"), so the 300-level cells are
listed deviations. Two ways in: make a LAMBDA call frame smaller (measure which frames of
`call_prepared` / `evaluate_node_in_context` / the function dispatch are large, box or outline the big
locals), or raise the default budget where the host stack allows it (the eval harness runs on 256 MB;
an embedder's default thread is 2 MB, so a larger default needs a decision amending 0080).

## Acceptance criteria
- [ ] probes/lambda-call-depth H3, H6, I3, I6 pass in release with the default budget; their lines are
      removed from `eval/suites/probes/deviations` and arrays.md "How deep LAMBDA calls nest" is updated.
- [ ] A self-call 20,000 deep is still `#NUM!` on a 2 MB thread, without a stack overflow.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Excel's own depth limit past 300 (unknown: deeper probe workbooks fail to open in Excel for the web).
