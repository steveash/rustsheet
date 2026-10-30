---
id: T-3194
title: "SPEC-0020 amendment: a lifted VLOOKUP, HLOOKUP or MATCH over a whole-column lookup_value reads a used head and one tail row (probes/t3096 4.5 s)"
milestone: M2
lane: perf
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
A VLOOKUP, HLOOKUP or MATCH whose `lookup_value` is a whole column (`VLOOKUP(Keys!$A:$A,tbl,2,FALSE)`), read through
INDEX or an aggregate, costs about the used rows of that column plus one row for the blank rest. Today it looks up and
holds all 1,048,576 elements. Values, errors and error order stay bit-identical.

## Context
Gap triage 2026-10-30 (opus refill; `make eval` on main's parent 33f68044, re-timed on origin/main c59497fc). After the
lunar-calendar probes (T-2908), probes/t3096-lookup-value-blank-elements is the slowest case in the eval: 4,458 ms for
203 cells. Every cell passes. The time goes to its 32 whole-column cells, Calc!B44:E51 and the rows like them:
`INDEX(VLOOKUP(Keys!$A:$A,Data!$A$1:$B$5,2,FALSE),2)`, `INDEX(MATCH(Keys!$A:$A,Data!$A$1:$A$5,0),1000)` and the HLOOKUP
forms. That is about 140 ms a cell, against well under 1 ms for the same lookups over `Keys!$A$1:$A$3`.

Callgrind on origin/main (`valgrind --tool=callgrind ./target/release/rustsheet-eval show
probes/t3096-lookup-value-blank-elements.case.json`), 28.3G instructions in total:
- `Model::index_impl` 28.2G, of which `index_source` → `reference_arg_reading` → `lift_values` 24.1G;
- `lift_grids` 17.3G and `vlookup_impl` 15.3G; `fn_match` 8.8G;
- `evaluate_range` (reading the 1,048,576-row key column) 6.8G;
- malloc and free 18G together. Dropping `Vec<Vec<ArrayNode>>` costs 5.5G, dropping `CalcResult` 4.1G, and
  `lift::driver::same_as` 4.0G (`LiftOptions::REUSE_EQUAL_RUNS` compares every element to the one before it).

So each element of the blank tail is still allocated, compared and dropped, and each `#N/A` builds its own
`"Not found".to_string()` (`lookup_each.rs`). SPEC-0020 (docs/specs/SPEC-0020-whole-column-array-operands.md) already
represents such operands as a used head plus one repeated tail row for aggregates, and Amendment 1 (T-0488) extended it
to UNIQUE, LOOKUP, SMALL and LARGE. Its non-goals name lookups as clipped already (T-0448). That holds for the
`lookup_array`/`table_array` side, not for a lifted `lookup_value`. Code: `Model::lookup_each`
(functions/lookup_and_reference/lookup_each.rs), MATCH's lift, `Model::lift_values` (functions/lift/driver.rs), the
INDEX source path (functions/lookup_and_reference/index.rs), `tailed_array.rs`.

Semantics to keep. They come from the probe's own Excel cells (excel-graph, T-3096; docs/excel-semantics/lookup.md
"VLOOKUP / HLOOKUP lift over an array lookup_value").
- A blank element of a lifted lookup_value is looked up exactly as the scalar blank-cell key, as the number 0. Rows
  44-51 (whole column) equal the scalar control rows 2-8 in all four tables, so one tail element's result stands for
  the whole tail. Depending on the table and mode that is the 0 row, the -1 row or `#N/A`, never `""` or FALSE.
- INDEX past the used head (`...,1000)`) reads a tail element.
- The shared cell-visit cap stays as specified: elements times the searched column's evaluated cells, so
  `VLOOKUP(SEQUENCE(1048576),SEQUENCE(1048576,2),2,FALSE)` stays `#ERROR!` (DEVIATION, decision 0014, lookup.md). The
  amendment must say whether a tailed whole column is still charged for 1,048,576 elements. Keeping that charge is the
  safe default: a cap result must not change.
- A spill of the whole lifted result still materializes, as SPEC-0020's non-goals say.

## Acceptance criteria
- [ ] SPEC-0020 gains an amendment (its own section, like Amendment 1) for a lifted lookup_value of VLOOKUP, HLOOKUP and
      MATCH (and XMATCH/XLOOKUP if they share the path). It covers which call shapes are tailed, which consumers read the
      tailed result (at least INDEX with a scalar row and the aggregates already tailed), the invariants (same value,
      error and first error; same cells read and recorded) and the limits. Reviewers approve it.
- [ ] probes/t3096-lookup-value-blank-elements takes at most 500 ms in `rustsheet-eval run --suite probes --filter t3096`
      on a release build (from 4,458 ms), and the Outcome gives callgrind's total for the case before and after.
- [ ] A bench in crates/rustsheet/benches/lookup.rs covers `INDEX(VLOOKUP(A:A,tbl,2,FALSE),k)` and
      `SUM(--ISNA(MATCH(A:A,col,0)))` over a 1,000-row used column.
- [ ] Unit tests compare the tailed and materialized paths: a blank key matching a 0, `""` or FALSE row, `#N/A`, an
      error element in the head, approximate and exact modes, HLOOKUP over a whole row.
- [ ] `make eval` shows no new known failure in any suite; `make check` green; reviewers approve.

## Out of scope
- The lunar-calendar probe times (T-2908).
- A spill of a whole-column lifted lookup; whole-column `lookup_array`/`table_array` arguments, which already clip.
