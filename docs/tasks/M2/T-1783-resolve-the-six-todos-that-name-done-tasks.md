---
id: T-1783
title: "Resolve the six TODOs that name done tasks (T-0126, T-0236, T-0931, T-1114)"
milestone: M2
lane: quality
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Every `TODO(T-XXXX)` in the repository names a task that is still open. The six that name a done
task today are either resolved (the work they wait for has landed) or re-pointed to the open task
that now owns the work.

## Context
CLAUDE.md allows only `TODO(T-0123)` so that every TODO has an owner. The 2026-10-17 planner found
six whose task is `done`, so nobody will pick them up:

| Site | Names | State of the work | Fix |
|---|---|---|---|
| crates/rustsheet-engine/src/functions/util/aggregate_args.rs:67 (`skip_empty_args` doc) | T-0236 (done) | Excel evidence now exists: `AVEDEV(1,)` and `DEVSQ(1,)` count the empty argument as 0 (probes/stat-order-floor-batch A274, A299); T-1748 (open, M9) implements it; SUMSQ is unprobed | re-point to `TODO(T-1748)` |
| crates/rustsheet-engine/src/functions/statistical/count_and_average.rs:25 (`SKIP_EMPTY_ARGS` doc) | T-0236 | same | re-point to `TODO(T-1748)`, and say the evidence for AVEDEV/DEVSQ exists |
| crates/rustsheet-engine/src/functions/statistical/count_and_average.rs:67 (`for_each_value_skip_empty` doc, "no Excel evidence either way yet") | T-0236 | same; the "no evidence" sentence is now wrong | rewrite the sentence (cite probes/stat-order-floor-batch) and re-point to `TODO(T-1748)` |
| crates/rustsheet-engine/src/test/lookup_and_reference/test_fn_xmatch.rs:289 ("not Excel evidence: TODO(T-0126)") | T-0126 (done) | T-0126 probed it: probes/lookup-whole-column-blank-tail (60 cells, all passing) | compare the test's pinned values with that probe's oracle; where they agree, cite the probe and drop the TODO; if one disagrees, keep the value, keep a TODO naming a new task you file for it |
| crates/rustsheet-engine/src/worksheet.rs:695 (`used_area_end` doc) | T-1114 (done) | T-1114 made TEXTJOIN, XIRR and XNPV independent of where the clip ends (its Outcome) | rewrite the sentence to say so, drop the TODO |
| bindings/python/tests/test_facade.py:277 (`test_moderately_nested_formula`) | T-0931 (done) | the parser bounds depth at `MAX_FORMULA_DEPTH` = 256 (expressions/parser/depth.rs) | add the test the TODO asks for (a formula 10,000 levels deep raises `EditError` and leaves the cell unchanged), drop the TODO |

Comments and tests only: no engine behavior changes.

## Acceptance criteria
- [ ] For each TODO marker in crates/, bindings/, tools/ and scripts/ (`rg -o "TODO\(T-[0-9]+\)"`), the task it names has
      `status:` todo, in-progress or blocked (a one-line shell or Python loop over the ids; paste its empty
      output in the PR).
- [ ] The new Python test exists and passes under `make py-bindings` (part of `make check`).
- [ ] `make eval`: 0 fixed, 0 regressions in every suite.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing T-1748 (the empty-argument rule itself) or probing SUMSQ.
- A check that enforces this in CI (T-1779).
- `FIXME` comments and bare TODOs.
