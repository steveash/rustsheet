---
id: T-0934
title: "TEXT elapsed-time formats: fractional seconds, negatives and plain h/m/s after [h]/[m]/[s] (47 known failures)"
milestone: M10
lane: functions
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
`TEXT(x, fmt)` with elapsed-time tokens (`[h]`, `[hh]`, `[m]`, `[mm]`, `[s]`, `[ss]`) matches
Excel for fractional seconds, negative values and mixed elapsed/plain tokens.

## Context
Evidence: `poi/ElapsedFormatTests.xlsx` (excel-graph sidecar from T-0019; the workbook uses the 1904
date system), formula `=TEXT(Cn, Bn)`. Values: 3.14159, `2/24+3/(24*60)+4/(24*60*60)` (2:03:04),
0, and the negatives of the first two. Formatter: `crates/rustsheet-engine/src/formatter/`
(`lexer.rs` has the `Elapsed*` tokens; `format.rs` renders them).

Root causes:
1. **Fractional seconds after an elapsed token return #VALUE!** (30 cells): rows 2-7, 12-17,
   22-27, 32-37, 42-47. For example A2 `[h]:m:s.000` on 3.14159 expected "75:23:53.376"; A4
   `[s].000` expected "271433.376"; A15 `[hh]:mm:ss.0` on 2:03:04 expected "02:03:04.0"; A27
   `[ss].000` on 0 expected "00.000"; A42 `[h]:m:s.000` on -3.14159 expected "-75:23:53.376".
2. **Negative values** (A40, A41, A50, A51 and the negative rows above): Excel formats the absolute
   value and puts one "-" at the very front, before any literal text. A40
   `"It was "[h]" [yes, "h"] hours and "mm:ss` on -(2:03:04) expected
   "-It was 2 [yes, 2] hours and 03:04", actual "It was -3 [yes, -3] hours and 56:56".
3. **Plain h/s after an elapsed token of the same unit is also elapsed** (A10, A11, A21): A10
   expected "It was 75 [yes, 75] hours and 23:53" (actual "[yes, 3]"); A11
   `[s]" [yes, "ss"] seconds"` expected "271433 [yes, 271433] seconds" (actual "[yes, 53]").
4. **Elapsed token after a plain `s`** (A8, A9, A18, A19, A28, A29, A38, A39, A48, A49) is
   **CONFLICTING**: Excel for the web gives "53:23 @ hour 1293", desktop Excel cached
   "53:23 @ hour 75" (see number-formats.md, decision 0019). These cells are not scored and are
   out of scope until an excel-desktop probe on current Microsoft 365 decides; do not implement
   either variant here.

## Acceptance criteria
- [ ] The 37 lines for root causes 1-3 (A2:A7, A10:A17, A21:A27, A32:A37, A40:A47, A50, A51)
      disappear from eval/baseline/poi.txt with no new F lines.
- [ ] Each rule is recorded in `docs/excel-semantics/number-formats.md` with the cells above.
- [ ] `make check` green; reviewers approve.

## Out of scope
Empty format strings (T-0933). Date/time formats without elapsed tokens
(`poi/DateFormatTests.xlsx`, `poi/DateFormatNumberTests.xlsx`, excel-cached).
