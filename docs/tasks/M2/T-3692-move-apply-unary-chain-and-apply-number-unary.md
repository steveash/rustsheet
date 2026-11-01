---
id: T-3692
title: "Move apply_unary_chain and apply_number_unary out of cast.rs (at the 1,000-line limit)"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
crates/rustsheet-engine/src/cast.rs is back under the 1,000-line limit with room to grow: the unary chain
evaluation lives in a module of its own.

## Context
- After T-3338 cast.rs is exactly 1,000 lines (scripts/check_file_size.py fails above 1,000), so the next change to
  it fails the ratchet.
- `Model::apply_unary_chain` and `Model::apply_number_unary` (the unary `-`/`%` chain, its tiny-subnormal rule and the
  element-wise lift) are cohesive and separate from the casting helpers; move them to e.g. `unary_chain.rs`, a pure
  move with no behaviour change.

## Acceptance criteria
- [ ] The two functions and their docs move unchanged; callers compile; cast.rs well under 1,000 lines.
- [ ] `make check` green (eval unchanged); reviewers approve.

## Out of scope
- Any change to the unary semantics.
