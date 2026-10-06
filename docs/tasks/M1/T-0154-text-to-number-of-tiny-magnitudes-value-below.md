---
id: T-0154
title: "Text-to-number of tiny magnitudes: #VALUE! below ~1E-309, 0 up to the normal range, 0e309"
milestone: M1
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-0174]
---

## Goal
Number text of tiny magnitude reads the way Excel reads it: #VALUE! for nonzero values below 1E-309, 0 from 1E-309 up to and including the smallest normal double 2.2250738585072E-308, and #VALUE! for the zero mantissas T-0174 recorded (`0e309`, `0e-400`; not `0.0e309`).

## Context
Found by T-0920 (probes/text-number-words, excel-graph 2026-10-05); rule and cells in
docs/excel-semantics/numbers.md "Number text of 1E+308 or more is not a number". Known failures:
probes/text-number-words A9, B5, H6-H9, K1-K5, K7. The exact lower cut-off (1E-309 reads as 0,
1E-310 is #VALUE!) and whether the flush-to-0 threshold is the smallest normal double or 1E-307
were pinned by T-0174 (probes/text-number-tiny: cut-off exactly 1E-309, flush range ends at 2.2250738585072E-308, `0e309`/`0e-400` #VALUE! but `0.0e309` is 0; known failures there: see the probes/text-number-tiny lines in eval/baseline/probes.txt; the #VALUE! rows fail in B, C and D, the flush-to-0 rows only in C (VALUE); see numbers.md). Code: `parse_excel_number` in
crates/rustsheet-engine/src/number_format.rs (one place; it feeds `cast_number`, VALUE and the lexer).
Numeric fitting of a boundary, hence tier opus.

## Acceptance criteria
- [ ] The engine follows the boundaries T-0174 recorded in numbers.md; the T-0174 probe cells pass too.
- [ ] The listed probes/text-number-words cells pass; baseline shrinks.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
