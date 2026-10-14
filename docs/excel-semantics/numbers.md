# Numbers and floating point

Background: Excel stores IEEE-754 doubles. It displays at most 15 significant digits and applies
some extra rounding rules. Microsoft documents a few of them
(https://learn.microsoft.com/en-us/troubleshoot/microsoft-365-apps/excel/floating-point-arithmetic-inaccurate-result),
but the exact rules must be measured.

Every rule below that cites oracle-sidecar values was re-checked with exact float parsing on
2026-10-05 and still holds (T-0026, docs/research/numeric-reaudit-2026-10-05.md).

## Near-zero snapping in addition and subtraction
Rule: when the formula's **outermost** operation is `+`/`-` (a SUM call snaps wherever it sits, see
"SUM forms" below), and the
result is within a few ULP of zero relative to the operands, Excel returns exactly 0.
`=0.1+0.2-0.3`, `=(0.1+0.2)-0.3`, `=SUM(0.1,0.2,-0.3)` and `=1-0.9-0.1` all give 0. Snapping does
**not** happen when the whole formula is parenthesized (`=(X-b)`) or another operation follows
(`=X-b+0`, `=(X-b)*1`). For X = b·(1±k·2^-52): k ≤ 3 always snaps, k = 4–7 snaps for some bases only
(15, 9, 9, 9 of 24), k = 8 never; identical for `X-b`, `X+(-b)` and `SUM(X,-b)`. The exact
threshold (T-0110, fitted on all 1,296 cells): snap iff the two addends of the final addition
have opposite signs and |result| < 8 ULP of the **smaller** addend in magnitude (= 2^(e-49), e
its binary exponent). The opposite-sign condition is implied, not observed: every probe cell
subtracts, and with equal signs |result| is at least the smaller addend. Snapped results are at most 7 ULP and kept ones at least 8. The larger
addend or a relative bound (|r|/max < 2^-k) does not separate them: for b = 1 the `+` side snaps
up to k = 7 but the `-` side, whose smaller addend 1-k·2^-52 has half the ULP, only up to k = 3.
The earlier hypothesis (no snapping with parentheses or in SUM) was wrong for SUM; it holds for `+`/`-`.
Evidence: probes/precision-and-rounding.case.json A2–A4, A15; probes-numeric/30bc5f8fab02d0a2
`snap-addsub/*` (1,296 cells). Oracle: excel-graph, 2026-10-04.
Status: VERIFIED
Implemented in: `snap_near_zero` (arithmetic.rs), applied at the cell formula's root by `Model::evaluate_formula_root` (an outermost `+`/`-`) and `fn_sum` (every SUM call, functions/math_and_trigonometry/sum.rs); a wholly parenthesized formula is a root `Node::Group` (`Parser::parse_parenthesized`, T-1017, T-1341, T-1392) and is not snapped. Array operands of `+`/`-` are not snapped (unprobed); SUM over array arguments does snap (below).
Groups and names (T-1137, probes/near-zero-snap-groups, excel-graph 2026-10-08): `=((0.1+0.2-0.3))`, `=(((0.1+0.2-0.3)))` and `=((1-0.9-0.1))` are not snapped (5.551115123125783e-17, 5.551115123125783e-17, -2.7755575615628914e-17), like the single-level `=(0.1+0.2-0.3)`; a group that is an operand of an outer `+` (`=(0.1+0.2-0.3)+0`, `=0+(0.1+0.2-0.3)`) is not snapped either. A defined-name body **is** snapped at its own root: `nz` = `=0.1+0.2-0.3` and `nzb` = `=Sheet1!$A$1-0.3` (A1 = `=0.1+0.2`) give 0 for `=nz`, `=nzb`, `=nz+0`, `=nz*1` and `=SUM(nz)`; the engine snaps a name body at its root through `evaluate_formula_root` (T-1160). Name bodies written as a group or a SUM (T-1164, probes/near-zero-snap-name-bodies, excel-graph 2026-10-08): `nzg` = `=(0.1+0.2-0.3)` and `nzgg` = `=((0.1+0.2-0.3))` are not snapped (`=nzg`, `=nzgg`, `=nzg+0` and `=SUM(nzg)` give 5.551115123125783e-17, like the cell forms), while `nzs` = `=SUM(0.1,0.2,-0.3)` is snapped (`=nzs` and `=nzs+0` give 0). The engine keeps a grouped root on a name body (`Model::parse_internal_formula`, `user_formula_to_internal`, the xlsx import re-parse, sheet copy and rename, name display); a parenthesized reference body stays a reference (`rg` = `=(Sheet1!$B$1:$B$2)`, `ig` = `=(INDEX(...))` and `og` = `=(OFFSET(...))` give ISREF TRUE, ROWS/SUM as without the parentheses: probes/defined-name-grouped-reference, excel-graph 2026-10-08), so `is_reference` looks through the group (`Node::ungrouped`). The same parse also keeps the grouped root of a conditional-formatting formula written wholly in parentheses (`user_formula_to_internal` serves both); whether Excel snaps inside CF formulas is UNVERIFIED and not observable through the oracle (T-1171: excel-graph reads cell values only, the case format and the Graph workbook API carry no conditional-formatting rules or their rendered fill; see docs/architecture/oracle.md, Unobservable features), so it needs desktop Excel (M14, Phase 4, decision 0039, T-1204). Unprobed: a formula typed as `=(X-b` that Excel completes with `)`. FORMULATEXT of `=((0.1+0.2-0.3))` is `=((0.1+0.2-0.3))`: the engine keeps every written level around a whole formula (one root `Group` holding every level, T-1161, T-1341).

SUM forms (T-1131, probes/near-zero-sum-forms, excel-graph 2026-10-11): a SUM call snaps its final addition **wherever it sits**, unlike `+`/`-`: `=(SUM(0.1,0.2,-0.3))`, `=SUM(A1:A3)+0` and `=SUM(A1:A3)*1` (A1:A3 = 0.1, 0.2, -0.3) all give 0, as do `=SUM({0.1,0.2,-0.3})`, `=SUM({0.1,0.2},-0.3)`, `=SUM(0.1,{0.2,-0.3})`, `=SUM(A1:A2,A3)` and a range with a blank or text between the addends. A trailing zero addend stops the snap: `=SUM(0.1,0.2,-0.3,0)` and `=SUM(A1:A3,0)` give 5.551115123125783e-17 (the last two addends are -0.3 and 0). 2-D ranges add in row-major order, so `=SUM(D1:E2)` (0.1, 0.2 / -0.3, 0) is 5.551115123125783e-17 (last addend 0) and `=SUM(J1:K2)` (0.1, -0.3 / 0.2, -0.1) is -0.09999999999999998. The engine snaps in `Model::fn_sum` for every SUM call. AGGREGATE(9, ...) and SUBTOTAL(9/109, ...) snap the same way (T-1336, probes/near-zero-aggregate-forms B1-B6, B10-B12, excel-graph 2026-10-11: `=AGGREGATE(9,4,A1:A3)`, `=AGGREGATE(9,6,A1:A3)`, `=SUBTOTAL(9,A1:A3)`, `=SUBTOTAL(109,A1:A3)` and `+0` of each give 0 over 0.1, 0.2, -0.3; `=AGGREGATE(9,4,{0.1,0.2,-0.3})` is #VALUE!); the engine snaps in `subtotal_sum` and the AGGREGATE dispatch. PERCENTOF snaps both of its sums the same way (probes/percentof-snap-overflow-union, excel-graph, 2026-10-13, T-0416; it was `#NAME?` only because the case builder stored it without `_xlfn.`): `=PERCENTOF(A1:A3,C1)` and `=PERCENTOF({0.1,0.2,-0.3},1)` are 0, and `=PERCENTOF(A1:A2,A1:A3)` and `=PERCENTOF(A1:A3,A1:A3)` are #DIV/0! (the total snaps to 0). The engine sums both arguments through `fn_sum`. 
Tasks: T-0110 (implementation), T-1131 (SUM forms), T-1336 (AGGREGATE/SUBTOTAL), T-1017 (grouped root), T-1137 (probe), T-1160 (name bodies), T-1164 (grouped and SUM name bodies)

## ROUND family rounds in two decimal stages
Rule: ROUND, ROUNDUP and ROUNDDOWN never scale by a binary power of ten. They (1) cut the exact
binary value to 15 significant digits, then (2) round that decimal at `num_digits` (ROUND: half
away from zero; ROUNDUP: away from zero; ROUNDDOWN: toward zero) and return the nearest double.
Stage 1 rounds to nearest, but an **exact tie at the 16th digit** goes toward zero for ROUND and
away from zero for ROUNDUP/ROUNDDOWN (inconsistent; Excel is right):
`ROUND(999999999999957.5,0)` = 999999999999957 and `ROUND(999999999999956.5,0)` = …956, yet
`ROUNDUP(…957.5,0)` = `ROUNDDOWN(…957.5,0)` = 999999999999958; `ROUND(1E+15+5,0)` = 1E+15 but
`ROUNDDOWN(1E+15+5,0)` = 1000000000000010; `ROUND(12345678901234.75,2)` = 12345678901234.7.
Consequences: `ROUND(1.005,2)` = 1.01 and `ROUND(2.675,2)` = 2.68 (the binary values are just below
the half, but 15 digits read 1.00500000000000); `ROUNDDOWN(1-2^-53,0)` = 1; `ROUNDUP(1E+15+1,0)` =
1E+15; `ROUND(1E+15+86,2)` = 1000000000000090; `ROUNDDOWN(123456789.123,-5)` = 123400000 exactly;
a tie at a coarser digit is half away (`ROUND(12345678901234.5,0)` = 12345678901235, `ROUND(-2.5,0)`
= -3). Large `num_digits` returns the 15-digit value (`ROUND(1.5,400)` = 1.5), very negative ones
give 0 for ROUND (`ROUND(1.5,-400)`) and `#NUM!` for ROUNDUP (`ROUNDUP(1.5,-400)`, the result
overflows). Near the largest double the 15-digit cut itself overflows: `ROUND(2^1023*(2-2^-52),0)`
and `ROUNDDOWN(…,0)` are `#NUM!`. A result of zero has no sign (`ROUNDDOWN(-0.5,0)&""` = "0").
`ROUND(1.23456789012345E+300,-299)` = 1.2E+300.
Evidence: probes/round-family A1–A53 (except A20); probes/precision-and-rounding A6 (1.01),
A7 (2.68), A8 (`ROUND(-2.5,0)` = -3). Oracle: excel-graph, 2026-10-04 and 2026-10-05 (T-0106).
Status: VERIFIED. Implementation: `crates/rustsheet-engine/src/decimal_round.rs` (T-0106).
ROUNDUP's stage 2 differs in its last ulp: see "ROUNDUP adds the unit in binary" below.

## TRUNC is ROUNDDOWN
Rule: `TRUNC(number, num_digits)` behaves like `ROUNDDOWN` on every probed cell, with `num_digits` optional (0).
It cuts the binary value to 15 significant digits first, so binary noise below the 15th digit never
drops a unit: `TRUNC(-(0.1+0.7),1)` = -0.8, `TRUNC(1-2^-53,0)` = 1, `TRUNC(0.29,2)` = 0.29 and
`TRUNC(123456789.123,-5)` = 123400000 exactly. A 16th-digit tie goes away from zero:
`TRUNC(999999999999957.5,0)` = 999999999999958 and `TRUNC(1E+15+9,0)` = 1000000000000010. A huge
`num_digits` wraps like ROUNDDOWN's, as the magnitude modulo 65,536 with the sign kept:
`TRUNC(123.456,65537)` = 123.4, `TRUNC(123.456,-65537)` = 120, `TRUNC(5,2147483647)` = 5,
`TRUNC(123.456,-16)` = `TRUNC(123.456,-400)` = 0. The cut overflows at the largest double, so
`TRUNC(2^1023*(2-2^-52),0)` is `#NUM!`, while
`TRUNC(…,-300)` = 1.79769313E+308 stays finite. A zero result has no sign (`TRUNC(-0.5)&""` = "0").
The upstream IronCalc examples from b93d4224 also hold:
- `ROUND`, `ROUNDUP` and `ROUNDDOWN` of `1E+307*10` with 10 digits keep 1E+308.
- `ROUND(123.456,-400)` = `ROUNDDOWN(…)` = 0 and `ROUNDUP(…)` = `#NUM!`.
- `ROUNDUP(123.456,-16)` = 1E+16.
- `ROUNDUP(-(0.1+0.7),1)` = -0.8, because the 15-digit cut leaves nothing to round.
Evidence: probes/trunc-decimal-stages A1–A56 (excel-graph, 2026-10-11, T-0297).
Status: VERIFIED
Implemented in: `fn_trunc` in functions/math_and_trigonometry/mathematical.rs, via `round_decimal` (T-0297)

## ROUNDUP adds the unit in binary
Rule: when ROUNDUP has to round up, it does not return the double nearest the rounded-up decimal.
It takes the rounded-down decimal (stages 1 and 2 above, as ROUNDDOWN), converts it to the nearest
double, and adds the unit `10^-num_digits` as a double. So the result can be one ulp off the
decimal one, in either direction: `ROUNDUP(123.456,2)` = 123.45 + 0.01 = 123.46000000000001,
`ROUNDUP(0.29,1)` = 0.30000000000000004, `ROUNDUP(2.675,2)` = 2.6799999999999997,
`ROUNDUP(8.255,1)` = 8.299999999999999, `ROUNDUP(7.123,1)` = 7.199999999999999. When nothing is
dropped there is no add (`ROUNDUP(0.29,2)` = 0.29). Negative values round the magnitude and keep the
sign (`ROUNDUP(-123.456,2)` = -123.46000000000001). ROUND and ROUNDDOWN return the decimal-correct
double. A wrapped `num_digits` behaves the same as its wrapped value: `ROUNDUP(x,65536+k)`,
`ROUNDUP(x,2^31+k)` and `ROUNDUP(x,k)` agree on every probed cell.
Evidence: probes/roundup-wrapped-scaling (480 cells: 20 values × k = 1..4, plain, 65536+k, 2^31+k
and -(65536+k), ROUNDUP and ROUNDDOWN); probes/round-huge-digits-wrap A16, A35, A38 (excel-graph,
2026-10-09, T-0268); ironcalc-calc MATH_AND_TRIGONOMETRY/ROUND.xlsx D6 (desktop Excel cached value).
Rejected: multiplying the rounded-up digits by `10^-k` (52/80 cells) and binary scaling
`ceil(x*10^k)` (37/80); the add model matches 80/80.
Status: VERIFIED
Implemented in: `round_decimal` in decimal_round.rs (T-0268)
Tasks: T-0268

## ROUND family: huge num_digits wrap
Rule: a `num_digits` of huge magnitude is narrowed to an integer, and ROUND narrows it differently
from ROUNDUP and ROUNDDOWN (after truncation toward zero; Excel is right, however odd):
- ROUNDUP and ROUNDDOWN keep the sign and take the magnitude, capped at 2^32-1, modulo 65,536.
  `ROUNDUP(123.456,65536)` = 124 (digits 0), `ROUNDDOWN(123.456,65537)` = 123.4 (digits 1),
  `ROUNDUP(123.456,-65537)` = 130, `ROUNDDOWN(123.456,-65535)` = 0 and `ROUNDUP(123.456,-65535)` =
  `#NUM!`, `ROUNDDOWN(0.5,2^31)` = 0 and `ROUNDUP(0.5,2^31)` = 1, `ROUNDUP(42,-2147483649)` = 50.
  2^32 and anything larger act as 65,535 (`ROUNDUP(123.456,2^32+2)` = 123.456), and -2^32 and below
  as -65,535 (`ROUNDDOWN(123.456,-1E+300)` = 0, `ROUNDUP(…)` = `#NUM!`).
- ROUND saturates `num_digits` to a 32-bit integer k (1E+15 acts as 2147483647), then counts the
  digits it keeps, k + e + 1 (e the decimal exponent of the value), in 32 bits with wraparound.
  Past the top the count wraps negative and the result is 0. Past the bottom it wraps positive and
  the value is returned unrounded. So `ROUND(5,2147483647)` = 0 but `ROUND(5,2147483646)` = 5,
  `ROUND(50,2147483646)` = 0, `ROUND(5E+14,2147483633)` = 0 but `…,2147483632)` = 5E+14,
  `ROUND(1E+15,1E+15)` = 0, `ROUND(0.5,1E+15)` = 0.5, `ROUND(42,123456789.123)` = 42,
  `ROUND(123.456,65536)` = 123.456; `ROUND(5E-10,-2147483640)` = 5E-10 but `…,-2147483639)` = 0,
  `ROUND(5E-300,-2147483350)` = 5E-300 but `…,-2147483349)` = 0.
- `ROUNDUP(123.456,4294901762)` (digits 2) = 123.46000000000001, one ulp above the decimal result.
  That comes from ROUNDUP's binary add, not from the wrap: `ROUNDUP(123.456,2)` gives the same
  (see "ROUNDUP adds the unit in binary", T-0268).
- e is taken from the value after its 15-digit cut: `ROUND(10-1E-15,2147483646)` = 0 and
  `…,2147483645)` = 10 (the cut carries 9.99999999999999… to 10, e = 1), `ROUND(1-1E-16,2147483647)` = 0.
Evidence: probes/round-huge-digits (216 cells), probes/round-huge-digits-wrap (A:C by d, E:O ROUND by
the exponent), probes/round-huge-digits-negative (91 cells), probes/round-huge-digits-carry (6 cells); oracle: excel-graph, 2026-10-07 (T-0218).
gen-m2/caa981b0753f455c Calc!A270 (`ROUND(1E+15,1E+15)` = 0, so the quotient is `#DIV/0!`) and
gen-m2/dd99be1f2454159f Calc!A159.
Status: VERIFIED
Implemented in: `effective_digits` in crates/rustsheet-engine/src/decimal_round.rs (T-0218).
Tasks: T-0218, T-0268

## MOD is #NUM! for a huge quotient
Rule: `MOD(n,d)` is `#NUM!` when `|n/d| >= 1.1259E+12` (exactly that decimal, not a power of two):
`MOD(1125899999999,1)` = 0, `MOD(1125900000000,1)` = `#NUM!`. It depends only on the quotient, not on
the magnitude of `n` or `d` and not on signs: `MOD(-1125900000000,-1)`, `MOD(1125900000000,-1)` and
`MOD(2251800000000,2)` are `#NUM!` while `MOD(2251799999999,2)` = 1, `MOD(1E+15+7,1000000)` = 7 and
`MOD(1,1E-12)` (quotient 1E+12) is 2.01E-17 but `MOD(1.2,1E-12)` is `#NUM!`; a fractional quotient
just below the limit works (`MOD(562949999999.5,0.5)` = 0, `MOD(562950000000.5,0.5)` = `#NUM!`).
The older documented limit 2^27 does not apply (`MOD(2^40,1)` = 0).
Evidence: probes/mod-huge-quotient, mod-quotient-threshold (1-5), mod-quotient-boundary (bisected with
excel-graph, 2026-10-05); probes/round-family A20 (T-0106).
Status: VERIFIED
Implemented in: `fn_mod` in crates/rustsheet-engine/src/functions/math_and_trigonometry/mathematical.rs
Tasks: T-0141

## Formula number literals are read at 15 significant digits
Rule: a numeric literal in a formula is cut (truncated, see "15-digit literal parsing") to 15 significant digits when parsed:
`1.7763568394002505E-12-1.77635683940025E-12` = 0, `1.00000000000000049-1` = 0 and
`1.0000000000000009-1` = 0 (a full double parse would leave 4.9E-16 and 8.9E-16), and
`MOD(2,1.7763568394002505E-12)` equals the 15-digit-literal result 1.1089014151366425E-12.
Evidence: probes/literal-digits C1-C6 (found by T-0141); excel-graph, 2026-10-05.
Status: VERIFIED
Implemented in: see "15-digit literal parsing"

## Number → text conversion uses 15 significant digits
Rule: `=CONCAT(0.1+0.2)` → "0.3": numbers become text with 15 significant digits (124 of 136
`CONCAT` probes equal C's `%.15g`; `(a/b*b)&""` round-trips to the integer in 100 of 100). The
12 others: `1e307*100` → `#NUM!`; subnormals print `0` (`1e-307/10`, `2^-1074`, see below); the
exponent is uppercase (`-2.22044604925031E-16`, `5.55111512312578E-17`, `1E+21`); and Excel stays
in fixed notation where `%.15g` would switch: `123456789012345678` → "123456789012345000",
`1e15+0.3` → "1000000000000000", `9999999999999999` → "9999999999999990" and `2^53+1` →
"9007199254740990" (digits past 15 are zeros), `1e-5` → "0.00001", `1e-15` → "0.000000000000001"
(`LEN` 17). Large values switch to E notation too: `1E+20&0.1` → "1E+200.1" and `1E+20&100` →
"1E+20100"; the exact switch is the 20-character rule below.
Under `&`: `0.1+0.2&100` → "0.3100", `1/3&7` → "0.3333333333333337", `=7+Data!$E5&10` with E5 = 1E-15 → "710". Snapping does not apply inside the CONCAT argument:
`CONCAT(0.1+0.2-0.3)` → "5.55111512312578E-17". `TEXT(x,"0.000…")` also pads after 15
significant digits: `TEXT(1/3,"0.000000000000000000")` = `0.333333333333333000`. The 15th digit is rounded, not cut,
and the count is of significant digits, not decimal places: `TEXT(2/3,…)` = `0.666666666666667000`,
`TEXT(0.000123456789012345,…)` = `0.000123456789012345`, `TEXT(1.005*1000,…)` = `1005.000000000000000000`,
`TEXT(2^53+1,…)` = `9007199254740990.000000000000000000`, `TEXT(0.1+0.2-0.3,…)` = `0.000000000000000056`
(probes-numeric/4f413cc257ce15c4 A23, A51, A79, A71, A7; T-0980).
Notation rule (T-0107): plain notation whenever the plain string, without the sign, has at most
20 characters, otherwise `d.dddE±XX` with at least two exponent digits: `99999999999999900000`
and `-10000000000000000000` stay plain, `1E+20`/`-1E+20`/`1E+21` do not; `0.000000000000000001`
(1E-18), `0.000000000000000123` and `0.00012345678901234` stay plain, `1E-19`, `1.23E-17`,
`9.99999999999999E-16` and `1.23456789012345E-05` (21+ characters plain) do not. Trailing zeros of
the 15 digits are dropped (`2^0.5` → "1.4142135623731"). An exact tie at the 16th digit goes toward
zero, as in ROUND's 15-digit cut: `1E+15+15` → "1000000000000010", `1E+14+1.5` → "100000000000001",
`12345678901234.75` → "12345678901234.7".
Exponent width (T-0186, VERIFIED): in E notation the mantissa has 15 significant digits when the
decimal exponent magnitude is at most 98 (`1.23456789012345E+98`, `E-98`, `E+51`, `E-5` all print 15
digits), and 14 when it is 99 or more (`1.23456789012345E+99` → "1.2345678901235E+99",
`E+100`, `E+307`, `E-99`, `E-300` likewise; `9.87654321098765E-99` → "9.8765432109877E-99"). Note
`E+99` has a 2-digit exponent yet already loses a digit, so this is not the 20-character cap alone:
the cut is decided by `|exp| >= 99`. `&""` and `CONCAT` agree. `TEXT(x,"General")` shows the same
cut one digit lower: 6 digits for `|exp| <= 98` ("1.23457E-05", "1.23457E-50") and 5 for
`|exp| >= 99` ("1.2346E+100", "1.2346E-99", though "1.23457E-99" would fit its 11-character
limit; probe D8-D10). `(1/(2^-1022))&""` is "4.4942328371558E+307". Evidence:
probes/number-text-exponent-digits (excel-graph, 2026-10-06).
The General cut (5 digits at `|exp| >= 99`) is VERIFIED at the 98/99 boundary (T-1001): `TEXT(1.23456789012345E+98,"General")`
is "1.23457E+98", `E+99` "1.2346E+99", `E-98` "1.23457E-98", `9.87654321098765E-99` "9.8765E-99". A mantissa that rounds
up to 10 carries into the exponent: `9.9999996E+98` → "1E+99", `E+99` → "1E+100", `E-98` → "1E-97", `E-99` → "1E-98",
`E-10` → "1E-09" (never "10E+98"); `9.9999996E+10` stays fixed "99999996000". Evidence:
probes/text-general-exponent-boundary (excel-graph, 2026-10-07). It is implemented in the formatter's General branch (T-0986, carry T-1001). D8
(`1.23456789012345E-5` → "1.23457E-05", E notation although the exponent is small) follows the 11-character
fixed-versus-E rule (number-formats.md, T-0938) and passes.
Evidence: probes/number-to-text-notation A1–A52 (excel-graph, 2026-10-05, T-0107);
probes/precision-and-rounding.case.json A9, A16; probes-numeric/4f413cc257ce15c4
`number-to-text/*`. Oracle: excel-graph, 2026-10-04. gen-m1/f91603cd8616521f Calc!A50, A186, A27,
A21, A188 (oracle: excel-graph, 2026-10-05, T-0109).
Status: VERIFIED. Implementation: `number_to_text` in `crates/rustsheet-engine/src/number_format.rs` (T-0107; the
`|exp| >= 99` cut: T-0199).

## Negative zero prints as 0
Rule: `-x` where x is 0, empty or FALSE converts to the text "0" (no sign); the oracle cannot tell
whether that value is a negative zero or +0. `-0&0.5` → "00.5",
`-0&"x y"` → "0x y", `-Data!C5&1` with C5 empty → "01", `0.1&-'Q''1'!B$4` → "0.10".
Evidence: gen-m1/f91603cd8616521f Calc!A20, A249, A13 and 9 more cells (A7 A28 A33 A71 A110 A112 A125 A177 A250; A71 is
`-Data!B$3` with B3 = FALSE → "01000");
gen-m1/57fe331cb7611dbf Calc!A235 (oracle: excel-graph, 2026-10-05, T-0109).
Status: VERIFIED. Implementation: `number_to_text` (T-0107).

## Overflow in `+ - * /` is #NUM! where it happens
Rule: a sum, difference, product or quotient beyond the double range is the error `#NUM!` at that
point, so it propagates through `&`, CONCAT, TEXT and LEN as an error. It never becomes an
"infinite" number that is formatted later: `CONCAT(1e307*100)`, `(1e307*100)&""`,
`TEXT(1e307*100,"0.000000000000000000")` and `LEN((1e307*100)&"")` are all `#NUM!`. The same holds
for `=9E+307+9E+307`, `=9E+307-(-9E+307)`, `=-9E+307-9E+307`, `=9E+307/0.1`, `=-9E+307/0.1` and the
negative products `=-1E+307*100`, `=1E+307*-100` (ERROR.TYPE 6). The intermediate counts:
`=9E+307+9E+307-9E+307` is `#NUM!`, not 9E+307. `SUM(9E+307,9E+307)` is `#NUM!` too. A result of
exactly 1e308 is finite: `=1E+307*10-10^308` is 0, and `=10^308*10` is `#NUM!`.
Arrays overflow per element: `INDEX({1E+307,1}*100,1)` is `#NUM!`, `INDEX({1E+307,1}*100,2)` is 100,
`ROWS({1E+307;1}*100)` is 2 and `SUM(IFERROR({1E+307,1}*100,0))` is 100.
Aggregate overflow (T-0192, VERIFIED, probes/aggregate-overflow, excel-graph 2026-10-06): `SUM` overflows at
the intermediate point, like `+`: `SUM(9E+307,9E+307,-9E+307)` is `#NUM!` while `SUM(9E+307,-9E+307,9E+307)`
is 9E+307. A cell error beats the overflow wherever it sits; of several errors `SUM(1/0,NA(),..)` gives `#DIV/0!` (argument order, or `#DIV/0!` outranking `#N/A`: one case, not separated):
`SUM(9E+307,9E+307,NA())` and `SUM(NA(),9E+307,9E+307)` are `#N/A`, `SUM(1/0,NA(),9E+307,9E+307)` is `#DIV/0!`,
`SUM(9E+307,9E+307,"x")` is `#VALUE!`, `SUM(F1:F4)` (a range holding 9E+307, 9E+307, -9E+307, #N/A) is `#N/A`
and `AGGREGATE(9,6,F1:F3,NA())` is `#N/A` (option 6 ignores errors in the referenced range, not an error passed directly as an argument). These overflow to `#NUM!` (ERROR.TYPE 6): SUMPRODUCT,
AVERAGE (also `AVERAGE(9E+307,9E+307,-9E+307)`), PRODUCT (`PRODUCT(1E+200,1E+200)`, and still `#NUM!` with a later
factor 0 or 1E-200), SUMSQ (`SUMSQ(1E+200)`), SUBTOTAL 9, 1 and 6 (and 7, 8, 10, 11, 107, 108, 110 and 111 over three 1E+200, probes/subtotal-variance-one-vs-two-pass K10:K17), AGGREGATE 9 and 1 (option 6), SUMIF, SUMIFS,
SERIESSUM, MMULT, SUMX2PY2, SUMXMY2, DEVSQ, STDEV and VAR, and `SUM(F1:F2)+0`, `SUM(F1:F2)&""`,
`MAX(F1:F2)+F1` propagate it. Not an overflow: `COUNT` (3), `SUMX2MY2({9E+153},{0})` (8.1E+307, a finite near-limit result: SUMX2MY2 overflow itself is unprobed),
`MEDIAN(F1:F2)` (9E+307) and `GEOMEAN(F1:F2)` (9E+307, exactly), F1:F2 holding 9E+307 twice (direct arguments not probed). `PERCENTOF(F1:F2,F1:F2)`
and `PERCENTOF(F1,F1:F2)` are `#NUM!` (an overflowing sum, like SUM; probes/percentof-snap-overflow-union (excel-graph, 2026-10-13, T-0416)).
Implementation: `+ - * /` (T-0173), `SUM` (T-0185), and SUMPRODUCT, SUMSQ, PRODUCT, AVERAGE and SUBTOTAL 1, 6, 9
(T-0990, `CalcResult::finite_number_or_num` on the final result, which covers a later 0 or 1E-200 factor
since an infinity stays infinite or turns NaN; SUBTOTAL 101/106/109 share the code but are unprobed), and
SUMIF, SUMIFS, SUMX2PY2, SUMXMY2, DEVSQ, STDEV and VAR (T-0991, the same check). Probed (T-1021, probes/aggregate-overflow Calc!A49:C62): Excel gives `#NUM!` (ERROR.TYPE 6) for VAR.S, STDEV.S, VAR.P, STDEV.P, VARA and STDEVA over
F1:F2 (9E+307 twice) and over F5:F6 (1E+200 twice), and for the spilling array-criteria SUMIF per criterion:
`INDEX(SUMIF(F1:F3,{">0","<0"}),1,1)` is `#NUM!` while `...,1,2` is -9E+307. The engine matches for VAR.S, STDEV.S and SUMIF;
VAR.P, STDEV.P, VARA and STDEVA have matched since T-1037, when all twelve VAR/STDEV functions moved onto `excel_variance` with the same check (this closed T-1029).
Equal huge values still overflow: F5:F6 is 1E+200 twice (true variance 0) and Excel gives `#NUM!` for all six (A55:A60). Excel tries the one-pass sum of squares first and only then switches to two passes (statistics.md, "VAR/STDEV switch"), so do not "fix" this to 0.
MEDIAN halves before adding when the sum of the two middle values overflows, and GEOMEAN takes the n-th root of a power-of-two-scaled product when the plain product leaves the normal range, with the whole part of the root's power of two applied exactly and its remainder under the root, `(m * 2^r)^(1/n) * 2^q` (T-0992, T-1076; in range see "GEOMEAN in-range root", T-1113). Mixed-magnitude GEOMEAN is probed (probes/geomean-mixed-magnitude, excel-graph, 2026-10-07, T-1032): `GEOMEAN(1E+100,9E+307)` is 9.486832980505138e203 (the engine was one ulp high with `2^(e/n)` as a float power; it matches since T-1076), `GEOMEAN(1E-100,1E-300)` is 1.0000000000000001E-200 (one ulp above 1E-200; the engine matches bit for bit) and `GEOMEAN(1E+154,1E+300)` is 1E+227, and three equal 1E+200 or 1E-200 values give that value; the engine matches all five. Past 500 values (probes/geomean-overflow-many-values, excel-graph, 2026-10-07, T-1095): GEOMEAN over 600 cells alternating 1E+300 and 3E+250 is 1.7320508075688774E+275, over the first 501 of them 1.940817314759203E+275, over 600 x 1E+300 exactly 1E+300 and over 501 cells holding 7E+280 in every third row and 2E+200 otherwise 1.4094597464129783E+227; the engine matches all four. With 599 equal values and one half-size one, which reached the engine's then remainder-over-500 branch, 599 x 1E+300 and 5E+299 is 9.988454217380303E+299 and 599 x 1E-300 and 5E-301 is 9.988454217380302E-301, and the engine matches both. The underflowing counterpart, 600 cells alternating 1E-300 and 3E-250, is 1.7320508075688772E-275 and the engine is one ulp high (T-1099; its remainder is 0, so the miss is on the ordinary root path). Since T-1099 the engine keeps the product as a significand in [1, 2) (the bits of the plain left-to-right product) and puts the whole remainder r < n under the root; the remainder-over-500 branch above is gone (F6 and F7 still match), and the one-ulp miss is explained under "GEOMEAN root rounding". AGGREGATE A27-A29 pass since T-0993
("AGGREGATE" below).
Evidence: probes/aggregate-overflow Calc!A1:C62 (aggregates); probes/arith-overflow Calc!A1:D7, A10:D15; probes-numeric/4f413cc257ce15c4 Calc!A121-A124;
probes/precision-and-rounding A12, A13. Oracle: excel-graph, 2026-10-06.
Status: VERIFIED
Implemented in: `arithmetic::{plus, minus, times, divide}` (crates/rustsheet-engine/src/arithmetic.rs);
tests `overflowing_product_is_num_inside_formulas` and
`overflowing_sum_and_quotient_are_num_and_a_subnormal_divisor_is_zero`. SUM overflow: the `fn_sum` total check (T-0185).
Tasks: T-0136, T-0173

## A subnormal divisor is zero
Rule: a divisor below the smallest normal double (2.2250738585072014E-308) divides like zero:
`=1/1E-308`, `=1/(2^-1023)` and `=1/(1E-307/10)` are `#DIV/0!` (ERROR.TYPE 2), while
`=1/(2^-1022)` (the smallest normal) is the finite 4.49423283715579E+307. The same flush as
"Subnormal results become 0": the divisor reads as 0. An underflowing quotient is plain 0:
`=1E-307/1E+300` is 0.
Evidence: probes/arith-overflow Calc!A8:D9, A16:D18. Oracle: excel-graph, 2026-10-06.
Status: VERIFIED
Implemented in: `arithmetic::divide`.
Tasks: T-0173

## Division rounds twice (64-bit, then double)
Rule: the operator `/` rounds the exact quotient first to a 64-bit significand (x87 extended
precision) and then to a double, both to nearest, ties to even. This differs from the correctly
rounded quotient only when the exact quotient lies within half an extended ulp of the midpoint
between two doubles (about 1 division in 4,000 with full-significand operands): the first rounding
lands on the midpoint, and the second rounds it to the even neighbor.
`=1/(2-2^-52)` is 0.5, where the correctly rounded quotient is 0.5000000000000001;
`=1/(1+26955297717044*2^-52)` = 0.994050331987417 (correctly rounded: 0.9940503319874169).
The reciprocal inside `^` and POWER is the same division: `x^-1`, `POWER(x,-1)`, `x^-3` (1 over
the square-and-multiply power), `x^-0.5` (1 over `sqrt`) and `x^-0.3` (1 over `exp(0.3·ln x)`) all
round twice, so `8^(-1/3)` = 0.5 and `(-8)^(-1/3)` = -0.5 although `8^(1/3)` = 1.9999999999999998.
Evidence: probes-numeric/power-reciprocal-rounding (72 cells: 8 values whose reciprocal rounds
differently, each through `^-1`, `POWER(,-1)` and `/`, 4 controls, 6 each through `^-3`, `^-0.5`
and `^-0.3`, every one double-rounded; oracle: excel-graph, 2026-10-13, T-0221);
probes-numeric/power-odd-roots A50, A56 (`(±8)^(-1/3)`).
Other operations (probes-numeric/double-rounding-ops, 80 cells with operands picked by exact
simulation so that single and double rounding give different doubles; every cell matches the
double-rounded value; oracle: excel-graph, 2026-10-13, T-1496): `x/y` with neither operand 1
(8 pairs) and `SUMPRODUCT(x/(y+{0}))` round twice, as do `a*b` and `PRODUCT(a,b)` (the 106-bit
product, 8 pairs), `a+b`, `a-b`, `SUM(a,b)` and `AVERAGE(a,b)` (operands whose exponents differ by
12-52, so the sum needs more than 64 bits; 8 pairs each), and `SQRT(x)` and `x^0.5` (8 values).
Implemented for `*` and PRODUCT by `arithmetic::extended_product` (T-1498, A1-A16 pass) and for
`+`, `-`, SUM and AVERAGE (each addition of the running total) by `arithmetic::extended_sum` (T-1499,
A33-A64 pass) and for SQRT and `^0.5` by
`arithmetic::extended_sqrt` (T-1500, A65-A80 pass). Unprobed: the `sqrt` inside `x^-0.5` stays the plain
root (Excel may divide by the 64-bit root without rounding it to a double first), and a subnormal
radicand takes the plain root. AVERAGEA, SUMIF(S) and the database
sums accumulate with the plain `+` and are unprobed (T-1516).
Range edges of `*` and PRODUCT (probes-numeric/double-rounding-range-edges, excel-graph, 2026-10-14,
T-1513): the second rounding happens at the result's own precision. An exact product just under the
midpoint between `DBL_MAX` and 2^1024 rounds up to the midpoint in 64 bits and then, tied, to 2^1024:
`#NUM!` although the singly rounded product is 1.7976931348623157E+308 (3 pairs, one negative; a
control well under the midpoint stays `DBL_MAX`). An exact product just under the midpoint between
the largest subnormal and 2^-1022 is the smallest normal 2.2250738585072014E-308 (3 pairs): the
second rounding ties on the subnormal grid (multiples of 2^-1074), not on 53 bits. Both `*` and
PRODUCT agree. Implemented in `arithmetic::extended_product`.
Each of these operations, taken alone, rounds to a double twice. Intermediates do not stay at 64
bits: in a chained expression every operator's result is rounded (twice) to a double before the next
operator uses it. probes-numeric/double-rounding-chained (54 cells; operands picked so that keeping
64-bit intermediates would give a different double; oracle: excel-graph, 2026-10-13, T-1501): all
of `a*b+c` (+ the control `SUM(a*b,c)`), `a*b-c`, `(a+b)*c`, `a*b*c`, `a*b/c`, `a/b*c`,
`SQRT(a*b)` and `SQRT(a+b)` (6 operand sets each) equal the value with a double-rounded
intermediate, none the 64-bit-intermediate value and none the exact (fused) value.
No operand set exists for a division by a small constant: `x%` (= x/100), `x/3` and AVERAGE of
three cells divide by a number whose binary expansion repeats within a few bits, so the run of
zeros or ones that double rounding needs never occurs (3,000,000 random operands found no
difference for `x/100`, or for `(a+b+c)/3`); so this probe cannot say whether they round twice (they have oracle values in A81-A89, which the engine matches). MOD and QUOTIENT cannot
be told apart by this construction: `MOD(0.3,0.1)` = 0.09999999999999998, `MOD(1,0.1)` =
0.09999999999999995, `QUOTIENT(0.3,0.1)` = 2, `QUOTIENT(1,0.1)` = 10 are recorded only.
The engine rounds once for `*`, PRODUCT, `+`, `-`, SUM, AVERAGE, SQRT and `^0.5` (tasks T-1498,
T-1499, T-1500); `/` already passes.
Implemented in: `arithmetic::extended_quotient`, used by `arithmetic::divide` and the reciprocals in
`arithmetic::power`.
Status: VERIFIED (excel-graph)
Tasks: T-0221, T-1496, T-1498, T-1499, T-1500, T-1501

## x87 precision is per operator, not per kernel (T-1524)
Rule: where a kernel's formulation is fitted, it is evaluated in doubles, each operation behaving
like the Excel operator of the same name ("Division rounds twice"). The same formulation with
every intermediate held at 64 bits (x87 registers) and one final rounding scores lower on every
such sweep: PV, FV, NPER, `^` with integer and non-integer exponents, EXP(n*LN(x)) and GAMMA at
integers n <= 88 (table). The twice rounding itself is visible inside a kernel only in `^`:
square-and-multiply with each product rounded twice gives all 42 cells whose base is
2.23606797749979 (`SQRT(5)^n` and `(0.2^-0.5)^n`, n = 3..255 and -99, plus
`(SUM(0,(0.2^(-0.5)))^99)` and `POWER(SQRT(5),99)`), and with products rounded once only 2.
On the other sweeps the data do not separate twice-rounded operations from plain doubles. For
the kernels whose formulation is still unknown (PMT, IPMT, PPMT, NPV, GAMMALN, GAMMA at
half-integers, ERFC) the x87 model is unsupported rather than refuted: it only ties the
correctly rounded value, because holding a kernel at 64 bits makes it almost correctly
rounded, while Excel's error is wider (up to 3.5 ulps for PMT). Bit-exact counts (fit lab,
`rustsheet-fitlab <fit>`; "ext" = every intermediate at 64 bits, one final rounding; full tables
in T-1524's Outcome):

| fit (cells) | doubles, Excel operator model | ext (64-bit intermediates) | correctly rounded | engine |
|---|---|---|---|---|
| `annuity-pv` (308) | **308** | 174 | 69 | 107 |
| `annuity-fv` (308) | **308** | 230 | 53 | 101 |
| `annuity-nper` (95) | **95** | 6 | 6 | 61 |
| `exp-ln` EXP(n*LN(1+r)) (154) | **154** | 47 | 154 (LN, EXP each correctly rounded) | 154 |
| `power-1+r` (1+r)^±n, POWER (462) | **462** | 155-161 (1+r at 64 bits / a double) | 161 | 462 |
| `power-mantissa` literal bases ^3..255 (76) | **76** | 10-11 | 10 | 76 |
| `power-sqrt5` SQRT(5)^n (20) | **20** | 1 | 1 | 1 |
| `power-nonint` (180 of 183 modelled) | **180** | 94 | 93 | 183 |
| `power-recip` (45) | **45** | 37 | 20 | 45 |
| `gamma-int` GAMMA(1..171) (171) | **92** (downward product) | 40 | 40 | 1 |
| `gammaln-int` (171) | 82 | 80 | 80 | 108 |
| `gamma-half-int` (62) | 34 | 45 | 45 | 0 |
| `erfc-x87` ERFC tail (205) | 58 (T-1187 fraction) | 44 | 44 (128-bit) | 44 |
| `annuity-pmt` (308) | 91 (129 with P-1 = expm1) | 132 | 132 | 54 |
| `annuity-ipmt` / `annuity-ppmt` (308) | 195 / 43 | 218 / 85 | 219 / 94 | 189 / 15 |
| `annuity-npv` (88) | 27 | 26 | 18 | 21 |

The x87 instruction model (`X87Insn`: fyl2x, frndint, f2xm1 and fscale, each rounded once to 64
bits) is no better than ext as a whole power: 161, 94 and 37 on `power-1+r`, `power-nonint` and
`power-recip`. A hybrid that computes the positive power that way and then takes Excel's
twice-rounded 1/ for a negative exponent beats ext (175, 101 and 43), only because the 1/ is the
Excel operator, and it stays far below the double model (462, 180 and 45). A 64-bit 1+r (as an x87
register would hold it) does not help either (155 against 161). Where ext ties correctly rounded
(PMT, IPMT, PPMT, GAMMA at half-integers), both are accurate; that is no evidence about Excel's
algorithm. The lead this task started from, "PMT with `(1+r)^n` from the exact 1+r, kept at 64
bits, 133/308", is this tie: correctly rounded gets 132, and Excel's PMT lies up to 3.5 ulps from
the exact value, which no model accurate to half an ulp can follow.
Evidence: the oracle sweeps probes-numeric/m9-fin-pv-fv, m9-fin-pmt, m9-fin-nper-npv and
m9-fin-power (excel-graph, 2026-10-06, T-0942); power-integer-full-mantissa,
power-reciprocal-rounding and power-non-integer-sweep (excel-graph, 2026-10-12/13);
t1042-gamma-int-1-90, -91-171 and -half-int; t1040-erfc. Fits: `tools/src/rustsheet_tools/fitlab/kernels/`
`annuity.py`, `power.py`, `gamma.py`, `erfc.py` (`erfc-x87`).
Ours: the engine needs no extended-precision type. It already has the per-operator model
(`arithmetic::extended_quotient`, `extended_product`, `extended_sum`); the fits' wins are kernel
formulations that use it. PV/FV, NPER and GAMMA at integers are recorded in financial.md and
statistics.md, `^` in "Integer exponents beyond 360" below.
Status: VERIFIED (excel-graph) for the fitted kernels listed; for PMT, IPMT, PPMT, NPV, GAMMALN,
GAMMA at non-integers and ERFC the x87 model is unsupported (it ties correctly rounded) and the
kernels stay unexplained (time-boxed, decision 0056).
Tasks: T-1524, T-0901, T-1495, T-0973, T-0968, T-1528, T-1529

## Numbers near 1E+307 print with 14 significant digits
Rule: `=(1/(2^-1022))&""` is "4.4942328371558E+307" (14 digits) although the cell value is
4.49423283715579E+307 (15 digits, A16); the engine prints 4.49423283715579E+307.
Evidence: probes/arith-overflow Calc!A16:B16. Oracle: excel-graph, 2026-10-06.
The 20-character-cap hypothesis was wrong: the cut is `|exp| >= 99` (see "Exponent width" under
"Number → text conversion uses 15 significant digits"; probes/number-text-exponent-digits, T-0186).
Status: VERIFIED. Implemented in `number_to_text` (T-0199); the rounding is half up on the 15-digit string,
and a carry that takes the 14-digit mantissa to 10^14 raises the exponent (VERIFIED, T-1000: `9.99999999999995E+99&""` → "1E+100", `E-100` → "1E-99", `E-99` → "1E-98", `E+100` → "1E+101", `E+307` → "1E+308", `-9.99999999999995E+99` → "-1E+100"; `9.99999999999994E+99` → "9.9999999999999E+99" with no carry; at `|exp| <= 98` there is no cut, `E-98` and `E+98` keep all 15 digits. Evidence: probes/number-text-exponent-carry, excel-graph, 2026-10-07).

## Criteria equality
Rule: COUNTIF/SUMIF-style numeric criteria compare the 15-digit rounded values (no absolute epsilon): over {1E-300, 0, 1E-17, 2E-16, 1+2^-52},
`COUNTIF(r,0)` = 1 and `"=0"` = 1, `COUNTIF(r,1)` = 1 (1+2^-52 matches 1), `"<>1"` = 4. Lookup ordering (MATCH/XMATCH/XLOOKUP) never equates tiny values with 0.
Evidence: probes/tiny-vs-zero-criteria (A1-A33); the general 15-digit rule rests on the Comparison tolerance entry below. The blank-cell criterion path (`result_is_equal_to_empty`) uses the same rule but is inferred, not probed.
Implemented in: `numbers_equal` (functions/criteria.rs), `Ord for CalcResult` (calc_result.rs). Tasks: T-1028. Oracle: excel-graph, 2026-10-06. Status: VERIFIED

## Comparison tolerance
Rule: comparisons round both numbers to 15 significant digits and compare the results exactly, so
operands a few ULP apart can be equal or not depending on where the 15th digit falls. Measured with X = b·(1+k·2^-52) against b: `=`, `<>`, `<=` and `>` treat X as equal to b for
k ≤ 2 at every base; for k = 3–8 only some bases (7 of 12 still equal at k = 8). `<` and `>=` are
uninformative here because X ≥ b in every probe. No absolute tolerance exists: rounding both operands to 15 significant digits and comparing exactly
reproduces all 648 cells (T-0989; it also fixed the 2^-10 base, where an absolute epsilon wrongly said equal).
Evidence: probes-numeric/99bc3137b5869417 `compare-tolerance/*` (648 cells). Oracle: excel-graph, 2026-10-04.
Status: VERIFIED
Implemented in: `cmp_excel_precision` (number_format.rs), used by `compare_values` (functions/util.rs) and `Ord for CalcResult` (calc_result.rs). Tasks: T-0989, T-0180

## 15-digit literal parsing
Rule: numeric literals, `"text"+0` and `VALUE("text")` keep only 15 significant digits;
digits 16+ are **truncated**, not rounded: `VALUE("9307149050110945.41")` = 9307149050110940,
`=1234567890.12345678` = 1234567890.12345. Truncation matched 450 of 450 probes.
Evidence: probes-numeric/23733e52a2ff91ab `literal-15-digit/*`; probes/precision-and-rounding A11.
Oracle: excel-graph, 2026-10-04.
Status: VERIFIED
The same truncation applies to numeric text in COUNTIF/SUMIF criteria (`COUNTIF(r,"123456789012345678")`
matches a cell holding 123456789012345000, and `"<…"`/`">=…"` compare against the cut value) and to
`NUMBERVALUE` and `AVERAGE` text arguments (probes/text-number-15-digits B1-B8, B11-B13; excel-graph,
2026-10-06).
Implemented in: `parse_excel_number` in crates/rustsheet-engine/src/number_format.rs, used by the lexer number token, `cast_number`, `parse_formatted_number` (so `VALUE`), criteria operands, `NUMBERVALUE` and `AVERAGE`, and COUNT's direct text (via `cast_number`, T-0252). Other sites keep `parse_finite_number` because the digit count cannot change their answer: the date-criterion check only tests number-ness; the sequence detector (autofill sources are already 15-digit values), conditional-formatting date bounds, `formatter/lexer.rs` condition operands and utils.rs are not affected in practice. `"<17 digits>"+0` is cut the same way (probes/text-number-15-digits B14, excel-graph 2026-10-06). Unverified: autofill over E-notation numbers (1E-10, 2E-10) copies instead of extending in our engine; Excel's behavior is unknown (T-0171).
Tasks: T-0143, T-0150

## Subnormal results become 0
Rule: a result below the smallest normal double is returned as 0: `=2^-1074` → 0,
`=CONCAT(1e-307/10)` → "0".
Evidence: probes/precision-and-rounding A14; probes-numeric `number-to-text/*/1e-307/10`;
probes/text-number-words H6, H7, K1, K4, K5 (`="1e-308"+0` is 0).
Oracle: excel-graph, 2026-10-04.
Status: VERIFIED
Per function group (T-0189, VERIFIED, probes/subnormal-function-results, excel-graph 2026-10-06;
`C` companion cells `(x)=0` tell a true 0 from a kept subnormal, since the Graph API reports 0.0 and
`&""` prints "0" for both). The result is a true 0 (`(x)=0` TRUE) for the postfix `%`
(`=1E-306%`, `=-1E-306%`), `/` and `*` (`1E-307/10`), PRODUCT (`PRODUCT(3E-308,0.5)`,
`PRODUCT(1E-200,1E-110)`), SUMPRODUCT, SUMSQ (`SUMSQ(1E-160)`), AVERAGE (`AVERAGE(3E-308,-2.9E-308)`)
and POWER. The subnormal is KEPT (`(x)=0` FALSE although it prints "0") for `-` (`=3E-308-2.5E-308`),
`SUM` / `SUBTOTAL(9,..)` of the same operands, `EXP(-745)`, `EXP(-720)` and for MIN, ABS and SIN
of such a value. So the flush belongs to multiplication and division (`%`, PRODUCT, SUMPRODUCT, SUMSQ,
AVERAGE all multiply or divide), not to addition. `EXP(-708)` (3.3E-308, normal) prints with 14
digits ("3.3075530036384E-308"; see "Exponent width" under "Number → text conversion uses 15
significant digits"). `TEXT(1E-307/10,"0.00E+00")` is "0.00E+00". A literal below the smallest normal
reads as 0: `=1E-308=0` is TRUE and `ISNUMBER(1E-308)` is TRUE (A70, A71; `=1E-308*1=0` cannot tell this from the
`*` flush). Oracle artifact: for a kept subnormal the Graph API reports the value 0.0, so the numeric
cells A4, A6, A60-A62, A65 and A69 cannot score the engine; only their `C` companions can (T-0989).
The probe leaves out
literals with exponent below -308 (`1E-310`, `1E-320`, `1E-620`): the Graph workbook builder rejected the
file (`unsupportedWorkbook`) when they appeared.
Scoring (decision 0042): an excel-graph 0 against a kept subnormal scores as a pass when the case has a
FALSE excel-graph companion `(x)=0` for the same formula, since Graph reports both as 0.
Implemented in: `arithmetic::plus`/`minus` keep subnormals (T-1027); number text below the smallest
normal reads as 0 (`Model::parse_number_text`; VERIFIED for the strings `1e-308` and `1e-309` only, T-1143: the `(x)=0` companions of `("1e-308"+0)`, `("1e-308"-0)`, `VALUE("1e-309")` and `VALUE("1e-308")` are all TRUE, probes/subnormal-text-and-tail-companions); `arithmetic::finite_or_num` flushes `*`, `/`, `^` and POWER; `arithmetic::flush_subnormal`
flushes the results of PRODUCT and SUMPRODUCT (each running product, T-1007), SUMSQ (each square, T-1057), AVERAGE, AVERAGEA, AVERAGEIF(S) and
SUBTOTAL 1/101 (final result; T-0987, T-1007), and of the postfix `%` (T-0988).
Subnormal tails of functions (T-1143, VERIFIED at the probed arguments only, probes/subnormal-text-and-tail-companions, excel-graph
2026-10-08): these return a true 0 (companion `(x)=0` TRUE) at `CSCH(710)`, `NORM.S.DIST(k,TRUE)` for k = -37.6, -37.75 and -38,
`NORM.S.DIST(k,FALSE)` for k = -37.75 and -38, `PHI(-37.7)`, `NORM.DIST(19,0,0.5,FALSE)`, `ERFC(26.7)` and `ERFC.PRECISE(26.7)`.
It is a flush of the subnormal result, not an overflow or a cutoff (boundary probe below: CSCH(709.3) and CSCH(709.6), in the band
where e^x is finite, are 0 too). A normal result is kept:
`NORM.S.DIST(-37.6,FALSE)` is 4.0441448093482273E-308 (companion FALSE); `standard_normal_pdf` (the PHI operation order) reproduces it, `normal_pdf`'s `-0.5*d*d / (SQRT_2PI*sd)` order is one ulp off. The NORM.S.DIST TRUE tail is
also recorded in the m9-norm-s-inv notes below and in statistics.md. The engine flushes the result of CSCH, NORM.S.DIST, NORM.DIST, PHI, ERFC and ERFC.PRECISE with `arithmetic::flush_subnormal` (T-1180);
the boundaries are probed (probes/subnormal-function-boundaries, excel-graph 2026-10-08): the result is a true 0
exactly where it would be subnormal, a normal result is kept. `CSCH(709.3)`, `CSCH(709.6)`, `CSCH(-710)` and `CSCH(-709.6)` are
0 (companion TRUE), `ERFC(26.5)` and `ERFC.PRECISE(26.5)` are 2.2109076642637346E-307 (kept), `ERFC(26.6)` is 0,
`NORM.S.DIST(-37.61,FALSE)` is 2.776580428767599E-308 (kept; the density is 1.9E-308 at -37.62) and `-37.62` to `-37.65`
are 0. This is a flush of the subnormal result, not a cutoff in the algorithm. `ERFC(26.4)` is 4.4017768588035533E-305
and the engine is a few ulp off (T-1186). Swapping `normal_pdf`'s constant for `(2*pi).sqrt()` regresses NORM_DIST.xlsx NORM.INV!E13 and E17, so NORM.DIST keeps its order.
Subnormal flush edges (T-1007, VERIFIED, probes/subnormal-flush-edges, excel-graph 2026-10-07): a subnormal literal argument
reads as 0 (`PRODUCT(1E-308,1E+300)` is a true 0, `SUM(1E-308,1E-308)*1E+300` is 0). PRODUCT and SUMPRODUCT flush every running
product, not just the total: `PRODUCT(1E-200,1E-110,1E+300)` is 0 (1E-310 is flushed before the 1E+300 factor scales it back up;
flushing only the total would give 1E-10), and `SUMPRODUCT({1E-200,1},{1E-110,1E-300})` is exactly 1E-300 (the 1E-310 term is
dropped; keeping it prints 1.0000000001E-300). AVERAGEA, AVERAGEIF, AVERAGEIFS, SUBTOTAL(1,..), SUBTOTAL(101,..) and
AGGREGATE(1,..) flush a subnormal mean to a true 0 like AVERAGE (`C` companions TRUE) while SUM of the same operands keeps it.
SUMSQ flushes each square too (T-1057, VERIFIED, probes/subnormal-sumsq-sumproduct-edges, excel-graph 2026-10-07):
`SUMSQ(1E-155,1E-150)` is exactly 1E-300 (the 1E-310 square is dropped; keeping it prints 1.0000000001E-300), and
`SUMSQ(1E-155*ROW(1:200))*1E+300` is 0.0002685685, the sum of k^2 for k = 15..200 only (squares below 2.2E-308, k <= 14, are dropped).
The order does not matter: `SUMSQ(1E-150,1E-155)` is also exactly 1E-300, so it is each square that is flushed, not the running sum.
SUMPRODUCT with three arrays flushes after each factor, left to right: `SUMPRODUCT({1E-200},{1E-110},{1E+300})` is 0 while
`SUMPRODUCT({1E-200},{1E+300},{1E-110})` is 1E-10.
Comparison and literals (T-0989): a comparison decides on the two 15-digit values alone, with no absolute tolerance, so
`EXP(-708)=0` is FALSE (C4-C6, C60-C62, C65 pass), while a literal below the smallest normal is flushed to 0 when it is
read (`=1E-308=0` is TRUE). Still failing: the engine flushes the result of `+` and `-` (`3E-308-2.5E-308`, C69, C73-C75;
Excel keeps it), and EXP(-745)/EXP(-720) (A4, A6). Fixing `+`/`-` regresses A69, A73-A75 and the text-number-tiny/words
cells, because Graph reports 0.0 for a kept subnormal: a scoring decision is needed (T-1027).
Tasks: T-0179

## Formula literals of 1E+308 or more are invalid
Rule: Excel's largest typeable number is 9.99999999999999E+307. A workbook whose formula text
contains a literal ≥ 1E+308 (`=1E+308`, `=A2-1E+308`) is rejected on load by Excel for the web
(observed there only; the inputs are not committed because Excel cannot load them)
("unsupportedWorkbook"); compute such values instead (`=10^308`, `=1E+307*10`). Probe and
generator inputs avoid these literals for that reason.
Evidence: T-0009 single-cell workbooks sent to excel-graph, 2026-10-04: `=1E+308`, `=1E+308-1`,
`=1.7976931348623157E+308`, `=A2-1E+308` rejected (createSession → 501 unsupportedWorkbook);
`=1E+307`, `=1E+307*10`, `=10^307*10`, `=1E+300*1E+300` loaded. Before the fix,
probes/precision-and-rounding A13 and the `1e308*10` cells of probes-numeric `number-to-text` failed the same way.
Status: VERIFIED

## Number text of 1E+308 or more is not a number
Rule: text read as a number (`"x"+0`, `--"x"`, `VALUE`, `COUNT`/`SUM`/`AVERAGE` text arguments,
array elements, criteria operands) is a number only below 1E+308 in magnitude, the same limit
as formula literals: `="1e308"*1`, `="1.7976931348623157e308"*1`, `="1e400"*1`, `="-1e400"+0`,
`VALUE("1e400")`, `SUM("1e400",1)` are #VALUE!, while `="9.99999999999999e307"*1` and
`="0.01e309"+0` (1E+307) are numbers. The words "inf", "NaN", "infinity" (any case or sign) are
text, never numbers (NUMBERVALUE too: `NUMBERVALUE("1e308")` and `NUMBERVALUE("inf")` are
#VALUE!; `COUNT("1e308",1)` is 1): `="inf"*1`, `=--"Infinity"`, `VALUE("NaN")` are #VALUE!, `COUNT("inf","NaN",
"1e400","+1")` is 1 (only "+1" counts), and `COUNTIF(r,"inf")` / `">1e400"` compare as text. A
`TEXT` format condition whose number is out of range (`[>1e400]`) makes the format invalid
(#VALUE!), `[>1e308]` included.
Small magnitudes (probes/text-number-tiny and text-number-tiny-zero; Excel, `0+A1` and `VALUE(A1)`
agree on every row): a nonzero value below 1E-309 is #VALUE!
(`5e-310`, `9.99e-310`, `9.9999e-310`, `-1e-310`, `"1e-323"`, `"4.9e-324"`, `VALUE("1e-400")`,
`0.001e-307`), while `1e-309`, `100e-311` and `1.000001e-309` are numbers: the test is on the
value's leading digit, not on the written exponent. From 1E-309 up to and
including DBL_MIN 2.2250738585072014E-308 (text-number-words H6; text-number-tiny `2.2250738585072e-308`, `2e-308`,
`1.5e-309`, `-2e-308`, `-1e-309`) the text reads as 0 (not the subnormal value; sign unverified); `2.2250738585073e-308` keeps its value, as do `9.9e-308`, `1e-307`, `1.1e-307`.
Zero mantissas are tested on where their last digit sits, the exponent less the fraction digits:
within ±308 the text is 0, beyond it #VALUE! (0: `0e308`, `-0e308`, `0e-307`, `0e-308`,
`0.0e309`, `0.00e310`, `0.000e311`, `00.0e309`, `.0e309`; #VALUE!: `0e309`, `0e+309`, `-0e309`,
`00e309`, `0.e309`, `0.0e310`, `0e-309`, `0e-310`, `0e-400`, `0.0e-308`, `0.0e-309`, `0.00e-307`).
Number text longer than 255 characters is #VALUE! for every form probed (the same limit as date-clock text, dates.md; T-1097, probes/text-number-long-zero, excel-graph
2026-10-07, 78 text cells, `0+A` and `VALUE` agree, LEN confirms the text arrived): at LEN 255 and below
`0.`+zeros, `1.`+zeros, leading-zero integers, all-zero integers, `1` plus trailing spaces and a tiny nonzero
fraction (`0.`, zeros, `1`, which keeps its value, 1E-253 at LEN 255) read as numbers (0, 1, 1, 0, 1, 1E-253), and at LEN 256, 257, 260,
300, 306, 307, 308 and 320 all six forms are #VALUE!. This explains the 305- and 320-zero cells above (LEN 307 and 322).
The engine applies the limit through the shared number-text coercion (`cast_number_in`), VALUE and NUMBERVALUE, counting surrounding spaces (`MAX_NUMBER_TEXT_LEN`, shared with clock text; T-1101).
Other paths (probes/text-number-long-paths, excel-graph 2026-10-07, text of LEN 255 vs 256): `SUM(text,1)` and `AVERAGE(text,2)` read 255 and give `#N/A` at 256 (QUIRK; T-1107: any direct text argument over 255 characters, numeric or not, padded with spaces or not, is `#N/A` in SUM, AVERAGE, SUMSQ, PRODUCT and MAX, while 255 characters of non-numeric text is `#VALUE!`; probes/sum-long-text-args), `COUNT(text,1)` counts the text at 255 and skips it at 256 (2 vs 1), `ROUND`, `ABS`, unary minus, `(text)%` and NUMBERVALUE are `#VALUE!` at 256. T-1121 (probes/long-text-args-min-criteria, excel-graph 2026-10-07): the same `#N/A` at LEN 256 (`#VALUE!` at 255 non-numeric) holds for MIN, AVERAGEA, MAXA and MINA direct text arguments. A criterion over 255 characters is `#VALUE!` in COUNTIF, COUNTIFS, SUMIF, SUMIFS, AVERAGEIF, AVERAGEIFS, MINIFS and MAXIFS, whether a literal, a cell reference or `"="&text`; at 255 it matches normally. In an array or multi-cell range criterion only the element over 255 characters is `#VALUE!`, shorter elements match normally (`COUNTIF(B1:B3,B1:B3)`, A24:A26: 1, 1, `#VALUE!`; array constants cannot hold REPT, so only ranges are probed). Against other errors (T-1125, probes/ifs-long-criterion-precedence, excel-graph 2026-10-07) a bad criteria range (`#REF!`) wins over the long criterion, while the long criterion still gives `#VALUE!` next to a `#N/A` criterion in either position, and mismatched range shapes are `#VALUE!` anyway so their order cannot be seen. A `#REF!` sum range, a non-range criteria range and ranges on different sheets against the long criterion are unprobed (the engine resolves them first). DATEVALUE with padding and cell input (`-` plus long text typed into a cell) are unprobed.
Formula literals follow the same rule: `=1E-309`, `=-1E-309`, `=2E-308`, `=0E308`, `=0.0E309`,
`=0E-308` evaluate to 0 and `=2.2250738585073E-308` keeps its value (probes/literal-number-tiny),
while Excel refuses to open a workbook holding `=1E-310`, `=-1E-310`, `=5E-324`, `=0E309` or
`=0E-400`, one per workbook (excel-graph createSession: FileCorruptTryRepair, 2026-10-07).
Every other text-to-number path follows the same rule (probes/text-number-tiny-paths):
`AVERAGE("1e-310",2)`, `AVERAGEA("1e-310",2)`, `SUM("1e-310",1)`, `NUMBERVALUE("1e-310")`,
`NUMBERVALUE("0e309")`, `="1,000e-313"+0`, `="$1e-310"+0` and `="1e-310%"+0` are #VALUE!, while
`AVERAGE("1e-309",2)` is 1, `NUMBERVALUE("1e-309")` and `="1,000e-312"+0` are 0 and
`COUNT("1e-310","1e-309",1)` is 2. A criterion `"1e-310"` or `">1e-310"` is a text criterion
(it matches the text "1e-310" and compares as text), `"1e-309"` and `"<1e-309"` compare with 0,
and a `TEXT` format condition `[>1e-310]` or `[>0e309]` makes the format invalid (#VALUE!). The
cut-off is on the written leading digit, not on a rounded value:
`"0.9999999999999999999e-309"+0` is #VALUE!.
The sign of negative flushed values is unverified (the oracle JSON cannot tell -0 from 0).
Typed into a cell (probes/typed-number-tiny, excel-graph 2026-10-10, T-1100): `1e-310`, `0e309`,
`1e-400` and `1E+308` stay text (ISNUMBER FALSE); `1e-309`, `2e-308`, `1e-308` and `0e308` are
stored as the number 0 (`=A1=0` TRUE). Cell entry follows the same rule as `VALUE` and number text.
Evidence: probes/text-number-words (A1-A10, B1-B5, C1-C4, D1-D3, E1-E6, H1-H10, J1, K1-K10, L1-L8); probes/text-number-tiny (Data!B1:D20); probes/text-number-tiny-zero (Data!B1:E22); probes/literal-number-tiny (Lit!A1:C8); probes/text-number-tiny-paths (Paths!C1:D23); probes/typed-number-tiny (Sheet1!A1:D8).
Oracle: excel-graph, 2026-10-05, 2026-10-06, 2026-10-07 and 2026-10-10.
Status: VERIFIED (conditional-format rule values: UNVERIFIED, T-1299)
Implemented in: `in_excel_range`, shared by `parse_excel_number` (`cast_number`, `VALUE`, the
lexer, criteria operands, AVERAGE text, NUMBERVALUE) and `parse_finite_number` (format
conditions, conditional formats, the criteria date guard),
in crates/rustsheet-engine/src/number_format.rs; the tiny side (T-0154) in `parse_excel_number`
(`cast_number`, VALUE, the lexer, criteria operands, AVERAGE/AVERAGEA text arguments, formatted
number text, NUMBERVALUE) and, for validity only, the TEXT format-condition lexer. Cell entry
(`set_user_input`) shares `parse_excel_number`, as does `value_needs_quoting` (T-1100). Conditional
formats still read tiny text unchanged: their rule values are not observable through the oracle
(excel-graph reads cell values only), so that needs desktop Excel (M14, Phase 4, decision 0039).
Tasks: T-0920, T-0140, T-0154, T-0174, T-1097, T-1100, T-1299

## MOD is an exact remainder
Rule: `MOD(n,d)` = exact `fmod(n,d)`, plus `d` when the result is non-zero and its sign differs from `d`.
Not `n - d*INT(n/d)` (which loses precision: `MOD(5,1E-10)` would give 0).
Evidence: ironcalc-calc MOD_QUOTIENT.xlsx (Excel Online cached) C20 `MOD(10.0000000001,0.1)`=9.999945316252479e-11,
C21 (d=-0.1) =-0.09999999990000055, C25 `MOD(5,1E-10)`=9.999981783901343e-11, C26 (d=-1E-10)=-1.821609865774887e-16.
Status: VERIFIED (Excel-cached)

## SINH / COSH / TANH are exp-based, not libm
Fit against Excel-cached values in `ironcalc-calc/MATH_AND_TRIGONOMETRY/TRIGONOMETRIC.xlsx`
(sheet `SINH_COSH_TANH`; T-0914): COSH = `(e^x + e^-x)/2` (34/34 cells; libm 33/34); SINH uses the
same exp form for |x| >= 0.5 and the Taylor series below (B10, B18, B42; libm misses B42);
TANH = `(e^x - e^-x)/(e^x + e^-x)` for |x| >= 1 (all cells), `sinh/cosh` below 1 (D18, D19), ±1 for
|x| > 20. Status: PARTIAL — TANH at 0.00625/0.123/0.604 and COTH/CSCH/ASINH/ACOSH/ATANH/ACOTH still
differ by an ulp; COTH/CSCH/SECH deliberately stay on libm because the exp forms regressed B10, B47,
C54 of `COTH_CSCH_SECH`. Evidence is cached values only (no oracle probe).
T-0111 (empirical fit): for 0.9 <= |x| < 1 TANH uses `(e^2x - 1)/(e^2x + 1)`. Two cached cells need
it: x = ±0.9999999999 (`SINH_COSH_TANH!D15/D17`, sinh/cosh and libm miss) and x = -0.9950546550253772
(poi `FormulaEvalTestData_Copy.xlsx` `EverythingTests!G1360`, sinh/cosh and the exp quotient both
1 ulp high; libm and `e^2x` fit). The 0.9 cutoff is a guess between 0.604 (sinh/cosh fits) and 0.995;
confirm with an oracle probe of TANH on [0.5, 1) once T-0004 lands.

### NORM.S.INV / NORMSINV / NORM.INV(p,0,1): oracle sweep (T-1006)
Probe `probes-numeric/m9-norm-s-inv` (639 sweep cells = 213 arguments x 3 functions, plus 9 flush/subnormal cells, oracle: excel-graph, 2026-10-08) is the fitting set for T-0925.
- The three functions agree on all 213 arguments (same value or same error).
- NORM.S.DIST returns 0 in its far tail instead of a subnormal (a flush of the subnormal result, see "Subnormal tails of functions"): `NORM.S.DIST(k,TRUE)=0` is TRUE for k = -37.6, -37.75
  and -38 (A640-A642) and FALSE for k = -37.5 (A643), where the value is 4.605353009581954E-308 (A646). The true Phi(-37.6) is about
  1.07E-309 (subnormal) and Phi(-37.5) about 4.6E-308 (normal), so the cutoff is where Phi crosses the smallest normal double, near
  k = -37.52 (unprobed). The bare values A644/A645 read 0.0 but are not flush evidence on their own: Graph also reports a kept
  subnormal as 0.0. The engine keeps the subnormal there (A640-A642, A644, A645 fail), which a NORM.S.DIST follow-up (T-0968, T-1040)
  must match.
  Hence `NORM.S.INV(NORM.S.DIST(k,TRUE))` is `#NUM!` at k = -38 and -37.75 (the engine returns -38.000000000082615 and -37.74999999999998) and
  finite at k = -37.5; this is the ordinary p = 0 `#NUM!`, not a subnormal-p rejection. NORM.S.INV accepts a subnormal p:
  `NORM.S.INV(3E-308-2.5E-308)` = -37.55912122001426 (A647; the subtraction keeps the subnormal, T-1027 covers that in the engine).
  A648 `NORM.S.INV(2.3E-308-1E-309)` = -37.5184972450123 is not a subnormal p: the literal 1E-309 reads as 0 ("Subnormal results become 0", T-1007),
  so it is NORM.S.INV(2.3E-308). 1E-300 gives -37.0470962993612.
- Excel does not round-trip: `NORM.S.INV(NORM.S.DIST(k,TRUE))` is mostly 0-2 ulp off k for k <= 2, counting ulp as the spacing of
  doubles at |k| (k = +-1 give a result below |k|, 2 ulp(1) = 4 doubles away); 3 ulp at k = -31.75, -30.5, -29.25 and -25.5.
  Above 2 the error grows overall, not monotonically, with the resolution of p near 1 (k = 2.5 is -4 ulp, 3.0 is -7, 3.25 is
  exact, 3.5 is -38, 3.75 is -71, 4.25 is +706, k = 8 gives 7.991573950493902). Inferred, not measured: it comes from rounding p,
  not from the inverse, since the inverse is within 1 ulp where p is well resolved (k = -37.5 gives -37.50000000000001,
  k = 0.5 gives 0.5000000000000001, k = -0.5 gives -0.5000000000000001). NORM.S.INV(0.5) is exactly 0.
- Values at the AS241 region edges and around 0.5 (no pattern is claimed, T-0925 fits them): p = 0.02425 - 1E-6 gives
  -1.972978604638878, 0.02425 + 1E-6 gives -1.9729434985927772, 0.97575 - 1E-6 gives 1.972943498592777, 0.97575 + 1E-6 gives
  1.9729786046388784, 0.5 - 1E-9 gives -2.5066283428845328E-9, 0.5 + 1E-9 gives 2.5066282037387115E-9.
- 1 - 1E-k for k = 1..15 is in the case (for example k = 10: 6.361340889697421, k = 15: 7.941444487415977).
Evidence: `probes-numeric/m9-norm-s-inv` (`rustsheet-eval show probes-numeric/m9-norm-s-inv.case.json`). As of 2026-10-08, 192 of 639
sweep cells pass: 438 ulp-level and 3 close mismatches, plus the six `#NUM!` cells (A1-A6). Of the 9 flush/subnormal
cells, A643 and A648 pass; A640-A642, A644 and A645 (NORM.S.DIST keeps the subnormal) and A646 (close) are NORM.S.DIST cells
(T-0968, T-1040), and A647 is the subtraction flush (T-1027).
Status: VERIFIED (excel-graph); the NORM.S.INV mismatches are tracked by T-0925.

### Inverse hyperbolics, COTH/CSCH and the SINH/TANH thresholds: oracle sweep (T-1005)
Probe `probes-numeric/m9-hyperbolic` (268 cells, 255 distinct formulas, oracle: excel-graph, 2026-10-08) is the fitting set for T-0917.
- ASINH and ACOSH are the plain log formulas, not libm: all 65 ASINH/ACOSH cells equal `ln(x + sqrt(x*x + 1))` (negative x:
  `-ln(-x + sqrt(x*x + 1))`) and `ln(x + sqrt(x*x - 1))` bit-for-bit in IEEE doubles. So small arguments lose precision:
  ASINH(1E-10) = 1.000000082690371E-10 (relative error 8e-8), ASINH(3E-10) = 3.000000247771113E-10,
  ASINH(1E-9) = 1.000000082240371E-9, and ACOSH(1.000000000001) = 1.414276423083533E-6 (about 7e-11 relative below the
  true value 1.4142764231805424E-6). The engine returns the correctly rounded value (ASINH: 22 failing cells, 6 of them clear mismatches).
- ACOTH for large x (A101-A110, x = 15 .. 1.5E10) tracks libm `atanh(1/x)` within 1 ulp: bit-for-bit on A105 and
  A107-A110, 1 ulp off glibc `atanh(1/x)` on A101-A104 and A106 (A103 matches Rust's `f64::atanh`), so Excel is not
  correctly rounded there either; the exact formula is left open for T-0917. The engine uses `0.5*ln((x+1)/(x-1))`
  there and loses digits: ACOTH(1.5E7) engine 6.666666662802374E-8 vs Excel 6.666666666666676E-8 (6e-10 relative),
  ACOTH(1.5E10) engine 6.666667217824696E-11 vs Excel 6.666666666666667E-11 (8e-8 relative).
- Near 1, ACOTH(1 + 1E-12) = 14.162039699929037 and ATANH(1 - 1E-12) = 14.162095209226402 differ only because the stored
  inputs sit at different distances from 1 (x-1 = 1.0000889E-12, 1-y = 0.99997788E-12); both are correctly rounded and the
  engine matches both cells. Not a quirk.
- COTH and CSCH (A117-A156) are close to, but not exactly, libm `1/tanh(x)` and `1/sinh(x)`. COTH differs by 1 ulp at
  1E-4, 0.01, 0.1, 2 and 5 (A123, A127, A129, A135, A137) and by 2 ulp at 1E-3 (A125); CSCH differs by 1 ulp at 1E-8 and
  0.01 (A120, A128). Notably CSCH(1E-8) = 1E8 exactly, where libm `1/sinh(x)` (and the engine) gives 99999999.99999999.
  The other COTH/CSCH cells equal the libm expressions bit-for-bit, except CSCH(710) (below); the formula is left open for T-0917.
- Overflow: SINH(710) and SINH(-710) are `#NUM!` (engine: +-1.117E308), but CSCH(710) = 0 (not an error; engine 8.95E-309),
  CSCH(709) = 2.433561501246846E-308, COTH(709) = COTH(710) = 1, TANH(x) = 1 for every probed x >= 19 (19, 19.5, 19.99, 20, 20.01,
  21, 22, 25, 30, 350, 700, 709, 710). SINH(709.78) = 8.964113971972578E307 is finite.
- SINH/TANH values on 0.40..1.0 (step 0.01 / 0.02 / 0.025, plus 0.9950546550253772 and 0.9999999999) are in the case; the
  engine's 0.5 (SINH) and 0.9 (TANH) cutoffs are fitted against them by T-0917. Not probed: SINH in (709.78, 710).
Evidence: `probes-numeric/m9-hyperbolic` (`rustsheet-eval show probes-numeric/m9-hyperbolic.case.json`).
Status: VERIFIED (excel-graph); the engine mismatches are tracked by T-0917.

### SIN/COS/TAN/SEC/CSC/COT near multiples of PI(): oracle sweep (T-1008)
Probe `probes-numeric/m9-trig-reduction` (666 cells = 111 arguments x 6 functions, oracle: excel-graph, 2026-10-08) is the fitting
set for T-0915. Arguments are `k*PI()+d` (k = 1, 2, 3, 10, 100, 1000, 1E4, 1E5, 1E6; d = 0, +-1E-12, +-1E-9, +-1E-6, +-0.25) and
`k*PI()/2+d` (k = 1, 3, 5, 11, 101, 1001; d = 0, +-1E-12, +-1E-9). No cell is an error.
Observed (no formulation is claimed; T-0915 fits it):
- Excel differs from the correctly rounded result of the double argument by an amount that grows with k, the number of periods
  reduced, and is not tied to how close the argument is to the multiple: `SIN(PI())` = 1.22514845490862E-16 (the correctly rounded result,
  which the engine returns, is 1.2246467991473532E-16), `SIN(3*PI())` = 3.67544536472586E-16, `SIN(10*PI())` = -1.22514845490862E-15,
  `SIN(10000*PI())` = -4.861840097181158E-13 (engine -4.85682353956849E-13), `SIN(100000*PI())` = -3.396567055391486E-11,
  `SIN(1000000*PI())` = -2.232413837122138E-10. `CSC(PI())` = 8162276138809536 (engine 8165619676597685).
- Away from the multiples the offset persists: `SIN(PI()+1E-12)` = -9.999663857368501E-13 (engine -9.999664359024263E-13),
  `SIN(PI()+1E-9)` = -9.999999602255255E-10, `SIN(PI()-1E-9)` = 1.0000002052552165E-9, `SIN(PI()+1E-6)` = -1.0000000000170965E-6.
  For SIN(k*PI()+d) with |d| <= 1E-6, Excel minus engine is about (-1)^(k+1) * k * 5.0166E-20 on 57 of the 63 such rows (the rest
  differ by rounding), for every d in the grid. The offset only vanishes in the result when it is under half an ulp of the value
  (for example d = +-0.25 with small k).
- `SEC(PI()/2)` = 1.6324552277619072E16 (engine 1.633123935319537E16), `COS(2*PI()+1E-9)` = 1, `SIN(PI()/2)` = 1, `SIN(3*PI()/2-1E-9)` = -1.
- Hypothesis, not a rule: Excel's `SIN(PI())` is exactly the 15-digit literal 1.22514845490862E-16 and `SIN(2*PI())` is exactly twice that
  with the opposite sign, which would fit a reduction by a two-part pi whose low part is that literal (true pi - fl(pi) is 1.2246467991473532E-16).
Evidence: `probes-numeric/m9-trig-reduction` (`rustsheet-eval show probes-numeric/m9-trig-reduction.case.json`). As of 2026-10-08,
247 of 666 cells pass; the 419 others differ from the engine (252 clear mismatches, 152 close, 15 ulp).
Status: VERIFIED (excel-graph); the engine mismatches are tracked by T-0915.

## Non-integer exponents: exp(y·ln x)
Rule: for an exponent that is not an integer, Excel computes `x^y` as follows:
- `y` > 0: `exp(y·ln x)`. `6^2.5` = 88.18163074019438, 2 ulps below libm `pow`, and `10^33.5` = 3.162277660168401E+33.
- `y` < 0: one over the positive power, `1/exp(-y·ln x)`. `10^-33.5` = 3.1622776601683576E-34 and `27^-2.5` = 0.0002639918926335737.
- `y` = 0.5: `sqrt(x)`. `2^0.5` = 1.4142135623730951, where `exp(0.5·ln 2)` is 1.4142135623730950.
- `y` = -0.5: `1/sqrt(x)`. `3^-0.5` = 0.5773502691896258 and `0.5^-0.5` = 1.414213562373095.
- Each `1/` is the operator `/`, which rounds twice (see "Division rounds twice"): `8^(-1/3)` = 0.5.

An odd root of a negative base is minus the same formula on the magnitude. `(-8)^(1/3)` = -exp(ln 8/3) = -1.9999999999999998, which is the value Excel's own workbook caches.

Libm `pow` (correctly rounded) matches only 93 of the 180 numeric sweep cells. `exp(y·ln x)` without the ±0.5 cases matches 148. The model matches all 180.

Out-of-range results:
- A positive exponent that overflows is `#NUM!`.
- Quirk: a negative exponent is one over the positive power even when that power underflows, so the result is `#DIV/0!`. This holds for integer exponents too: `1E-10^-33.5`, `POWER(1E-10,-33.5)`, `0.1^-400`, `1E-10^-40`, `1E-10^-31` and `1E-300^-1.1` (where the positive power is subnormal).
- A negative exponent whose result underflows is 0: `1E10^-33.5` = 0, `1E+200^-1.6` = 0, `2^-1074.5` = 0.

`(-8)^(-1/3)` is -0.5: the twice-rounded `1/1.9999999999999998`. The model matches all 156 cells
of `(±x)^(±1/n)` for n = 3, 5, 7 over 13 bases (probes-numeric/power-odd-roots A1-A156, T-0221).
Evidence:
- probes-numeric/power-non-integer-sweep A1-A182: bases 2, 3, 5, 6, 7, 10, 0.5, 1.5, 100, 1E10, 1E-10, 123.456, 0.9 and 1.1, each with exponents ±0.5, ±1.5, ±2.5, ±0.3, 1/3, ±33.5, 0.25 and 2.7.
- probes-numeric/power-non-integer-sweep A183-A194: odd roots of negatives and out-of-range results.
- Both are oracle excel-graph, 2026-10-12, T-1319.
- probes-numeric/power-odd-roots A1-A156 (excel-graph, 2026-10-13, T-0221).
- gen-m1/6462bf910ef9f28f Calc!A41 and A207; gen-m2/1b6ceed7489e0800 Calc!A62; gen-m7/9ff3afcacd3bd0fc Calc!A13.
- ironcalc-calc POWER.xlsx Sheet1!D3 and F9.
Not probed: other negative bases; exponents within 1E-15 of ±0.5.
Implemented in: `non_integer_power` in crates/rustsheet-engine/src/arithmetic.rs.
Status: VERIFIED (excel-graph)
Tasks: T-1319, T-1421, T-0221

## POWER and `^`: odd roots of negatives, zero and overflow
Rule: `POWER(-8, 1/3)` is -1.9999999999999998 (an odd root of a negative number is real; see "Non-integer exponents: exp(y·ln x)"); `POWER(0,0)` is #NUM!;
`POWER(0,-1)` is #DIV/0!; text operands give #VALUE!; array operands work element by element.
Evidence: ironcalc-calc/MATH_AND_TRIGONOMETRY/POWER.xlsx (Excel-authored): Sheet1!D2 (#DIV/0!),
D3 and F9 (-1.9999999999999998, matched since T-1319: see "Non-integer exponents"), sheets 2-8 for ranges.
The operator `^` shares the zero case: `0^-0`, `+0^0`, `(-0)^0` and `0%^0` are all `#NUM!`, while
`-0^0.5` = 0 (gen-m1/6462bf910ef9f28f Calc!A19, A30, A205, A222; gen-m1/1c62c4bff4ed1976 Calc!A60;
oracle: excel-graph, 2026-10-05, T-0109).
Status: VERIFIED for `0^0` → #NUM! (POWER and `^`). UNVERIFIED for (a) overflow of a non-zero base
being #NUM! (not #DIV/0!), (b) the 2^-48 relative tolerance that decides whether an exponent "is"
1/odd (only exactly 1/3 is evidenced; probe `POWER(-8,0.333333333333333)`), (c) the operator `^`
sharing the other special cases.
Implemented in: `arithmetic::power` (POWER and `^`).

## SIN / COS at multiples of 90° return k·6.1257422745431E-17
Rule: where the exact answer is 0, `SIN`/`COS` of k·π/2 return ±k·6.1257422745431E-17 (14
significant digits), not IEEE/libm's ±k·6.123233995736766E-17: `COS(RADIANS(90))` =
6.1257422745431E-17, `SIN(RADIANS(180))` = 1.22514845490862E-16, `COS(RADIANS(270))` =
-1.83772268236293E-16, `SIN(RADIANS(360))` = -2.45029690981724E-16, and so on up to k = 7 (630°). Of the workbook's
other 1,433 scored cells, 5 differ from libm by a few ulp and the rest are bit-exact. The excel-graph oracle and the
desktop Excel that saved the workbook agree.
Evidence: poi/47813.xlsx Numbers!C92, B182, C272, B362, C452, B542, C632 (oracle sidecar
`eval/suites/poi/oracle/47813.oracle.json` and the workbook's cached values). Oracle: excel-graph, 2026-10-04.
Status: VERIFIED
Tasks: T-0915

## ACOS(0.5) is the double nearest π/3
Rule: `ACOS(0.5)` = 1.0471975511965976 (the double nearest π/3); libm returns 1.0471975511965979.
Evidence: poi/48539.xlsx Instance1!E23 (oracle: excel-graph, 2026-10-04); poi/FormulaEvalTestData_Copy.xlsx Q104.
Status: VERIFIED
Tasks: T-0939

## GEOMEAN root rounding
Rule: in GEOMEAN's over/underflow path Excel's result is the n-th root of the same product the
engine forms, rounded faithfully (within 1 ulp of the exact `X^fl(1/n)`) but by no known model: it
is not glibc's `pow(X, 1/n)` (10 of 17 probed sets) nor Excel's own `^` on X (14 of 17), and which
sets miss is not set by the distance from the exact root (misses at 0.357 and 0.397 ulp, matches at
up to 0.498; see the separating probe below). T-1099's "agrees within about 0.38 ulp" bound is refuted.
Evidence: probes/geomean-product-order (17 random sets of 5 to 301 values, magnitudes 1E+-150
to 1E+-307, picked so a sequential and a pairwise product round differently; excel-graph,
2026-10-07, T-1099): the engine matches 10, and Z6, Z7, Z8, Z10, Z11, Z13, Z14 are one ulp off;
probes/geomean-overflow-many-values F4 is the same case (exact root 0.407 ulp from glibc's).
Models simulated against all 29 over/underflow cells (the five geomean-mixed-magnitude, seven
geomean-overflow-many-values and 17 geomean-product-order cells) and rejected, each failing at
least 4: pairwise (tree) or 2- to 32-way interleaved products, an exact or 64-bit product, decimal
mantissas or scaling, [0.5, 1) mantissas with rescaling thresholds, truncated instead of floored
remainder, `exp(ln(X)/n)`, `2^(log2(X)/n)`, `pow(m,1/n) * 2^(E/n)`, the correctly rounded root with
exact 1/n, and x87-style compositions with 53- or 64-bit log, 1/n and product. Probably the same
non-correctly-rounded power as `^` (next section, T-0221).
Separating probe (probes/geomean-root-models, excel-graph, 2026-10-07, T-1108): for each of the 17
geomean-product-order sets, Root!E holds the engine's root argument X = M*2^(r-52) (Excel reproduces it
bit for bit) and F-I take Excel's own n-th root of it. Excel's `^` and POWER agree with each other on
all 17 and give GEOMEAN's root (GEOMEAN / 2^q) on 14, five of the seven engine misses among them
(Z6, Z7, Z8, Z10, Z14); they differ by one ulp on Z5 and Z11 (lower) and Z13 (higher), where glibc's
`pow` matches Z5. So GEOMEAN's root is closer to Excel's `^` (14 of 17) than to glibc's (10 of 17),
but it is not that `^` applied to X either. `EXP(LN(X)/n)` also gives 14 (a different set: it misses
Z1, Z5, Z11) and `10^(LOG10(X)/n)` 11. No tried model reproduces all 17.
The "about 0.38 ulp" agreement bound does not hold: of 8 new sets whose glibc root lies 0.32 to 0.40
ulp from the exact root, Pin!C60 (0.357 ulp) and Pin!F60 (0.397) miss by one ulp and the other six
match.
Log-domain root with the remainder added in logs (T-0290, rejected): `exp((ln(M) + r*ln2)/n) * 2^q`,
the in-range path's `exp(ln/n)` with `r` added to `ln(M)` instead of scaling M by `2^r`. Run over every
probe, it fixes 16 known failures and regresses 9 that the current model passes. The experiment
replaced both of the current model's branches: `(M*2^r)^(1/n)`, and `M^(1/n) * 2^(r/n)` above r = 1000.
- Fixes: geomean-pow2-count L4, M1, M4 (the wrapped n-copies cells); geomean-product-order Z6, Z8,
  Z10, Z13, Z14; geomean-overflow-many-values F4; geomean-stored-3ulps-low R1;
  geomean-exponent-sum C2; exponent-sum-2 C28; overflow-count C1; overflow-cut C1;
  overflow-cut-2 C9; overflow-cut-3 C13.
- Regressions: geomean-product-order Z1, Z5, Z17; geomean-overflow-many-values F1, F3;
  geomean-stored-3ulps-low R2; exponent-sum C9; overflow-cut C26; overflow-cut-2 C8.
Neither model reproduces all cells. Choosing between them per cell would need a selector backed by
evidence, which was not searched for (T-0290 was time-boxed). As with `EXP(LN(X)/n)` in the
separating probe, Excel's root looks like a power rounded differently from both.
Status: VERIFIED (the misses and the probe values); no reproducible root model (T-1108, T-0290).
Tasks: T-1099, T-1108, T-0221, T-0290

## GEOMEAN is #NUM! when the product of significands overflows
Rule: Excel splits every value into a binary significand M in [1, 2) and a power of two, multiplies
the significands into one double without rescaling (the order is unprobed: the probes use equal or
alternating values), and is `#NUM!` when that product
overflows, i.e. when the sum of log2(M) reaches 1024. Within the probed exponent sums, magnitude does not
matter, only M (but large positive exponent sums have a further `#NUM!`, see 2^1000 below):
1028 copies of 5E+204 (M = 1.9934) are finite and 1029 are `#NUM!`; so are 1100 copies of 1.25E+204
(the same M) and 3E+200 (M = 1.96), while 1100 copies of 1E+300 (M = 1.49), 5E+300, 9E+204 or
7E+150 are finite; for M = 1.5 (1.5, 3, 0.75) the cut is 1750 finite / 1751 `#NUM!`; 1E+300
1770 / 1771; 1.9999 1024 / 1025; 1E-300 2400 finite / 2500 `#NUM!` (predicted cut 2428/2429, not probed); 1E+307
(M = 1.78, predicted cut about 1231) is `#NUM!` at every probed count from 2000; powers of two (M = 1) never trip it
(2000 copies of 2), but see 2^1000 below. It applies even when the plain product is in range: 1752 values alternating 1.5
and 0.75 (plain product 1.125^876) are `#NUM!`, 1750 are 1.0606601717798212.
Equal values are not always returned exactly: 1001 copies of 5E+204 give 5.0000000000000006E+204,
1024 copies 4.9999999999999995E+204, 1025 and 1027 copies 5E+204, 1028 copies
5.0000000000000006E+204, 1100 copies of 5E+300 5.000000000000001E+300. In range, 1101 copies of 1.5
give exactly 1.5 (the engine matches since T-1113). 2400 copies of 1E-300 give
1.8066704015823645E-300 = 1E-300 * 2^(2048/2400): the power-of-two sum is 2048 too high there (1100
copies give 1E-300 exactly; T-1120). Bracketed further (probes/geomean-exponent-sum, -sum-2, excel-graph,
2026-10-08, T-1120): the excess is always 2048 * k, so the result is `v * 2^(2048k/n)`. k grows with n,
perhaps doubling the count per step (tentative: only 1E-307 shows two steps). For 1E-300: k = 0 at 1500-1700 copies and k = 1 from 1800 to
2428. For 1E-307: k = 1 from 1700 to 3500 copies and k = 2 from 3900 to 6000. 1E-200 (1000, 1600) and
1E-100 (1000, 1200) give k = 0. Powers of two (significand 1) never gain anything: 2^-1000 and 2^-100 are
exact up to 8200 and 4000 copies. The `#NUM!` and the low results that many copies of a stored 2^1000 once gave were an input artifact, not a
GEOMEAN rule: spec2xlsx used to write numbers through XlsxWriter's `{x:.16G}`, so Excel read
1.071508607186267E+301, three ulps below 2^1000 (a stored 2^1000 minus `2^1000` was -3 * 2^947), with
significand 1.99999999999999933 and so the significand cut above applied (1024 copies finite, 1025 `#NUM!`).
spec2xlsx now writes the shortest text that parses back to the same double (T-0289) and Excel reloads: with
2^1000 stored exactly, 1000-3000 copies of it all give 1.0715086071862673E+301 = 2^1000 (probes/geomean-exponent-sum-2
C11-C18, refilled 2026-10-08), the stored minus the computed 2^1000 is 0 (probes/geomean-pow2-count Q3, Q4), and
R1-R7 are all 1.0715086071862673E+301.
The earlier behaviour is reproduced by storing the 16-digit value itself (probes/geomean-stored-3ulps-low, excel-graph,
2026-10-08, T-1172): `Q1 = 1.071508607186267E+301` satisfies `Q1=2^1000` (TRUE: comparisons round both operands to 15 digits, "Comparison tolerance") yet `(Q1-2^1000)/2^947` is -3; GEOMEAN of 1000 copies is 1.0715086071862671E+301 and of 1024 copies
1.0715086071862668E+301, and 1025, 1026, 1100, 1500 and 3000 copies are `#NUM!` (the significand cut). The engine
matches all but the 1000-copy last ulp (the open root-rounding item above, T-0290).
Computed in the formula, 2^1000 was exact at every probed count, as
are 2^10, 2^100, 2^500, 2^900 and 2^1023 at 500-3000 copies (probes/geomean-pow2-count A-F, J; T-1156).
Fitted rule (T-1120, T-1156): with P the [1, 2) significand product and n the count, k is the number
of times `exponent(P) - n` (the exponent of the [0.5, 1) significand product `P * 2^-n`) falls below
the double range, wrapping modulo 2048: k = ceil((n - floor(log2 P) - 1022) / 2048) when positive,
else 0. As if Excel formed `P * 2^-n` by 11-bit exponent-field arithmetic. It applies only when the
values' power-of-two sum is negative (the product is below the double range) and P is at least 2. It
predicts k for all 41 cells of T-1120 (1E-300: n - floor(log2 P) is 984 at 1700 copies, k = 0, and
1042 at 1800, k = 1; 1E-307: 2912 at 3500 copies, k = 1, and 3245 at 3900, k = 2) and the 30 bracketing
cells of T-1156 (probes/geomean-pow2-count G-I, L-P, excel-graph, 2026-10-08): 2^-1000 times 1.001,
1.01 and 1.1 gain k = 1 wherever the formula is positive (1.001 at 1100 copies, floor(log2 P) = 1),
while 2^-1000 * 1.0001 (P below 2 up to 3000 copies), powers of two (P = 1: 2^-1000 up to 8200, 2^-100
up to 4000, 2 at 1101 and 2000 copies; probes/geomean-overflow-count F1, geomean-overflow-cut C4,
-cut-2 C18) and every positive sum (2^1000 times 1.0001, 1.001 and 1.1 at 1100-3000 copies) are
exact or plain roots. An in-range plain product never wraps: 1100-3000 copies of 1.0001 are 1.0001
(H). Three of the bracketing cells (L4, M1, M4) miss by one ulp of the root ("GEOMEAN root
rounding"). The rule was fitted on T-1120's probes; T-1156's probes tested its predictions and
added the two conditions. Ruled out
earlier: the excess does not depend only on the exponent sum (powers of two with a larger sum gain
nothing), is not a fixed count threshold (1E-300 and 1E-307 step at different counts) and is not the
significand product alone (1E-307 steps at a smaller product).
Evidence: probes/geomean-root-models Fallback!D1-D2, probes/geomean-overflow-count A1-K1 (range form),
probes/geomean-overflow-cut, -cut-2, -cut-3 (array form `GEOMEAN(IF(SEQUENCE(n),v))`, which matches the
range form on the repeated controls), excel-graph, 2026-10-07, T-1108 and T-1112. The predicted cuts
for M = 1.5, 1E+300, 1.9999 and 1E-300 were probed after the rule was fitted, and all held.
Status: the significand cut is VERIFIED within the probed ranges; the 2048 * k wrap is VERIFIED on the
probed cells as a fitted rule (no mechanism for the power-of-two exemption). RustSheet implements both
(T-1112, T-1120). Still open: the root's last-ulp rounding (numbers.md, "GEOMEAN root rounding";
T-1108).
Tasks: T-1108, T-1112, T-1113, T-1120, T-0289, T-1172

## `^` / POWER are not correctly rounded
Rule: Excel's power differs from correctly rounded pow in the last digit: `=2^(-0.5)` =
0.7071067811865475 (pow: …476), `=100^100` = 1.0000000000000005E+200.
Integer exponents are affected too: with x = 1E-15, `x^3` = 1.0000000000000001E-45 (pow: …03E-45);
`3^-0.5` = 0.5773502691896258 (pow: …257).
Evidence: gen-m2/1b6ceed7489e0800 Calc!A62, f0d1bacec5de8146 Calc!A196 (oracle: excel-graph,
2026-10-04, T-0203); gen-m1/6462bf910ef9f28f Calc!A33, A41, A48, A57, A86, A207 (oracle:
excel-graph, 2026-10-05, T-0109).
Status: VERIFIED
Tasks: T-0221, T-1302

### Integer exponents: right-to-left square-and-multiply
Rule: for an integer exponent n, `x^n` and `POWER(x,n)` equal binary exponentiation in doubles,
scanning n's bits from the low end (`r=1; b=x; while n: if n&1: r*=b; b*=b; n>>=1`), and
`x^-n` equals `1/` that result. 154 of 154 cells match for each of `(1+r)^n`, `POWER(1+r,n)` and
`(1+r)^-n` on 11 rates by 14 nper values (1..360); per form (`^n` / `POWER` / `^-n`) libm `pow`
matches 58/58/59, left-to-right square-and-multiply 84/84/86, and correctly rounded pow and
repeated multiplication at most 66. It also reproduces the
integer-exponent cells above: `(1E-15)^3` = 1.0000000000000001E-45, `((1E-15)^3)^2` =
1.0000000000000002E-90, `100^100` = 1.0000000000000005E+200. `EXP(n*LN(x))` is plain libm (154/154).
Non-integer exponents are not explained by this rule.
Evidence: probes-numeric/m9-fin-power.case.json (oracle: excel-graph, 2026-10-06, T-0942).
Implemented in: `integer_power` in crates/rustsheet-engine/src/arithmetic.rs (T-1302), for finite integer
exponents with |n| <= 2^53 whose result is finite and non-zero. Overflow and underflow beyond the
oracle's exponents 1..360 are unverified, so a zero or non-finite result falls back to the libm
`powf` path (and its #NUM! / #DIV/0! handling) unchanged. Exponents beyond 360, subnormal results and base 10 are in the next entry (T-1311).
Status: VERIFIED (integer exponents)
Tasks: T-0221, T-0901, T-1302

### Integer exponents beyond 360, subnormal results, base 10 (T-1311)
Rule: the square-and-multiply rule above holds for every integer exponent tried up to 2^53 and
every base except 10 (about 280 cells across `1.0000001`, `0.9999999`, `1.5`, `2`, `3`, `5`, `7`,
`13`, `20`, `50`, `100`, `1000`, `1E5`, `1E15`, `-10`, `-1.5`, negative n by `1/`). Four additions:
1. A result below 2.2250738585072014E-308 (subnormal) is 0, not the subnormal: `0.1^310`,
   `0.5^1074`, `(-0.5)^1073`, `2^-1074`, `10^-308`, `10^-4294967296` are all 0 (`=(x)=0` companion
   cells are TRUE, so it is a flush and not a kept subnormal; decision 0042).
2. A negative base with |n| >= 2^32 - 1 = 4294967295 is `#NUM!` for every such n, whatever the
   result would be: `(-1)^4294967295`, `(-0.5)^4294967296` (an underflow), `(-1)^-4294967296`;
   `(-1)^4294967294` is 1 and `(-0.9999999999)^3000000000` is 0.7408182005096358. The limit is probed up to 2^53 (larger exponents are treated the same, unverified).
   A positive base has no such limit (`0.5^9007199254740992` = 0, `1^9007199254740992` = 1).
3. Base exactly 10 with |n| >= 23 is the nearest double to the decimal `1En` (`10^33` = 1E+33,
   `10^100`, `10^300`, `10^275` = 1E+275 where square-and-multiply drifts), and a negative n is
   one over that double (`10^-23` = 1.0000000000000001E-23, `10^-289` = 9.999999999999999E-290,
   `10^-300` = 1E-300). `10^23` equals both. Bases `100`, `1000`, `-10`, `0.1` and `20`
   are plain square-and-multiply (`0.1^-33` = 9.999999999999965E+32, not 1E+33):
   `(-10)^33` = -1.0000000000000001E+33, `100^100` = 1.0000000000000005E+200.
4. Overflow is `#NUM!` as before (`100^200`, `1E15^23`, `10^309`).
5. Negative bases follow the same out-of-range rules as positive ones (T-0221): a positive power
   that overflows is `#NUM!` (`(-10)^401`, `(-2)^1025`) and one that underflows is 0
   (`(-0.1)^401`); a negative exponent whose positive power underflows is `#DIV/0!`
   (`(-0.1)^-401`, `(-0.1)^-400`, `(-1E-10)^-31`, `(-0.5)^-1075`, `(-1E-200)^-2`, `POWER(-0.1,-401)`)
   and one whose positive power overflows is 0 (`(-10)^-401`, `(-1E10)^-33`, `(-2)^-1074`,
   `(-1E200)^-2`). Evidence: probes-numeric/power-negative-base-range, power-odd-roots A157-A162
   (excel-graph, 2026-10-13).

Resolved by T-1524 (implementation: T-1495): the loop's products are Excel's `*`, rounded to 64 bits and
then to a double ("Division rounds twice"). With that rounding, square-and-multiply gives all 42
cells of power-integer-full-mantissa whose base is 2.23606797749979: `SQRT(5)^n` and
`(0.2^-0.5)^n` for n = 3..255 and -99 (40 cells), A181 `(SUM(0,(0.2^(-0.5)))^99)` and A182
`POWER(SQRT(5),99)`. `SQRT(5)^99` = 3.9720546451956613E+34 and `SQRT(5)^10` = 3125.0000000000023.
With products rounded once, which is what the engine does, it gives 2 of the 42 (…6807E+34,
3125.000000000003). Fit `power-sqrt5` scores the 20 `SQRT(5)^n` cells (20 against 1); the other 22
were checked outside the lab, which does not parse `0.2^-0.5` or SUM. The double rounding changes no other cell. Both roundings match
all cells of the 7 other full-significand bases: 76 literal-base cells (fit `power-mantissa`) and
the 57 `PI()^n`, `EXP(1)^n` and `SQRT(2)^n` cells (checked outside the lab, which does not parse
function-call bases). They also match the same cells of the 462 `(1+r)^±n` / POWER cells (`power-1+r`)
and of the large-exponent sweeps (`power-large-int`). Earlier rejected: left-to-right and repeated
multiplication, every multiplication order for n = 99, a 54- to 65-bit significand for the loop,
and `exp`/`log2`/`log10` compositions (`exp(n·ln b)` matched 10 of 19). Evidence: probes-numeric/power-integer-full-mantissa (184
cells; excel-graph, 2026-10-13, T-0221); gen-m2/fa437879253827be Calc!A23 is `(0.2^-0.5)^99`
(the same base double).
Non-integer exponents (`10^33.5` = 3.162277660168401E+33, libm 3.162277660168379E+33; `10^-33.5` =
3.1622776601683576E-34; `10^40.0000001` = 1.0000002302585496E+40) follow "Non-integer exponents: exp(y·ln x)" (T-1319).
Evidence: probes-numeric/power-large-integer-exponents{,-2,-3,-4}.case.json (oracle: excel-graph,
2026-10-11, T-1311; the non-integer cells are A54-A56 of -4).
Implemented in: `power` and `integer_power` in crates/rustsheet-engine/src/arithmetic.rs.
Status: VERIFIED (the 2^32 - 1 threshold is bracketed by 4294967294 and 4294967295 exactly; the base-10
rule is sampled at about 20 exponents in 23..=308 and inferred between them)
Tasks: T-1311

## ROUND family with huge num_digits
Rule (inconsistent; Excel is right): with an enormous num_digits ROUND/ROUNDDOWN return the number
unchanged (`=ROUNDDOWN(Data!E3,123456789.123)` → 0.5; `ROUND(Data!$B3,123456789.123)` keeps 1E-15;
`ROUNDDOWN(0.5, 1000000000000005.8)` → 0.5), yet `ROUND(1000000000000000.0,Data!$D$7)` evaluates to 0
(it is the divisor in caa981b0753f455c A270, which is `#DIV/0!`; dd99be1f2454159f A159 is similar).
Evidence: gen-m2/f7e6d351c8538ef5 A35, 893302d6c5b99e2e A136, 7469c3e920e1ade4 A336 (Calc sheet). Oracle: excel-graph, 2026-10-04 (gen-m2 sidecars, T-0203).
Status: UNVERIFIED (the rule separating the two outcomes needs a probe before implementing; T-0218)
Tasks: T-0218

## GAMMALN / GAMMALN.PRECISE (T-0926)

Excel's GAMMALN is not correctly rounded in general: the T-1042 sweep finds 91 of the integers 1..171
differ from the correctly rounded ln((n-1)!) (`GAMMALN(3)` = 0.6931471805599454, one ulp above ln 2;
`t1042-gamma-int-1-90` Calc!A93; see `statistics.md`, "GAMMA and GAMMALN sweep (T-1042)").
Observed in `ironcalc-calc/STATISTICAL/GAMMALN_GAMMALNPRECISE.xlsx`: exactly 0 at x = 1 and 2,
GAMMALN(4) = ln 6 (correctly rounded). The Lanczos `statrs::ln_gamma` is off by a few ulp, so
`special::ln_gamma` uses fdlibm's `lgamma` (`libm` crate; chosen over a vendored port to keep one
maintained definition), which fixed some cells; 12 cells (B/C rows 5, 9, 15, 23, 24, 26) still differ
in the last digit (T-0004, T-0973).

## Excel for the web and desktop Excel differ in the last digits of some functions
Rule: for ATAN2, IMARGUMENT, IMREAL∘IMSQRT, IMCOS, IMTAN, ERFC/ERFC.PRECISE, BESSELJ/BESSELY,
MIRR, IRR and SIN, Excel for the web (the excel-graph oracle) and desktop Excel can differ by up
to 4 ULPs (11 cells), within 1e-9 relative (4 cells), or leave different near-zero residues (3 cells) (two-flow IRR: desktop 2.2e-16, web 0;
`SIN(4)`: desktop -0.7568024953079283, web -0.7568024953079282).
Evidence: docs/research/oracle-calibration.md (18 ironcalc-calc cells, cached by desktop Excel,
recalculated through excel-graph).
Status: VERIFIED (observation). Decision 0030 (rule 4): engine fits target desktop values where
they exist; a web golden that is already a known failure and is missed within 1e-9 relative (or by
a residue below 1e-15) is not a fitting target. Sidecar numbers within 1e-9 of a 16.x desktop
cached value are the desktop value (T-0142; the fill prints a `DESKTOP` line); near-zero residues
(2.2e-16 vs 0) and pre-16 caches stay pending or web. Say which Excel a value comes from when working on these functions.

## GEOMEAN last digits differ from a log-sum evaluation

`=GEOMEAN("47")` is exactly 47 in Excel, and `=GEOMEAN(1,2,"5",TRUE)` is 1.778279410038923;
a log-sum evaluation (`exp(mean(ln x))`) gives 46.99999999999999 and 1.7782794100389228. The
engine matches both since T-1113 (next section). Evidence: Excel for the web (excel-graph) sidecar
`crates/rustsheet-xlsx/tests/calc_tests/STATISTICAL/GEOMEAN.oracle.json`, cells Sheet1!B27, G27.
Values are from Excel for the web, which can differ from desktop in the last digit (decision 0030).
Status: VERIFIED (these two values, web).

## GEOMEAN in-range root
Rule: when the plain product P of the values is a normal double, GEOMEAN is closest to
`r = exp(ln(P) / n)` in doubles, with two exceptions: a single value is returned as is, and when
r's 15-digit value c differs from r and c multiplied by itself n times (left to right, as P is
formed) gives P back, the result is c. So an exact root is exact: GEOMEAN(3,3), GEOMEAN(1,9),
GEOMEAN(1,1,27), GEOMEAN(2,4.5), GEOMEAN(3,27) are 3, 3, 3, 3, 9, and n copies of 3, 7 or 0.1
(n = 2 to 200) are 3, 7 and 0.1, where r is one ulp off. It is not an all-equal shortcut
(GEOMEAN(1,9) is exactly 3 too). The snap is a best fit: it changes none of the ~100 probed cells
that r already matches and fixes 37. Excel's GEOMEAN is faithful (within 1 ulp of the exact root
of P) and usually, but not always, correctly rounded: GEOMEAN(1,3) is 1.7320508075688774 (0.55 ulp
from sqrt(3); the correctly rounded root is …772) and GEOMEAN(1,2,"5",TRUE) is 0.81 ulp off, both
exactly r. Excel's own LN and EXP equal glibc's `log` and `exp` bit for bit on every probed P, and
`EXP(LN(P)/n)` in a cell is r, so the remaining misses are not a libm difference.
Remaining misses (r and c both wrong, one ulp): the 20-value mixed set (geomean-in-range-many A2,
0.61 ulp from the exact root) and 25 of the 48 sets of probes/geomean-root-refinement: 22 of rows
19-48 (picked because r and a Newton-corrected root differ there), A17 (GEOMEAN(7,7,1)), and A7 and
A8. A7 and A8 refute the snap's condition as Excel's own: GEOMEAN(0.01,1) and GEOMEAN(1,0.01) are
exactly 0.1, the correctly rounded root, although 0.1*0.1 is not the double 0.01, so c does not
multiply back to P there.
Rejected models, scored on all 159 in-range cells (geomean-in-range-many, geomean-single-and-copies,
geomean-root-refinement, geomean-overflow-count H1, ironcalc-calc GEOMEAN C23, G27, B27):
- r with all equal values returned as is: 72 of 73 in-range-many, but GEOMEAN(1,9) refutes it.
- The correctly rounded root of P (147 of 159) or of the exact product (144): both break C23 and G27.
- One Newton step from r or from `pow(P, 1/n)`, in the forms `r - (r^n - P)/(n r^(n-1))`,
  `((n-1) r + P/r^(n-1))/n`, `r + (P/r^(n-1) - r)/n`, `r (1 + (P/r^n - 1)/n)`,
  `r (P/r^n)^(1/n)`, `r exp((ln P - n ln r)/n)`, with r^n by `pow` or repeated products: at best
  147 of 159, and each breaks cells that r matches.
- Scaling by the first, last, smallest, largest or mean value, `s (Π(v/s))^(1/n)`: 39 to 63 of 73.
- From T-1113: glibc `pow(P, 1/n)` (71/72, but it breaks C23 and G27), `exp2(log2(P)/n)` (69), a
  log-sum (36), the overflow path's power-of-two-split root (64 or 53), and an extended-precision
  ln/exp (always misses C23 and G27).
- From T-0288, scored on 167 cells (the 159 plus the Copies duplicates): glibc `pow(P, 1/n)` with the
  snap above is the best fit so far, 157 of 167 (the current r with the snap: 140), but it breaks C23,
  G27 and geomean-root-refinement A20, which r matches, so it is not taken. Searching r and its
  neighbours (1 or 2 ulps) for the one whose left-to-right n-th power is nearest P: 152 and 155 (it
  breaks A20 and A33); with binary-exponentiation or `pow` powers 143 to 155. `exp(ln(P) * (1/n))`
  104, `10^(log10(P)/n)` 141, `2^(log2(P)/n)` 138 (149 each with the snap).

Excel's in-range root is a function of the double product P and n alone: every probed way of writing
the same P gives the same result (probes/geomean-same-product: P = 3 as (1,3), (3,1), (0.5,6), (1.5,2),
(0.75,4), (6,0.5) all 1.7320508075688774; P = 10.704 as (6.69,1.6), (1.6,6.69), (10.704,1), (2,5.352),
(0.5,21.408) all 3.271696807468565; P = 10 as four values in five orders all 1.778279410038923). One ulp
of P does matter: GEOMEAN(8.2,1.71) (P = 14.021999999999998) is 3.7445961063911817 and GEOMEAN(14.022,1)
one ulp higher. So the order or precision of the product is not the cause. Its rounding is not decided by the residual or by
the distance from the exact root either: GEOMEAN(1,3) (C23) and GEOMEAN(6.69,1.6) (geomean-root-refinement
A36) are both n = 2, r is 0.55 and 0.59 ulp above the exact root, and r squared misses P by +1 ulp and
the correctly rounded root squared by -1 ulp in both, yet Excel keeps r for the first and gives the
correctly rounded root for the second. The engine gives r for both, so geomean-same-product A9-A15
(P = 10.704) and A21 (GEOMEAN(8.2,1.71)) are one ulp off: the A36 and A29 misses again. That points to a library root (Excel's own power, not glibc's)
whose own error of up to about 0.8 ulp decides the last bit, which no formula in doubles reproduces.
Side finding for `^`: on geomean-single-and-copies Copies!E (Excel's `P^(1/n)`), `exp((1/n)*ln(P))` in
doubles matches 32 of 35 cells and glibc `pow` 19; the three misses are n = 2 (9, 49 and 0.1*0.1 to the
power 0.5), where Excel gives the correctly rounded square root (3, 7, 0.1). It matches only 2 of the 4 non-integer `^` cells of
T-0221, so it is a lead, not a rule.
Single values (probes/geomean-single-and-copies Single!A1-A15): GEOMEAN of 3, 0.1, 1E-300, 1E+300,
7, 0.3, 1.1, PI(), 2^-1022 and 1E+307, of a cell, a one-cell range and a range with text beside one
number, is the value unchanged, where `exp(ln v)` is not (3.0000000000000004 for 3,
1.0000000000000237E-300 for 1E-300).
Evidence: probes/geomean-in-range-many (column A: 0.5+MOD(k*i,d)/d over 20-900 values; column B: n
copies of one value, 50-1100; T-1113), probes/geomean-single-and-copies (Single: one value; Copies:
n copies with Excel's PRODUCT, EXP(LN(P)/n), P^(1/n), LN(P); T-1123), probes/geomean-root-refinement
(T-1123), probes/geomean-same-product (T-0288), ironcalc-calc GEOMEAN C23, B27, G27, probes/geomean-overflow-count H1,
probes/geomean-overflow-cut-3 C1, C5, C11, E1. Oracle: excel-graph, 2026-10-07 and 2026-10-08.
Status: VERIFIED for the probed cells; the formulation is a best fit (26 one-ulp misses in the 159 cells
scored above, plus 8 in geomean-same-product).
Implemented in: functions/statistical/geomean.rs
Tasks: T-1113, T-1108, T-1123, T-0288

## GCD / LCM argument bound
Rule: GCD and LCM accept arguments up to and including 2^53 (as doubles) and `#NUM!` above:
`GCD(2^53,2)` = 2, `GCD(2^53+1,2)` = 2 (2^53+1 reads as 2^53), `GCD(2^53+2,2)`, `GCD(2^54,2)`,
`GCD(1E+16,2)`, `GCD(-2^53,2)` are `#NUM!`. An LCM result is bounded the same way, on the rounded
double: `LCM(2^53,1)` = `LCM(2^53,2)` = 2^53; `LCM(3,3002399751580330+1)` (exactly 2^53+1) = 2^53;
`LCM(2^53,3)`, `LCM(2^52+1,2)` (2^53+2) and `LCM(3037000500,3037000501)` are `#NUM!`.
Microsoft documents `>= 2^53 -> #NUM!`; Excel (web) contradicts that. Desktop Excel was not available
to confirm (decision 0019), so the entry is web-only evidence.
Evidence: probes/resource-limits C1-C4, probes/gcd-lcm-bounds, lcm-result-bound, lcm-result-edge
(excel-graph, 2026-10-05).
Status: VERIFIED (excel-graph only)
Implemented in: functions/math_and_trigonometry/gcd_lcm.rs (`MAX_LCM_GCD`)
Tasks: T-0239

## CHISQ.DIST degrees of freedom and BITAND operands are bounded
Rule: CHISQ.DIST with degrees of freedom above 10^10 is `#NUM!` (10^10 itself is accepted;
CHISQ.DIST.RT/CHISQ.INV/CHISQ.INV.RT share the constant but are unprobed), and BITAND is `#NUM!` for an operand of
2^48 or more (2^48-1 is accepted; the other BIT functions share the constant but are unprobed).
These are Excel's own limits, not deviations.
Evidence: probes/resource-limits C5, C6 (CHISQ.DIST), C7, C8 (BITAND) (excel-graph, 2026-10-05).
Status: VERIFIED
Implemented in: functions/statistical/chisq.rs (`MAX_DEGREES_OF_FREEDOM`); functions/engineering/bit_operations.rs (`MAX`)

## AGGREGATE
Rule: `AGGREGATE(function_num, options, ...)`. Both numbers are truncated (`9.9` is 9, `7.9` is 7), text that reads
as a number and booleans are accepted (`"9"` is 9, TRUE is 1, FALSE is 0, an empty option is 0), and anything
outside 1-19 or 0-7 is `#VALUE!`; an error in either is returned. Options 0-3 skip cells whose formula is a
SUBTOTAL or AGGREGATE (4-7 keep them), options 2, 3, 6 and 7 skip error values, odd options skip hidden rows.
Functions 1-13 (AVERAGE, COUNT, COUNTA, MAX, MIN, PRODUCT, STDEV.S, STDEV.P, SUM, VAR.S, VAR.P, MEDIAN, MODE.SNGL)
take only references, unions included: a value, an array literal, a computed array (`E1:E6*1`) or a nested call
such as `SUBTOTAL(9,E1:E2)` is `#VALUE!`, yet an error argument (`NA()`, `1/0`) is returned whatever the option.
COUNT never returns an error from the range (errors are not numbers) and COUNTA counts errors unless the option
skips them. Over no numbers: COUNT, COUNTA, MAX, MIN, PRODUCT and SUM give 0, MEDIAN `#NUM!`, MODE.SNGL `#N/A`, the
others `#DIV/0!`. Functions 14-19 (LARGE, SMALL, PERCENTILE.INC, QUARTILE.INC, PERCENTILE.EXC, QUARTILE.EXC) take
one array (reference, union or computed array; options 2/3/6/7 skip its errors, the `AGGREGATE(15,6,x/(cond),k)`
idiom) and k: three arguments is `#VALUE!`, an error in k is returned even under option 6, no numbers is `#NUM!`,
and k follows the named function (`AGGREGATE(14,0,E1:E6,2.9)` = `LARGE(E1:E6,2.9)` = the 3rd largest).
Excel for the web refuses to open a workbook holding `AGGREGATE(14,0,E1:E3,E4:E6,2)`, `AGGREGATE(14,6,E1:E6,2,3)`,
`AGGREGATE(9,0,1,2,3)` or `AGGREGATE(2,0,E1:E6,"3",TRUE)` (`createSession` 501 unsupportedWorkbook), so those
forms have no evidence.
An array function_num, options or fourth argument lifts: each element is its own AGGREGATE call, and the results
spill, broadcast like element-wise operators (a dimension of 1 stretches, a row and a column make a grid, and a
position past a shorter array is `#N/A`). Examples:
- `AGGREGATE({9,4},0,E1:E6)` spills 23, 9 (SUM, MAX), and `{9;4}` spills down.
- `AGGREGATE({9,4,1},{0,6},D1:D11)` spills `#DIV/0!`, 10, `#N/A`.
- `AGGREGATE(14,6,D1:D11,{1,2,3})` spills 10, 4, 3, and `AGGREGATE(15,6,E1:E6,{1,2;3,4})` spills a 2x2.
- An array k lifts even in the reference form: `AGGREGATE(9,0,E1:E6,{1,2})` spills `#VALUE!` twice.
- Each element keeps its own form: `AGGREGATE({9,14},0,E1:E6,2)` is `#VALUE!` (2 is not a reference), then 5.
- Invalid elements fail alone: `{1,"x",TRUE}` gives 3.83, `#VALUE!`, 3.83, and `{9,99}` gives 23, `#VALUE!`.
- A range fourth argument is a reference, not lifted: `AGGREGATE({9,4},0,E1:E6,E1:E2)` spills 27, 9.

A lifted call is subject to the *IF(S) cell-visit cap (criteria.md, "*IF(S) lift array criteria", DEVIATION):
the elements to compute times the cells every element reads (the used cells of each reference plus the size of each
computed array, which is evaluated once per call), times the lifted argument count, above 2^27 is `#ERROR!`
(`AGGREGATE(14,6,SEQUENCE(1048576),SEQUENCE(1048576))`, unit test `lifted_calls_over_large_arrays_hit_the_visit_cap`).
A multi-cell range function_num or options lifts like an array (probes/aggregate-range-args, excel-graph, 2026-10-07, T-1058):
`AGGREGATE(G1:G2,0,E1:E6)` with G1:G2 = 9, 4 spills 23, 9 down, a row range spills across, `AGGREGATE(9,G1:G2,E1:E6)` spills
`#VALUE!` (option 9), 23, and a column against a row of options makes a 2x2 grid. A single-cell range is a scalar
(`AGGREGATE(G1,0,E1:E6)` is 23). A 2x2 function_num range spills a 2x2 grid (G1:H2 = 9, 4; 1, 14 gives 23, 9; 3.83, `#VALUE!`), and a 2x2 function_num against a 2x2 options range pairs them element-wise (probes/aggregate-range-function-num-grid A1, A11, excel-graph, 2026-10-09, T-0296). A range fourth argument is not lifted; a multi-cell range k reads its top-left cell: `AGGREGATE(14,0,E1:E6,G1:G2)` is one `#NUM!` (k = 9), and k = J1:J2 (1, 99) gives 9 while K1:K2 (99, 1) gives `#NUM!`, L1:M1 (2, 99) gives 5. In the reference form the range is just another reference (`AGGREGATE(9,0,E1:E6,J1:J2)` is 123). Unprobed (range k): an empty, text or error top-left cell, a range k in the array form, and a k that is a name or `OFFSET` result.
Evidence: probes/aggregate-array-args (48 cells; oracle: excel-graph, 2026-10-07, T-1038) and probes/aggregate-args
Misc!A36:B38 (oracle: excel-graph, 2026-10-06, T-0993).
Evidence: probes/aggregate-grid (217 cells), probes/aggregate-args (53 cells), probes/aggregate-overflow A27:C29.
Oracle: excel-graph, 2026-10-06 (T-0993).
Status: VERIFIED. Hidden and filtered rows and nested SUBTOTAL/AGGREGATE calls: see "SUBTOTAL and AGGREGATE over
hidden and filtered rows" below (T-1036).
Also unprobed: a scalar in the array form (the engine takes a number as one value, returns a scalar error under every
option, and gives `#VALUE!` for text or a boolean), and which error wins between a range and a later error argument.
Known gaps: k rounding and boolean k in LARGE/SMALL (T-1034).
The VAR.P last digit (probes/aggregate-grid B31, C31) has passed since T-1037.
Implemented in: `fn_aggregate` (crates/rustsheet-engine/src/functions/aggregate.rs); the lift reuses
`Model::lift_with_cost`, and the spill shape is `aggregate_static_result` in static_analysis/aggregate.rs (T-1038).

## SUBTOTAL and AGGREGATE over hidden and filtered rows
Rule: a row hidden by hand is read by SUBTOTAL 1-11 and skipped by 101-111; a row hidden by a filter is skipped by
both. AGGREGATE's odd options (1, 3, 5, 7) skip both kinds and its even options read both. Which kind a hidden row is
follows its sheet, not its position: in a workbook as read from a file, on a sheet with a filtered table every
hidden row counts as filtered, including
rows above or below the table and cells in columns outside it, and on a sheet without one every hidden row is a
manual hide (a table without a filter does not change that). SUBTOTAL, and AGGREGATE under options 0-3, skip a cell
whose formula contains a SUBTOTAL or AGGREGATE call anywhere: `=1+SUBTOTAL(9,A1:A2)`, `=SUBTOTAL(..)+1`,
`=-SUBTOTAL(..)`, `=SUBTOTAL(..)*1`, `=IF(TRUE,SUBTOTAL(..))`, `=SUM(SUBTOTAL(..))`, `=LET(x,SUBTOTAL(..),x)`,
`=AGGREGATE(9,0,A1:A2)` and `=1+AGGREGATE(..)` all count as 0, while `AGGREGATE(9,4,..)` reads them and a plain
`=SUM(A1:A2)` is read by all.
Examples (A1:A8 = 1, 2, 4, ..., 128, rows 2 and 5 hidden by hand): `SUBTOTAL(9,A1:A8)` = 255, `SUBTOTAL(109,A1:A8)`
= 237, `SUBTOTAL(110,A1:A8)` = 2444.7, `AGGREGATE(9,0,A1:A8)` = 255, `AGGREGATE(9,1,A1:A8)` = 237,
`AGGREGATE(15,1,A1:A8,2)` = 4. The same values filtered out of a table give `SUBTOTAL(9,..)` = 237 and
`SUBTOTAL(10,..)` = 2444.7.
How the probe makes a filtered row: the table's autoFilter holds a values filter on its first column that keeps
exactly the visible rows, and the hidden rows are written `hidden="1"`; Excel reads such a file as filtered without
re-applying the filter. A worksheet-level autoFilter (no table), a manual hide inside a filtered table made in the
application, and hidden columns are unprobed.
Evidence: probes/subtotal-hidden-filtered (150 cells; oracle: excel-graph, 2026-10-08, T-1036).
Status: VERIFIED
Engine gaps: none known. SUBTOTAL 110 and 111 skip hidden rows (T-1185), and SUBTOTAL 7, 8, 10, 11 and 107, 108, 110, 111 use `excel_variance` like VAR/STDEV and AGGREGATE (which gave the one-ulp match for `SUBTOTAL(10,..)`, probes/subtotal-hidden-filtered Hide!C10). The nested-call skip is implemented (T-1183,
`Node::contains_function`); whether a SUBTOTAL name inside a string, a defined name, an untaken IF branch, an uncalled LAMBDA body or an unused LET binding counts is unprobed (the engine counts any call in the tree).
Implemented in: `fn_subtotal` and `row_hidden_status` / `sheet_has_filtered_table` (T-1182) (crates/rustsheet-engine/src/functions/subtotal.rs),
`fn_aggregate` (functions/aggregate.rs). The case format's `hidden_rows` and `tables` (docs/architecture/eval-harness.md)
load through `Txn::set_row_hidden` and `Txn::add_table`.

## MULTINOMIAL is inexact
Rule: Excel's MULTINOMIAL is not the exact factorial ratio, even when every factorial is exact in a
double: `MULTINOMIAL(2,3)` = 9.999999999999998 (exact 10), `MULTINOMIAL(4,6)` = 209.99999999999977
(exact 210), and over a range holding 4, "2" (numeric text), 6 it is 13859.99999999998 (exact 13860). Non-integers truncate first
(`MULTINOMIAL(2.5,3)` = `MULTINOMIAL(2,3)`). Excel's algorithm is not identified (T-1190).
Evidence: probes/math-argument-modes Sheet1!C9, C35, C47, C49:C53 (excel-graph, 2026-10-08).
Ours: the exact ratio (10, 210, 13860).
Status: VERIFIED
Tasks: T-1082, T-1190

## Huge arguments of trig functions
Rule: `COS(1E+15)` is `#NUM!` in Excel (the argument is too large to reduce). Errors from a left operand win over a
later operand's: in `(x-COS(1E+15))=INDEX(B11:D11,6,2)` the result is `#NUM!`, not INDEX's `#REF!`. An
out-of-domain `ACOS(1E+15)` is `#NUM!` and stays an error when nested (`SERIESSUM(ACOS(1E+15),0,n,m)` is `#NUM!`).
`SIN(1E+15)` and `SEC(1E+13)` are `#NUM!` too, so the threshold is at most 1E+13.
Evidence: gen-m5/c2ce66769f6cc566 Calc!A32; gen-m5/065c0542a4ce3d1c Calc!A26; gen-m5/cd883013bea3b305 Calc!A23 (excel-graph, 2026-10-09);
gen-m6/9bd0bd18636e5f9b Calc!A4 (`SECH(CHOOSE(SIN(1E+15),..))`), gen-m6/3ee7ffc8d6ab061f Calc!A2 (`..-SEC(1E+13)`) (excel-graph, 2026-10-09, T-1239).
More: `SIN(-2.5E+15)` and `CSC(1E+15)` are `#NUM!`, and a nested `ASIN(PI())` or `ASIN(100)` stays `#NUM!`
(`A10<ASIN(PI())`, `CHOOSECOLS(0,ASIN(100))`): gen-m7/9ff3afcacd3bd0fc Calc!A24, gen-m7/f68fafa0ca2eba26 Calc!A37,
gen-m7/93976dd9a3578ed9 Calc!A13, gen-m7/2bb922eef72e785e Calc!A30 (excel-graph, 2026-10-11, T-1275).
Ours: COS, SIN, SEC and CSC compute a number; ACOS and ASIN yield a NaN number that `powf(0)` turns into 1 (and that
comparisons read as FALSE).
Status: VERIFIED for the observed cells; the exact threshold and TAN are UNVERIFIED.
Tasks: T-1192, T-1201, T-1202

## ASIN of a small argument
Rule: `ASIN(-0.0001)` is -0.00010000000016666668 in Excel; libm `asin` gives -0.00010000000016666667 (one ulp off),
while `atan(x/sqrt(1-x*x))` reproduces Excel. Which formulation Excel uses over the whole domain is unknown.
Evidence: gen-m6/2f6b96385ed3facd Calc!A38 (excel-graph, 2026-10-09, T-1239); gen-m7/0e40208f51c2454c Calc!A27 (same value)
and gen-m7/cb748b6a9b81fbb7 Calc!A38 `ASIN(0.42)` = 0.4334453200698859 (excel-graph, 2026-10-11, T-1275), where libm `asin` and both `atan2(x,sqrt(1-x*x))`
forms give 0.43344532006988595 and `atan(x/sqrt(1-x*x))` 0.433445320069886: no candidate fits both cells.
Ours: libm `asin`, one ulp off.
Status: VERIFIED for the cell; formulation UNVERIFIED
Tasks: T-1259

## MULTINOMIAL of a huge argument
Rule: MULTINOMIAL is `#NUM!` once its arguments add up to more than 170, where `sum!` overflows a double. That holds
even when the ratio is small: `MULTINOMIAL(171)`, `(170,1)`, `(86,86)`, `(100,100)`, `(1000)`, `(2^31)`, `(2^53)`,
`(1E+15)`, `(1E+15,0)`, `(1E+15,1)`, `(3,1E+15)` and `(1E+300)` are all `#NUM!`. Sums up to 170 have values:
`(170)` 1.0000000000001974, `(169,1)` 170.0000000000337, `(85,85)` 9.144841845133712e+49, `(150,20)` 5.221123511166111e+25, `(100,70)` 6.491922168402309e+48. `(100,100)` is `#NUM!` though its exact value (about 9e58) fits a double, so the limit is on the sum. `(0)` and `(0,0)` are 1.
The non-integer values are the inexactness in "MULTINOMIAL is inexact" (T-1190).
Evidence: probes/multinomial-large-arguments A1:A20 (excel-graph, 2026-10-09, T-1197); gen-m5/21b9ae9affd6b38c Calc!A25.
Status: VERIFIED (excel-graph)
Implemented in: `fn_multinomial` (functions/math_and_trigonometry/multinomial.rs) checks the saturating sum against 170
before any factorial, so the work is bounded (directive 5).
Tasks: T-1197

## FACT and FACTDOUBLE of a huge argument
Rule: FACT is `#NUM!` once n! overflows a double and FACTDOUBLE once n!! does: `FACT(170)` 7.257415615308004e+306 (`FACT(170.9)` the same, the argument is floored), `FACT(171)`, `(2^53)`, `(1E+15)`, `(1E+300)` `#NUM!`. `FACTDOUBLE(297)` 1.2552756225993057e+304, `(298)` 2.718138023126864e+305, `(300)` 8.154414069380598e+307, `(301)`, `(2^53)`, `(1E+15)`, `(1E+300)` `#NUM!`. Negative: `FACT(-0.5)` and `FACT(-1)` `#NUM!`; `FACTDOUBLE(-1)` is 1, `FACTDOUBLE(-1.5)` and `(-2)` `#NUM!`.
Evidence: probes/fact-large-arguments A1:A18 (excel-graph, 2026-10-09, T-1233).
Status: VERIFIED (excel-graph)
Implemented in: `fn_fact`, `fn_factdouble` (functions/math_and_trigonometry/mathematical.rs) stop at the first infinite product, so the work is bounded (directive 5).
Ours: the overflow boundary and the `#NUM!` results match; the last digits of `FACT(170)`, `FACT(170.9)` and `FACTDOUBLE(297, 298, 300)` do not yet (T-1249).
Tasks: T-1233, T-1249

## BASE min_length is at most 255
Rule: BASE's `min_length` truncates and may be at most 255: `BASE(1,2,255.9)` pads to 255 characters, `BASE(1,2,254.5)`
to 254, and `BASE(1,2,256)`, `(1,2,1E+15)` and `(1,2,"300")` (numeric text) are `#NUM!`, as is a negative one
(`-0.5`, `-1`). A non-numeric number is `#VALUE!` first (`BASE("x",2,1E+15)`); `BASE(-1,2,1E+15)` and `BASE(1,37,1E+15)`
are `#NUM!`.
Evidence: probes/base-min-length (excel-graph, 2026-10-09, T-1243); gen-m6 sample seed 17 `BASE(245,30,1E+15)` = `#NUM!`.
Ours: the same; the bound is checked before padding, which also bounds the allocation (it used to abort the process).
Status: VERIFIED
Implemented in: `fn_base` (crates/rustsheet-engine/src/functions/math_and_trigonometry/mathematical.rs)
Tasks: T-1243

## RANDBETWEEN and RANDARRAY argument checks
Rule: RANDBETWEEN compares the raw `bottom` and `top` (`bottom > top` is `#NUM!`, even
`RANDBETWEEN(5,4.5)`), then draws a whole number from `CEILING(bottom)` to `FLOOR(top)`, and gives
`CEILING(bottom)` when `FLOOR(top)` is below it: `RANDBETWEEN(5.5,5.7)` and `RANDBETWEEN(5.5,6.2)`
are always 6, `RANDBETWEEN(-5.5,-5.2)` always -5. It rejects booleans (`RANDBETWEEN(TRUE,TRUE)` is
`#VALUE!`), reads number text (`"5"`), lifts over an array argument (one draw per element), and
both arguments omitted (`RANDBETWEEN(,)`) is `#N/A`, not 0. RANDARRAY truncates `rows` and
`columns` (`1.9` → 1), gives `#CALC!` for 0 rows and `#VALUE!` for a negative count, `min > max`
or a non-boolean `whole_number` text; with `whole_number` TRUE a fractional `min` or `max` is
`#VALUE!` (`RANDARRAY(1,1,1.5,2.5,TRUE)`, `RANDARRAY(1,1,1,2.5,TRUE)`), while without it `min = max` returns that value
(`RANDARRAY(1,1,0.5,0.5)` = 0.5). Number text is read for every argument, including
`whole_number` (`"TRUE"`) and the boolean-as-number `1`. `TYPE(RANDARRAY(2,2))` is 64,
`ROWS(RANDARRAY())` is 1, and RAND, NOW and TODAY return numbers with TODAY a whole number and NOW
within that day.
Evidence: probes/volatile-invariant Calc!A1:A6, A8:A15, B1:B18, C1:C6, C8:C9, D1:E2, D4:D5,
F1:F9, G1 (excel-graph, 2026-10-10). Every cell was filled twice with identical values (T-1283);
F1:F9 take MIN/MAX/ROWS over 1000 draws in one cell, which is what shows the RANDBETWEEN range
rule (F8:F9: `RANDBETWEEN(5.2,7.9)` spans 6 to 7, not ROUND's 5 to 8); the spill members D2, E1,
E2 and D5 were not compared between fills, but their min equals their max; B15 and G1 have an occupied neighbour, so a 2-row or 2-column result would be `#SPILL!`.
Status: VERIFIED. The engine misses A2, A6, A8, A12, A13, B9, B10 (T-1318; A6 is unchecked until
then, because the engine's draw for it is random), B18, and the array lift F1:F9 (T-1316).

## SERIESSUM with x = 0
Rule (hypothesis): SERIESSUM with x = 0 is `#NUM!` only when a term has power 0 (0^0): `SERIESSUM(0,0,2,{1,2,3})` is `#NUM!`,
but `SERIESSUM(<blank>,123456789.123,-0.5,123456789.123)` (one term, a positive power) is not `#NUM!` (the IFS around
it in gen-m7 gives `#VALUE!`, so SERIESSUM was a number or `#VALUE!`).
Evidence: ironcalc-calc MATH_AND_TRIGONOMETRY/SERIESSUM.xlsx C1, H7 (Excel-cached); gen-m7/d38b52bfa1a9f9e6 Calc!A27 (excel-graph, 2026-10-11, T-1275).
Ours: `#NUM!` for every x = 0.
Status: UNVERIFIED (probe pending: negative powers, a blank x)
Tasks: T-1359

## ECMA.CEILING
Rule: `ECMA.CEILING(number, significance)` is a real Excel function with exactly the two-argument `CEILING` semantics (rounds away from zero for negative `number` with negative `significance`; `#NUM!` when `number` is positive and `significance` negative; zero significance gives 0; number text `"2.5"` and `TRUE` are read as numbers; text significance is `#VALUE!`). It is stored in a file as `_xlfn.ECMA.CEILING`, and `FORMULATEXT` prints it as `=ECMA.CEILING(2.5,2)` (no prefix). Written bare (`ECMA.CEILING(...)`) in a file, Graph rejects the whole workbook (`createSession` 501 `unsupportedWorkbook`); the one-argument form `_xlfn.ECMA.CEILING(2.5)` is rejected the same way (arity is exactly 2, like CEILING).
Evidence: probes/ecma-ceiling A1:A6, A8:A11, A13:A14; probes/ecma-ceiling-formulatext A12 (excel-graph, 2026-10-12, T-1301).
Status: VERIFIED. Ours: implemented (T-1301) as its own `Function::EcmaCeiling` (not an alias of CEILING, so FORMULATEXT keeps the name) dispatched to `fn_ceiling`, with an `_xlfn.ECMA.CEILING` entry in `functions/xlsx_names.rs`. The name is `ECMA.CEILING` in every `language.bin` locale (as upstream IronCalc 8a82396a has it).
Tasks: T-1301

## RADIANS, SQRTPI and MUNIT
Rule: all 94 cells of probes/radians-sqrtpi-munit agree with `x * (PI / 180)` for RADIANS and `sqrt(x * PI)` for SQRTPI to the last bit (`RADIANS(180)` = PI(), `RADIANS(1E-300)`, `SQRTPI(1E-155*1E-155)` and `SQRTPI(10^308)` included), so there are no ulp differences to file.
- **SQRTPI rejects booleans, RADIANS reads them.** `SQRTPI(TRUE)`, `SQRTPI(FALSE)`, `SQRTPI(1=1)`, a reference to a TRUE cell and `SQRTPI({TRUE})` are `#VALUE!`, while `RADIANS(TRUE)`, `RADIANS(D1)` and `RADIANS({TRUE})` are 0.01745...; numeric text (`SQRTPI("4")`) is read, `-TRUE` is a number (`SQRTPI(-TRUE)` is `#NUM!`). In an array the boolean element is the error: `SQRTPI({1,TRUE})` starts with 1.77... (the second element is not probed; we give `#VALUE!`). A negative argument is `#NUM!`.
- **MUNIT truncates its dimension** (`ROWS(MUNIT(2.9))` is 2), is `#VALUE!` below 1 (`MUNIT(0)`, `MUNIT(0.5)`, `MUNIT(-1)`) and for text that is not a number, and reads numeric text and TRUE as 1.
- **MUNIT lifts over an array or range argument**: each element gives the 1 at the top left of its own matrix, so `MUNIT({1,2})` is {1,1} (`ROWS` 1, `COLUMNS` 2, `SUM` 2), `MUNIT({2;3})` is 2 rows by 1 column, and an element of 0 or text makes that element, and so `SUM`, `#VALUE!`.
- excel-graph rejects the workbook (`createSession` 501 `unsupportedWorkbook`) for a literal `1E+308` (as numbers.md "Formula literals of 1E+308 or more" says) and also for the denormal literals `1E-310` and `-1E-310`; the probe computes them (`1E-155*1E-155`). `MUNIT(1048577)` and `MUNIT(1E+10)` were left out of the case for the huge-dimension reason in text.md, unprobed.
Evidence: probes/radians-sqrtpi-munit (excel-graph, 2026-10-13, T-1282).
Status: VERIFIED. Ours: `fn_sqrtpi` (mathematical.rs) reads booleans as errors and lifts, `fn_munit` (matrix.rs) lifts through `lift_number_operands`; SQRTPI's static result is `scalar_arguments` since it can return an array.
Tasks: T-1282
