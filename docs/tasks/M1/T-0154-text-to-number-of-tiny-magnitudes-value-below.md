---
id: T-0154
title: "Text-to-number of tiny magnitudes: #VALUE! below ~1E-309, 0 up to the normal range, 0e309"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: [T-0174]
---

## Goal
Number text of tiny magnitude reads the way Excel reads it: #VALUE! for nonzero values below 1E-309, 0 from 1E-309 up to and including the flush range reaches DBL_MIN 2.2250738585072014E-308 (the edge above it is only bracketed by 2.2250738585073E-308), and #VALUE! for the zero mantissas T-0174 recorded (`0e309`, `0e-400`; not `0.0e309`).

## Context
Found by T-0920 (probes/text-number-words, excel-graph 2026-10-05); rule and cells in
docs/excel-semantics/numbers.md "Number text of 1E+308 or more is not a number". Known failures:
probes/text-number-words A9, B5, H6-H9, K1-K5, K7. The exact lower cut-off (1E-309 reads as 0,
1E-310 is #VALUE!) and whether the flush-to-0 threshold is the smallest normal double or 1E-307
were pinned by T-0174 (probes/text-number-tiny: cut-off exactly 1E-309, flush range ends at 2.2250738585072E-308, `0e309`/`0e-400` #VALUE! but `0.0e309` is 0; known failures there: see the probes/text-number-tiny lines in eval/baseline/probes.txt; #VALUE! side: B/C/D of rows 1, 2, 4, 13, 15, 17, 19; flush side: C3, C5:C8, C18, C20 only, `0+A` already gives 0 there but `VALUE(A)` returns the subnormal value; see numbers.md). Code: `parse_excel_number` in
crates/rustsheet-engine/src/number_format.rs (one place; it feeds `cast_number`, VALUE and the lexer).
Numeric fitting of a boundary, hence tier opus.

## Acceptance criteria
- [x] The engine follows the boundaries T-0174 recorded in numbers.md; the T-0174 probe cells pass too.
- [x] The listed probes/text-number-words cells pass; baseline shrinks.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`parse_excel_number` (number_format.rs) now applies Excel's small-magnitude rule, so `cast_number`,
VALUE and the lexer share it: nonzero text below 1E-309 is not a number, 1E-309 up to DBL_MIN
reads as 0, and a zero mantissa is a number only when its exponent less its fraction digits lies
within ±308. The zero rule and literal behavior were pinned by two new probes
(text-number-tiny-zero, literal-number-tiny; excel-graph 2026-10-07): Excel refuses to open a
workbook with `=1E-310` or `=0E309`, so such literals are parse errors here. Eval: probes known
failures 236 -> 204 (35 fixed, 3 new-coverage cells baselined: text-number-tiny-zero B15:D15, the
305-zero text, filed as T-1097). Follow-ups: T-1097 (long zero text), T-1098 (`parse_finite_number`
callers).
