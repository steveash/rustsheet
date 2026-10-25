---
id: T-0475
title: "Spec: conditional formats evaluated on first style read"
milestone: M11
lane: perf
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A reviewed spec for evaluating conditional formats on the first style read instead of at load and
at every `evaluate()`: the public receivers that change, and what the change can and cannot be seen
to do.

## Context
T-0468 asked for a short spec if the public receivers change. They do:
`Model::get_extended_style_for_cell` and `UserModel::get_extended_cell_style` need `&mut self`. The
measure is corpus-bench 41978, which spends 184 s at load and 191 s per `evaluate()` in
`evaluate_conditional_formatting` (T-0430).

## Acceptance criteria
- [ ] docs/specs/SPEC-0021-conditional-formats-on-first-style-read.md covers the flag, the
      receivers, every caller, when the evaluation happens and what it can see, the alternatives
      and the tests; T-0468 points at it.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
