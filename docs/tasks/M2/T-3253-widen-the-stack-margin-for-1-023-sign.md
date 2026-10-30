---
id: T-3253
title: "Widen the stack margin for 1,023-sign runs: iterative Clone/Drop of sign runs"
milestone: M2
lane: engine
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
A formula holding a 1,023-sign run under the deepest nesting the depth bound admits keeps a
comfortable stack margin on a 1 MiB release stack in every tree walk, not just evaluation and
printing.

## Context
T-3193 made signs cost no depth (`MAX_SIGNS`, depth.rs) and made the printer peel signs, groups
and whitespace in a loop (stringify/wrapped.rs). Other walks still recurse once per sign node (and
once more per `Node::Spaced` after a spaced sign): the derived `Clone`, `Drop`, `PartialEq` and
`Debug`, static analysis and the rewrite passes. The test
`sign_runs_at_the_cap_survive_every_walk_under_the_deepest_nesting` (test_nesting_depth.rs) passes
on 1 MiB but overflows at 512 KiB, in the derived `Clone` (about 2x headroom, the architecture
review of T-3193 called it thin). Options: fold a run into one node (signs as data, as the T-3193
task context sketched), or iterative `Clone`/`Drop` for runs plus loops in the walkers that
recurse per sign. Measure before and after.

## Acceptance criteria
- [ ] The T-3193 cap test passes on a 512 KiB release stack (or the headroom is measured and
      recorded at 4x or better).
- [ ] No printed text, evaluation result or eval case changes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing `MAX_SIGNS` or Excel-visible behavior.
