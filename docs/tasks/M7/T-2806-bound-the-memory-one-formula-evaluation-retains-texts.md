---
id: T-2806
title: "Bound the memory one formula evaluation retains (texts in large arrays)"
milestone: M7
lane: engine
status: done
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
- [x] The MAKEARRAY formula above, and one other shape that fills a 2^20-cell array with long texts, either stay under a
      documented byte cap (`#ERROR!` past it, a DEVIATION entry in arrays.md) or are shown, with the measurement in the
      Outcome, to be bounded already.
- [x] No baseline gains a line.
- [x] `make check` green; reviewers approve.

## Out of scope
- Time bounds (SPEC-0024).

## Outcome
Measured first (release build, peak resident memory of a process with A1 `=REPT("a",32767)`, 6 GB address-space limit):
the MAKEARRAY formula, MAP and SCAN shapes were already bounded at 1.1 GB by SPEC-0024's text charge on each call's value
(`#VALUE!` at the work limit, about 2^30 bytes of text). But a text *copied* into many cells was not charged and was
owned bytes per cell: `IF(SEQUENCE(1024,1024),REPT("a",32767))`, the same over A1, IFERROR, IFNA, CHOOSE, INDEX, EXPAND,
TOCOL of those, and the `&` operator over an array (`SEQUENCE(1024,1024)&REPT("a",32767)`, new texts with no cap) each
aborted the process (32 GB needed). Fix, without a per-site byte cap (about 300 copy sites):
- `Text` (text.rs) holds its bytes in an `Arc`, so every copy is a pointer and an edit copies only bytes still shared.
  The copy shapes now peak at 115 MB (TOCOL 470 MB) and compute their values, as Excel would.
- The `&` operator over an array charges each text it makes to the formula's work budget (arithmetic.rs
  `handle_concatenate`), so those formulas stop at the limit: `#VALUE!` at 1.1 GB.
Documented in arrays.md "Copies of a text share its bytes" (DEVIATION) and the work-units entry; SPEC-0024's
"Retained memory" gap updated. The byte bound is the work budget's (`#VALUE!`, uncatchable), not a new `#ERROR!` cap:
one budget (directive 3). Tests: test_text_copies.rs (sharing for seven copy shapes, copy-on-write edits, the `&` charge
and its budget error). Eval delta: none (no baseline change). Follow-up: T-2948 (TEXT with a 32,767-character format
takes 22 ms a call: a time gap found while measuring).
