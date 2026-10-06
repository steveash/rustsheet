---
id: T-0980
title: "TEXT fixed-decimal formats round the value to 15 significant digits before padding (16 probes-numeric failures)"
milestone: M10
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`TEXT(x,"0.000000000000000000")` (more decimals than 15 significant digits fill) prints x rounded to 15
significant digits and pads the rest with zeros, like Excel, instead of cutting the digits at a fixed
decimal position.

## Context
Rule: docs/excel-semantics/numbers.md, "Number → text" (`TEXT(x,"0.000…")` pads after 15 significant
digits; the 15th digit is rounded, not cut). Evidence: `probes-numeric/4f413cc257ce15c4` (excel-graph),
16 failing cells, all TextMismatch, format `"0.000000000000000000"`:

| cell | value | expected | actual |
|---|---|---|---|
| A7 | `0.1+0.2-0.3` | "0.000000000000000056" | "0.000000000000000000" |
| A15 | `(43.1-43.2)+1` | "0.899999999999999000" | "0.899999999999998000" |
| A23 | `2/3` | "0.666666666666667000" | "0.666666666666666000" |
| A35 | `1.333+1.225-1.333-1.225` | "-0.000000000000000222" | "-0.000000000000000000" |
| A51 | `0.000123456789012345` | "0.000123456789012345" | "0.000123456789012000" |
| A59 | `1e15+0.3` | "1000000000000000.000000000000000000" | "1000000000000000.200000000000000000" |
| A71 | `2^53+1` | "9007199254740990.000000000000000000" | "9007199254740992.000000000000000000" |
| A79 | `1.005*1000` | "1005.000000000000000000" | "1004.999999999990000000" |
| A83 | `0.285*100` | "28.500000000000000000" | "28.499999999999900000" |
| A87 | `4.35*100` | "435.000000000000000000" | "434.999999999999000000" |
| A91 | `1-0.9` | "0.100000000000000000" | "0.099999999999999000" |
| A95 | `0.3-0.1` | "0.200000000000000000" | "0.199999999999999000" |
| A107 | `1/7` | "0.142857142857143000" | "0.142857142857142000" |
| A111 | `2^0.5` | "1.414213562373100000" | "1.414213562373090000" |
| A139 | `0.7+0.1` | "0.800000000000000000" | "0.799999999999999000" |
| A143 | `8.2-0.2` | "8.000000000000000000" | "7.999999999999990000" |

Every expected value is the value rounded to 15 significant digits (as `number_to_text` in
`crates/rustsheet-engine/src/number_format.rs` does, T-0107) then zero-padded. Tiny values keep their
significant digits (A7, A35: no snapping inside a TEXT argument), large ones drop digits past the
15th (A59, A71).

Code: `crates/rustsheet-engine/src/formatter/format.rs`, the `ParsePart::Number` branch and
`get_fract_part`: the fraction is formatted at `precision` decimals and then cut at
`15 - int_len` positions, which truncates and counts positions from the decimal point instead of
significant digits. Reuse the existing 15-digit helper (`to_precision` / the routine behind
`number_to_text`); do not add a second one.

probes-numeric is not gated (`gate = false`, no baseline file), so progress is read from the report.

## Acceptance criteria
- [ ] In `make eval`, the 16 cells above pass and probes-numeric/4f413cc257ce15c4 has no failing cell
      left (it has exactly these 16 today).
- [ ] No new F lines in any gated baseline (watch poi NumberFormatTests/NumberFormatApproxTests/
      GeneralFormatTests and ironcalc-calc TEXT cells: formats with few decimals must not change).
- [ ] Formatter unit tests for A23, A51, A59 and A7 above.
- [ ] numbers.md "Number → text" cites these cells and gets an `Implemented in:` line for TEXT.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Near-zero snapping of `+`/`-` results (T-0110).
- General format (T-0938) and scientific formats (T-0937).
