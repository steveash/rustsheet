---
id: T-3691
title: "Make Node Clone, Drop and Display iterative (or bound the node count) so small-stack debug embedders cannot abort"
milestone: M2
lane: engine
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
No formula the parser accepts can overflow a small stack in any build in the derived recursive walks of `Node`
(`Clone`, `Drop`, `Display`/`Debug`, `PartialEq`), or their node count is bounded so the at-limit tests fit a 2 MiB
debug stack.

## Context
- T-3338 measured: 256 levels of spaced `- ( ` around `B1` overflow a 2 MiB stack in an unoptimized build, in the
  derived `Clone` of the stored tree (formula_cells.rs `stored_formula`), about 4,000 frames for ~1,000 nodes; the
  existing 1,023-sign spaced runs (test_nesting_depth.rs `sign_runs_at_the_cap_survive_every_walk_under_the_deepest_nesting`)
  are the same. Release builds fit 1 MiB, the size the bounds are set for; debug builds need about 16 MiB for the
  heaviest spaced shapes (`on_small_stack`).
- An embedder running an unoptimized build on a small thread could abort (directive 5).
- Options: an iterative `Clone`/`Drop` for the unary/group/spaced wrapper chains (a manual impl that loops over
  `UnaryKind`, `UnaryPlus`, `Group`, `Spaced`), or a bound on wrapper nodes per path.
- Record the debug-stack requirement in docs/architecture (or syntax.md "Formula nesting depth cap") until fixed.

## Acceptance criteria
- [ ] The heaviest shapes (spaced 256-level `-(`, 1,023 spaced signs, 200 levels plus 824 signs) set, clone,
      drop and print on a 2 MiB stack in a debug build; tests pin it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing any accepted limit.
