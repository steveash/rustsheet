---
id: T-0278
title: "Legacy *IF(S) formulas intersect a range criterion (all but COUNTIF and AVERAGEIF return #VALUE!)"
milestone: M2
lane: functions-criteria
status: done
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
In a legacy (non-dynamic) formula every *IF(S) function intersects a multi-cell range criterion like COUNTIF and AVERAGEIF already do, instead of returning `#VALUE!`.

## Context
T-0263 fixture `fixtures/ifs-legacy-range-criteria.xlsx` (oracle excel-graph): with `A1:A6`/`D1:D2` criteria, formulas in rows 1 and 2 give the criterion `D1` / `D2`: COUNTIF, COUNTIFS, SUMIF (with and without a one-cell sum range), SUMIFS, AVERAGEIF, AVERAGEIFS, MAXIFS and MINIFS all intersect (3,3,7,7,7,2.33,2.33,3,1 in row 1). We already intersect COUNTIF (F) and AVERAGEIF (K); COUNTIFS (G), both SUMIFs (H, I), SUMIFS (J), AVERAGEIFS (L), MAXIFS (M) and MINIFS (N) return `#VALUE!` (see `make eval-show CASE=ifs-legacy-range-criteria`). COUNTIFS/SUMIFS/AVERAGEIFS/MAXIFS/MINIFS keep `Signature::Vector` criteria in `static_analysis.rs` (T-0207).

## Nightly fuzz crash (2026-10-08)
Nightly fuzz run 37724893331 (`main` b38d244), target `xlsx_import`, artifact
`crash-f924072410351cea683d891db23e1cca3a7e90a8`: the input is the committed seed
`eval/suites/fixtures/cases/ifs-legacy-range-criteria.xlsx` itself (5,500 bytes, the corpus seed
built by `fuzz/seed_corpus.py`; the fuzzer crashed on it unmutated). Loading it with `recalc: true`
through `Workbook::from_xlsx_bytes` panics in a debug-assertions build (cargo fuzz builds with them on):

    panicked at crates/rustsheet-engine/src/model.rs:984:25:
    Larger-than-1x1 array reached scalar-context cell (sheet=0, row=1, column=7, 1x2);
    implicit intersection was expected to collapse it.

Cell G1 is the legacy formula `COUNTIFS(A1:A6,D1:D2)`: the un-intersected `Signature::Vector`
criterion described above yields a 1x2 array in a scalar cell. Reproduced on `main` bccce59. Same
root cause as this task, so the steward raised it to P0 instead of filing a duplicate (decision 0035).

## Acceptance criteria
- [x] A unit test loads `fixtures/ifs-legacy-range-criteria.xlsx` through the facade with `recalc: true` in a debug build without panicking (the reproducer, decision 0035).
- [x] All cells of `fixtures/ifs-legacy-range-criteria` pass; baseline shrinks.
- [x] Dynamic-array behavior (lifting, probes/ifs-array-criteria) is unchanged.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing dynamic-array lifting of criteria.

## Outcome
The criterion arguments of COUNTIFS, SUMIF, SUMIFS, AVERAGEIFS, MAXIFS and MINIFS now have `Scalar`
signatures (`args_signature_ifs`, `args_signature_sumif` in `static_analysis.rs`), like COUNTIF and
AVERAGEIF, so a legacy formula gets the automatic `@` on import and intersects a multi-cell range
criterion; dynamic formulas still lift (probes/ifs-array-criteria unchanged). SUMIF's private
`args_signature_sumif_spill` is gone. The fuzz reproducer is `crates/rustsheet/tests/legacy_ifs_range_criteria.rs`
(loads the fixture through the facade with `recalc: true`; it panicked in debug before).
The `sumif_unique` round-trip test now expects the automatic `@` on the criterion internally; the export is unchanged.
Review follow-up: Excel evaluates SUMPRODUCT's arguments as arrays in a legacy formula, so a range
criterion nested in it lifts (probes/ifs-legacy-sumproduct, excel-graph): `add_implicit_intersection`
adds no `@` inside SUMPRODUCT and `remove_redundant_implicit_intersection` keeps every `@` there
(`is_array_context`). criteria.md records that and the failed-intersection quirk (the criterion becomes
`#VALUE!`, poi E1324). The probe's 17 off-row SUM/standalone cells are new-coverage known failures: the
spec adapter evaluates `legacy_formulas` as dynamic formulas (as for multiarea-at-legacy).
Eval: fixtures known failures 14 -> 0, poi 572 -> 571, probes +1 case (16 passing, 17 adapter-only
known failures), 0 regressions.
