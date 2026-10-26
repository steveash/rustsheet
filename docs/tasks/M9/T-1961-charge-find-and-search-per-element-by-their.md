---
id: T-1961
title: "Charge FIND and SEARCH per element by their text lengths in the lift driver"
milestone: M9
lane: perf
status: done
priority: P2
size: S
tier: opus
depends_on: [T-1775]
---

## Goal
A lifted FIND, FINDB, SEARCH or SEARCHB is charged against `limits::MAX_LIFTED_CELL_VISITS` by the comparisons its elements can make, so a lift over long texts is refused before its first element instead of running up to len(within_text) x len(find_text) comparisons per element (CLAUDE.md directive 5).

## Context
T-1775 moved FIND and SEARCH onto the SPEC-0011 driver as `KernelWork::Constant` with the cap named in `functions/lift/tests/kernel_work.rs` (`FIND_SCAN`, `TODO(T-1961)`): one element compares each start position of `within_text` with `find_text`, so `FIND(REPT("a",32766)&"b",REPT("a",32767)&SEQUENCE(1000))` does about 10^9 comparisons per element. `broadcast_text` had the same cost before. `functions/text/lifted.rs` (`text_call_cost`, T-1272) already charges TEXTSPLIT, TEXTBEFORE and TEXTAFTER by their text lengths; reuse it (directive 3) through `lift_call_costed` or a `LiftCost` that the text kernels compute from their read arguments.

## Acceptance criteria
- [x] A unit test shows a lifted FIND over long texts refused with the cap's `#ERROR!` before any element runs, and a 100,000-row `=FIND("b",A{r})` column and `=SUM(FIND("a",A1:A100000))` unchanged.
- [x] The `FIND_SCAN` cap entry and its `TODO(T-1961)` are gone; a DEVIATION line in docs/excel-semantics/text.md names the cap.
- [x] `make check` green; reviewers approve.

## Out of scope
- Excel's own FIND/SEARCH values.

## Outcome
FIND/FINDB and SEARCH/SEARCHB are `KernelWork::ByValue(ValueWork::TextScan)`: each lifted element is charged its two
texts' UTF-8 lengths plus 1 and, per start position of `within_text`, 1 plus the compared bytes (the shorter length) over
256 (FIND's `memcmp`) or 8 (SEARCH's unit-by-unit wildcard segments), from release timings recorded in
`lift/kinds/text.rs`. `ScanCost`/`text_call_cost` (functions/text/lifted.rs) charge a whole call over a delimiter list
read once, so they could not price a per-element pair of lifted texts; `text_scan_units` cross-references them.
`ArgValue::Text` now carries its byte length. `FIND_SCAN` and its TODO are gone; DEVIATION in text.md "Lifted FIND and
SEARCH calls are capped"; SPEC-0011 as-built note. Eval delta: none (0 fixed, 0 regressions; a resource cap).
