---
id: T-2296
title: "Probe SUMPRODUCT: does an error element in a range beat an earlier shape mismatch"
milestone: M5
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for which error SUMPRODUCT returns when an argument holds an error *element* (inside a range or
array, not a whole-argument error) and the argument shapes disagree.

## Context
T-2150 established (probes/t2200-blank-and-padded-m5-args A20-A23) that a whole-argument error or blank one-cell
argument beats an earlier shape mismatch. For an error element ours keeps the old order: the first mismatched argument
stops element reading, so `SUMPRODUCT(D1:D2,<2x2 with #DIV/0!>)` is `#VALUE!`, while an error element in an argument
ahead of the mismatch wins. Neither is probed (docs/excel-semantics/coercion.md "SUMPRODUCT: a blank one-cell
argument", Open line; code `fn_sumproduct` in crates/rustsheet-engine/src/functions/math_and_trigonometry/sumproduct.rs,
`TODO(T-2296)`). Use the `excel-probe` skill; suggested cells, with a 2x2 range R holding one `#DIV/0!` element and
a 2x1 range S of numbers: `SUMPRODUCT(S,R)`, `SUMPRODUCT(R,S)`, `SUMPRODUCT(S,R,NA())`, `SUMPRODUCT(R,S,NA())`,
`SUMPRODUCT(S,{1,#N/A;2,3})`, `SUMPRODUCT({1,#N/A;2,3},S)`, `SUMPRODUCT(S,R2,R)` with R2 a 2x2 of numbers.

## Acceptance criteria
- [ ] A probe case in eval/suites/probes with oracle values for the cells above.
- [ ] coercion.md entry's Open line replaced by the probed rule; an implementation task filed if ours differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing any change (file a functions-math task if needed).
