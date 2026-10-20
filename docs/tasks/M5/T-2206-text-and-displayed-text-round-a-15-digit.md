---
id: T-2206
title: "TEXT and displayed text round a 15-digit decimal tie half away from zero (TEXT(2.5,\"0\") is \"3\", TEXT(36.5,\"0\") is \"37\")"
milestone: M5
lane: functions-text
status: done
priority: P1
size: S
tier: sonnet
campaign: visible
depends_on: []
---

## Goal
TEXT and the displayed text of a number format round the value at the shown decimal place half away
from zero on its 15-significant-digit decimal, as Excel does: `TEXT(2.5,"0")` is "3", `TEXT(36.5,"0")`
"37", `TEXT(-2.5,"0")` "-3", `TEXT(0.125,"0%")` "13%", `TEXT(12.345,"0%")` "1235%",
`TEXT(1234.5,"#,##0")` "1,235". Today the engine rounds an exact tie to even ("2", "36", "-2", "12%",
"1234%", "1,234") whenever the significant-digit cut lands on the shown place (a tie after a leading
`0.`, as `TEXT(0.125,"0.00")`, is cut later and is not in this list).

## Context
Split from T-1711 (gap triage 2026-10-20). TEXT is an M5 function (catalog `milestone: M5`,
`prior: 2`), so this value failure belongs to M5's pick order, not to T-1711's M10. Ties like x.5,
x.25 and x.125 are common in money and percentage data, so every user of such a format sees it.

Evidence (excel-graph, Excel for the web, already in eval/):
- probes/displayed-text-formats S!A21 `TEXT(1234.5,"#,##0")` "1,235", ours "1,234" (F and TF lines in
  eval/baseline/probes.txt; T-1610).
- gen-m10 (T-1611) TEXT() value cells (F lines in eval/baseline/gen-m10.txt): 039accabcfe1e6c7 T!A12
  `TEXT(2.5,"""(""0"")""")` "(3)"; 0e3d8eba9432a18d T!A18 `TEXT(-2.5,"0;-0;""zero"";""text: ""@")` "-3";
  659cd5776d423ce3 T!A5 `TEXT(2.5,"#,##0 ;(#,##0)")` "3 "; 810fee377a42575b T!A2 `TEXT(36.5,"0")` "37";
  b4f66526f822ff2c T!A24 `TEXT(36.5,"0\%")` "37%".
- gen-m10 displayed text (TF lines in eval/baseline/gen-m10.txt), formats `0`, `#`, `#,##0`, `###,###`,
  `00000`, `#,##0\ \k`, `#,##0 ;(#,##0)`, `#,##0 ;[Red](#,##0)`, `0%`, `0\%`, `%0`, `#%`, `"("0")"`,
  `\"0\"`, `[$¥-411]#,##0` and `0;-0;"zero";"text: "@` over 2.5, -2.5, 36.5, 0.125 and 12.345 (95 lines):
  - `039accabcfe1e6c7`: F!G13, F!L13, F!G18, F!L18
  - `03a284c57e24ff37`: F!A5, F!L5, F!Q5
  - `05c1c646c95a2d60`: F!K10, F!N10, F!O10, F!P11
  - `0e3d8eba9432a18d`: F!G2, F!G7, F!G16, F!R2, F!R7, F!R16
  - `20512106c9574cdd`: F!Q9, F!Q12
  - `4398098060a7fa9d`: F!F2, F!Q2, F!H10, F!R10, F!H16, F!R16
  - `4c6a49fe4a6fa404`: F!L3, F!L16, F!W3, F!W16
  - `5255389143dbcef6`: F!K3, F!F13, F!I13, F!J13, F!K20
  - `5eb95c63c7d6180c`: F!D7, F!H7, F!M7, F!S7, F!D8, F!H8, F!M8, F!S8, F!D19, F!H19, F!M19, F!S19
  - `659cd5776d423ce3`: F!A2, F!D2, F!E2, F!F6, F!F10
  - `71ad5f93362f4ff4`: F!S6, F!W6, F!B6
  - `810fee377a42575b`: F!B17, F!M17
  - `8e073c5b2145567a`: F!A6, F!L6, F!W6, F!A11, F!L11, F!W11, F!C18, F!M18
  - `9a8b7d68507d7ada`: F!B5, F!G5, F!B13, F!G13
  - `b4f66526f822ff2c`: F!I14, F!M14, F!R14, F!X14, F!I18, F!M18, F!R18, F!X18
  - `bfb1a4b09b0c998e`: F!M5, F!W5, F!M7, F!W7
  - `bfbcd16cee266336`: F!B20
  - `e1c29d3fe448682c`: F!B18, F!H18, F!P18, F!R20
  - `eec351e03ccedfe0`: F!U15
  - `fbdab487cb11409e`: F!N3, F!R3, F!W3, F!N8, F!R8, F!W8, F!N20, F!R20, F!W20

Root cause (T-1711 triage, confirmed by reading the code on main 4ff52373):
`crates/rustsheet-engine/src/formatter/format.rs`, the `ParsePart::Number` branch, rounds with
`number_format::to_precision(value, precision + integer digits)`, which goes through Rust's `{:.*e}`
formatting and so rounds an exact binary tie to even (`36.5` at 2 significant digits is "3.6e1").
12.345 is a binary value just below the tie (1234.4999999999998 after the `%` scaling), and Excel
still gives 1235%: Excel rounds the 15-digit decimal, not the binary value. The General formatter
already does exactly this through `decimal_round::round_decimal(value, decimals,
RoundMode::HalfAwayFromZero)` (`formatter/general.rs` `format_general`, number-formats.md "General",
probes/text-general-ties-*), and `formatter/fraction.rs` `round_numerator` reuses it too. Reuse it
for the number branch (round at `p.precision` decimals after the `%`/`,` scaling, within 15
significant digits); write no new rounding helper. `to_precision` has other callers (fraction.rs,
text/common.rs, engineering): leave them alone.

Watch the sign: `round_decimal` returns 0 (never -0) for a value that rounds to zero. The sign rule for
values that round to zero or to one unit (`TEXT(-0.5,"0")` is "-1", ours "1"; `#,` on -0.5 is "") is
T-1129's and the is-negative test in the same branch; do not change which cells show a minus. The 21
gen-m10 TF lines over -0.5 stay T-1129's.

## Acceptance criteria
- [x] The 5 gen-m10 F lines and the 95 gen-m10 TF lines listed above leave eval/baseline/gen-m10.txt,
      and probes/displayed-text-formats S!A21 leaves eval/baseline/probes.txt (its F and TF lines), with
      no new F or TF line in any baseline (`make eval-baseline` refuses regressions).
- [x] A unit test in the formatter's tests pins `TEXT(2.5,"0")` "3", `TEXT(-2.5,"0")` "-3",
      `TEXT(0.125,"0%")` "13%", `TEXT(12.345,"0%")` "1235%" and `TEXT(1234.5,"#,##0")` "1,235" (all
      from the cells above), and fails before the change.
- [x] docs/excel-semantics/number-formats.md: the gen-m10 entry's "round-half (T-1711)" and the
      displayed-text-formats entry name this task and the rule (half away from zero on the 15-digit
      decimal, as General and ROUND).
- [x] `make check` green; reviewers approve.

## Out of scope
- The minus of a value that rounds to zero or to one unit (T-1129).
- Scientific mantissas (`0.00E+00`, T-1730), fractions (T-2030, T-0982), conditions (T-1734), `[$USD]`
  (T-1737), `?.??` and `#%` of 0.0049 (T-1738), negative and out-of-range dates (T-1731).
- DOLLAR and FIXED (their own rounding; T-2149, T-2184).

## Visible-parity stage
Decision 0083: a wrong TEXT or displayed-text result is visible to every user of the format. The
acceptance criteria above are the visible-parity stage.

## Outcome
The `ParsePart::Number` branch of `formatter/format.rs` now rounds at the shown decimal place with
`decimal_round::round_decimal(.., HalfAwayFromZero)` (15-digit decimal, as General and ROUND);
scientific formats keep the significant-digit cut (T-1730). Unit test `number_format_ties_round_half_away_from_zero`.
Eval delta: gen-m10 known failures 67 -> 62 (129 text cells fixed), probes 1632 -> 1631 (2 text), 0 regressions.
