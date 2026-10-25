---
id: T-2705
title: "Scientific mantissa ties: 0E+0 and #E+0 on 2.5 are 3E+0, 0.00E+00 on 12.345 is 1.23E+01 (gen-m10, 11 cells)"
milestone: M5
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
TEXT and the displayed text of a scientific code (`0E+0`, `#E+0`, `0.00E+00`) round the mantissa as Excel
does. An exact tie goes away from zero (2.5 is "3E+0"), and a decimal that only looks like a tie goes by
the double's own value (12.345 is "1.23E+01").

## Context
Gap triage 2026-10-25 (`make eval` on main c30bebf7). No open task owns these cells:
- T-1730 (done) fixed the exponent sign and engineering grouping.
- T-2206 (done) moved the fixed codes to the 15-digit decimal rounded half away from zero, and its Out
  of scope left scientific mantissas alone.
- T-2227 covers zero mantissas and carries only.
- T-2598 covers engineering codes (two or more integer placeholders) only.

Excel evidence (excel-graph, gen-m10, T-1611):
- Displayed text (TF lines in eval/baseline/gen-m10.txt):
  - `0E+0` on 2.5: "3E+0", ours "2E+0" (05c1c646c95a2d60 F!I10, 5255389143dbcef6 F!D13).
  - `#E+0` on 2.5: "3E+0", ours "2E+0" (5eb95c63c7d6180c F!G19, b4f66526f822ff2c F!L18).
  - `#E+0` on -2.5: "-3E+0", ours "-2E+0" (5eb95c63c7d6180c F!G8, fbdab487cb11409e F!Q3).
  - `0.00E+00` on 12.345: "1.23E+01", ours "1.24E+01" (05c1c646c95a2d60 F!L11, 5255389143dbcef6 F!G20,
    659cd5776d423ce3 F!B10, eec351e03ccedfe0 F!Q15).
- TEXT value (F line in eval/baseline/gen-m10.txt): 5eb95c63c7d6180c T!A7 `TEXT(2.5,"#E+0")` "3E+0",
  ours "2E+0".

12.345 is the double 12.3449999999999997513..., so its mantissa 1.2345 is just below the tie, and Excel
rounds it down. Under T-2206 the fixed codes round the 15-digit decimal instead (`0%` on 12.345 is "1235%").
The reading that fits all 11 cells is: the scientific mantissa rounds the double itself, with ties away
from zero (docs/excel-semantics/number-formats.md, "Displayed text over an edge-value pool", the
gap-triage 2026-10-25 bullet).

Code: crates/rustsheet-engine/src/formatter/format.rs, the `p.is_scientific` branch after
`round_shown_place` (the mantissa division by `10^exponent` and `to_precision(value, 15)`), and
`get_fract_part`. Reuse `decimal_round` (T-2206) rather than adding another rounding helper.

## Acceptance criteria
- [x] The 10 TF lines and the F line listed under Context leave eval/baseline/gen-m10.txt
      (`make eval-baseline`), with no new known-failure or TF line in any suite. In particular, every
      other scientific cell of gen-m10, probes/displayed-text-formats, poi/NumberFormatApproxTests and the
      engineering cells of T-2226 still pass.
- [x] A unit test in the formatter covers the four values above (2.5, -2.5 under `0E+0` or `#E+0`, and
      12.345 under `0.00E+00`).
- [x] number-formats.md's gap-triage 2026-10-25 bullet says "Implemented (T-2705)" and states the rule.
- [x] `make check` green; reviewers approve.

If the rule above breaks a passing scientific cell, stop and hand the task up (work-loop step 5b): the
rule then needs a probe, which is not this task's work.

## Out of scope
- Zero mantissas and carries (T-2227); engineering codes (T-2598).

## Visible-parity stage
Decision 0083: a wrong TEXT or displayed-text result is visible to every user of the format. The
acceptance criteria above are the visible-parity stage.

## Outcome
Done. The Context's reading (round the double itself) was wrong. A probe settled it (probes/t2705-scientific-mantissa-
rounding, excel-graph 2026-10-25): the mantissa rounds the 15-digit decimal half away from zero, once, at its shown digits.
`0.00E+00` on 1.005 is "1.01E+00". 12.345 is not below a tie (it is 12.34500000000000063…); ours was wrong only because it
rounded twice. `decimal_round::round_significant` (`significant_15` and then `round_off`) gives a one-integer-placeholder
mantissa in `formatter/format.rs`; engineering codes keep `round_engineering`.
`make eval-baseline`: gen-m10 fixed 13, namely the 10 TF lines, the T!A7 F line, bfbcd16cee266336 F!W12 and
fbdab487cb11409e F!Q20. No regressions, and poi is unchanged at 119. One probe cell fails, A13 (a `%` after the exponent
scales in ours but not in Excel); it is filed as T-2720. The unit tests in formatter/test/test_scientific.rs cover 2.5,
-2.5, 12.345, 1.005, 2.5-2^-51, 0.285 and 100.5.
