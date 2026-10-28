---
id: T-0491
title: "SPEC-0020 amendment: bounded range leaves and ROW position leaves in the tailed reading"
milestone: M11
lane: perf
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
An element-wise aggregate or SUMPRODUCT argument over single-column ranges that run far past the used area
(`Report!$A$2:$A$70000`), and over `ROW()` of such a range, is read as a head and a tail, with bit-identical
results; a tail row that depends on ROW() falls back to today's path.

## Context
SPEC-0020 Amendment 1, sections 1 and 2 (docs/specs/SPEC-0020-whole-column-array-operands.md): bounded leaves share one span (`top`, `bottom`), the head
ends at `clamp(used_end, top, bottom)`, `tail_len = bottom - H` must reach `MIN_TAILED_ROWS`; `ROW(r)` over a
leaf range is a position leaf whose tail is the sentinel `{#N/IMPL}`, and a sentinel tail element makes
`Tailed::elements` return `None`. Code: `crates/rustsheet-engine/src/tailed_array.rs` (`tailed_leaves`,
`read_tailed`, `tailed_operand`, `Tailed::elements`). Needed by T-0492 (corpus-bench 59969).

## Acceptance criteria
- [x] `tailed_leaves` takes single-column ranges of any one common span and `ROW(<leaf range>)`; mixed spans,
      `ROW()` without an argument and COLUMN stay refused.
- [x] Equivalence tests in `test/test_tailed_array.rs` (tailed vs `TAILED_OFF`) for spans starting above, inside
      and below the used area, a ROW tail picked (fallback) and not picked (tailed), and a real `#N/IMPL` in the
      tail (fallback), over SUM, COUNT, MIN and SUMPRODUCT.
- [x] `rustsheet-eval check` 0 regressions; existing tailed tests unchanged.
- [x] `make check` green; reviewers approve.

## Out of scope
- SMALL/LARGE (T-0492), UNIQUE (T-0493), LOOKUP (T-0494).

## Outcome
`tailed_leaves` now accepts single-column ranges of any one common span, plus `ROW(<leaf range>)` as a position
leaf. The head ends at `clamp(used_end, top, bottom)`, and the tail stands for `bottom - H` rows. In the head,
a position leaf evaluates to its head row numbers. In the tail it evaluates to the `#N/IMPL` sentinel, and
`Tailed::elements` refuses a sentinel tail element, so the argument is read the ordinary way. The hook for
ROW sits in `evaluate_node_in_context`'s `FunctionKind` arm.

For range operands, `tailed_operand` now calls `clip_block_to_used_area`. A bounded range's past-used-area
cells are recorded as one rectangle, where `evaluate_range` would read and record them one by one. I
corrected the amendment's "Records" bullet to say so.

Tests in `test_tailed_array.rs`, each comparing the tailed result with `TAILED_OFF`:
- 12 tailed forms: SUM, COUNTA, MIN, COUNT and SUMPRODUCT, with spans that start inside, at the end of and
  below the used area, on another sheet, and with ROW in a branch. One has a real `#N/IMPL` tail (H4 is
  `FORECAST.ETS`, which is not implemented) and no ROW, which stays tailed as an ordinary tail error.
- 11 refused forms: mixed spans, a ROW-dependent tail, ROW in a condition, a real `#N/IMPL` tail beside ROW
  (taken for the sentinel), ROW with no range beside it, too few tail rows, and COLUMN.
- A spill into a bounded tail.
- SUMPRODUCT of bounded arguments with different first rows.

`model.rs` shrinks by 3 lines because the guards moved into `tailed_array.rs`. `docs/excel-semantics/references.md`
"Seen-range cap" notes that a bounded tailed operand can reach the cap one rectangle sooner (DEVIATION,
directive 5).

Eval delta: none (`make eval-check`: 0 regressions, 0 fixes). The speed-up is measured by T-0492.
