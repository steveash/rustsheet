---
id: T-2806
title: "Bound the memory one formula evaluation retains (texts in large arrays)"
milestone: M7
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-2803]
---

## Goal
One formula evaluation cannot hold more than a fixed number of bytes of values at once. Today the per-value caps allow
an array of 2^20 cells, each holding a 32,767-unit text: about 64 GB in principle.

## Context
Recorded as a known gap in SPEC-0024 ("Known gaps: retained memory"). The SPEC-0024 work budget bounds time, not retained
memory. The existing per-value caps are arrays.md "Array results above 1,048,576 cells are #ERROR!", `cast::MAX_TEXT_UNITS`,
`MAX_CAPTURED_CELLS` and the nested-array caps, plus the 64 MiB `within_text_budget` per broadcast (functions/text/util.rs).
None bounds the text bytes of an array built from separate calls, for example
`MAKEARRAY(1024,1024,LAMBDA(r,c,REPT("a",32767)))`. First measure what that formula allocates today in a release build.
Then decide whether a per-formula byte cap is needed, and where it is charged (MAKEARRAY/MAP/SCAN results, `broadcast_text`,
lifted text kernels). If the work is larger than M, write a spec instead.

## Acceptance criteria
- [ ] The MAKEARRAY formula above, and one other shape that fills a 2^20-cell array with long texts, either stay under a
      documented byte cap (`#ERROR!` past it, a DEVIATION entry in arrays.md) or are shown, with the measurement in the
      Outcome, to be bounded already.
- [ ] No baseline gains a line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Time bounds (SPEC-0024).
