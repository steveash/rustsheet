---
id: T-3724
title: "COUNTIFS equality over a block of an indexed range bisects its run (rank-with-ties idiom, calipers pv26_002)"
milestone: M11
lane: perf
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A one-criterion number-equality COUNTIFS over a block cut from an indexed cached range (a growing
prefix `Z$11:Z11`) costs a bisection per group of identical numbers. It no longer costs a visit of
every equal number in the whole range. The counts do not change.

## Context
Calipers `pv26_002` ranks each column with ties as
`COUNTIFS(Z$11:Z$1450,">"&Z11)+1+COUNTIFS(Z$11:Z11,Z11)`, which is about 272,000 calls over 1,440 rows.
The first call counts over the whole range and is answered from the index's sorted run without visiting
it (T-0465). The second counts over a prefix window. `ValueIndex::count` then walked every match in the
*whole* range and kept the ones inside the window, and it walked every non-number cell as well. With
many ties, each column's prefix counts were quadratic. `ValueIndex::count` was 15 of 118 stack samples
of the release evaluation (2026-11-01, on top of T-3687).

`ValueIndex::numbers` is sorted by (value, position), so positions ascend within each group of
bitwise-identical numbers. `others` is pushed in position order.

## Acceptance criteria
- [x] Over a window, `ValueIndex::count` with an `Eq` number criterion counts each identical-number
      group of the equality run by bisecting positions (`positions_in_window`). It tests only the
      `others` inside the window (`others_in`, used by the whole-range path too). Other operators
      over a window walk as before.
- [x] A test compares growing prefixes and inner blocks of an indexed range (ties, values equal at
      15 digits but not bitwise, negative zero, texts and blanks) with the pass caches off. It fails
      when either bisection bound or the window cut of `others` is mutated.
- [x] calipers `pv26_002` release `evaluate()` before and after, in the Outcome.
- [x] `make check` green; reviewers approve.

## Out of scope
- Windowed `<`, `>`, `<>` counts: their runs span many values, so a positional structure such as a
  merge-sort tree would be needed. Nothing measured asks for that yet.

## Outcome
`ValueIndex::count` gains the windowed-equality path, plus `positions_in_window` and
`ValueIndex::others_in`. Each equality group is bisected, and an equality run holds only the few
numbers equal to the target at 15 digits. Calipers `pv26_002` release `evaluate()`: 7.4–7.6 s before
(T-3687), 6.8–7.1 s after.
