---
id: T-0491
title: "SPEC-0020 amendment: bounded range leaves and ROW position leaves in the tailed reading"
milestone: M11
lane: perf
status: todo
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
- [ ] `tailed_leaves` takes single-column ranges of any one common span and `ROW(<leaf range>)`; mixed spans,
      `ROW()` without an argument and COLUMN stay refused.
- [ ] Equivalence tests in `test/test_tailed_array.rs` (tailed vs `TAILED_OFF`) for spans starting above, inside
      and below the used area, a ROW tail picked (fallback) and not picked (tailed), and a real `#N/IMPL` in the
      tail (fallback), over SUM, COUNT, MIN and SUMPRODUCT.
- [ ] `rustsheet-eval check` 0 regressions; existing tailed tests unchanged.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SMALL/LARGE (T-0492), UNIQUE (T-0493), LOOKUP (T-0494).
