---
id: T-0985
title: "T of a multi-cell range: no implicit intersection in legacy formulas (3 poi known failures)"
milestone: M2
lane: functions-text
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`T(range)` over a multi-cell range matches Excel in legacy formulas (Excel-cached evidence: never the
implicitly intersected cell) and in dynamic-array formulas (unprobed).

## Context
Found by the gap triager on 2026-10-06. poi/FormulaEvalTestData_Copy.xlsx (Excel-cached, Excel 15,
legacy formulas in row 1352 of EverythingTests); all three expect "":
- M1352 `T(H12:J12)` (545, 454, 0; column M is outside H:J): actual `#VALUE!` (failed intersection).
- N1352 `T(L7:N7)` (1.001, "alphanum", "blanksIncl"): actual "blanksIncl" (N7, the intersected cell).
- O1352 `T(J1351:J1353)` (J1351 blank, J1352 = "non-numeric", J1353 = "non-numeric"): actual
  "non-numeric" (J1352).
All three top-left cells are non-text, so the evidence fits both "T reads the top-left cell" and "T of
a multi-cell range is always empty". The dynamic form now settles it for dynamic formulas: `T(H1:H2)`
with H1 = "x" (text top-left), from a row outside the range, is one "x" (probes/t-array-lift S!E7,
excel-graph, 2026-10-16, T-1729; a known failure, ours ""), so T reads the top-left cell. Semantics: docs/excel-semantics/
references.md "T of a multi-cell range is not intersected" (UNVERIFIED part).

Code: `static_analysis.rs` gives T `args_signature_scalars(arg_count, 1, 0)`, so the parser inserts
implicit intersection in legacy formulas; `fn_t` in `crates/rustsheet-engine/src/functions/text/common.rs`
has a `FIXME: Implicit intersection` and maps a `Range` to "". N (`fn_n` in
`functions/information/mod.rs`) has the same signature and returns `#N/IMPL!` for ranges; check it
with the same probe.

Needs an oracle probe first (excel-probe skill): legacy and dynamic forms of `T(A1:A3)` with a text
top-left cell, with text only in a later cell, and from a row outside the range; the same for
`N(A1:A3)` with a number top-left; and `T({"a","b"})`, `N({5,6})`. Then implement the observed rule.

## Acceptance criteria
- [ ] A probe case pins the questions above (oracle-filled); the references.md entry is rewritten
      from it with Status VERIFIED.
- [ ] `F	poi/FormulaEvalTestData_Copy.xlsx	EverythingTests!M1352`, `...!N1352` and `...!O1352`
      disappear from eval/baseline/poi.txt (`make eval-baseline`), with no new F lines; the probe's
      cells pass, and so does probes/t-array-lift S!E7.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other functions whose range arguments are marked scalar (WORKDAY holidays: T-0983).
