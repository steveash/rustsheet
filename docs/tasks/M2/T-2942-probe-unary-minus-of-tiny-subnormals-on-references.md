---
id: T-2942
title: "Probe unary minus of tiny subnormals on references, ranges, arrays, chains and negative operands"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for unary minus of a tiny subnormal (below 2^-1026) in the forms T-2771 left unprobed. The engine's
`apply_unary_chain` then matches it, or the rule in numbers.md is narrowed.

## Context
- T-2771 found that `-x` of a positive inline subnormal x below 2^-1026 is +x, while `0-x` keeps the sign
  (numbers.md "Unary minus of a tiny subnormal"; probes/t2771-roundup-subnormal-sign-controls).
- The engine applies that rule to the parity of a whole chain and to every operand kind (cast.rs `apply_unary_chain`).
  Only inline parenthesized operands were probed.
- Build a tiny value as `2^-1022*(1+k*2^-52)-2^-1022` (k < 2^48), and its negative as `2^-1022-2^-1022*(1+k*2^-52)`.
  Graph shows a subnormal as 0, so read every result back through `*2^100`.

## Acceptance criteria
- [ ] One excel-graph probe case reads back, at k = 12345 and k = 2^48 - 1 (and at k = 2^48 as a control):
      `-A1` and `--A1` with A1 holding the value, `-x` of a tiny negative x, `--x`, `---x`, `-{x}` over an array,
      `-A1:A2` lifted over a range, and `-x%` and `-(x)%`.
- [ ] numbers.md records the result; the engine matches every cell, or the mismatches stay known failures with this task as owner.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The ROUND family's own tiny-subnormal rules (T-2771, done).
