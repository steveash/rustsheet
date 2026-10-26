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
The 8-ULP bound is itself flushed when it is subnormal (T-2714): with addends 1.5·2^-p and -(1.5·2^-p·(1+k·2^-52)),
k = 1 and 3 snap for p = 960 to 973 (2^(e-49) is at least 2^-1022), even where the exact total is subnormal (p = 972,
973), and nothing snaps from p = 974 on, so a tiny total such as `1E-300-1E-300*(1+2^-52)` stays a kept subnormal
(`(x)=0` FALSE). `+` and SUM agree on every cell. Evidence: probes/t2714-snap-tiny-addends (45 pairs) and the `A+B`
column of probes/t2714-mod-near-smallest-normal (excel-graph, 2026-10-25).
Evidence: probes/precision-and-rounding.case.json A2–A4, A15; probes-numeric/30bc5f8fab02d0a2
`snap-addsub/*` (1,296 cells). Oracle: excel-graph, 2026-10-04.
Status: VERIFIED
Implemented in: `snap_near_zero` (arithmetic.rs), applied at the cell formula's root by `Model::evaluate_formula_root` (an outermost `+`/`-`) and `fn_sum` (every SUM call, functions/math_and_trigonometry/sum.rs); a wholly parenthesized formula is a root `Node::Group` (`Parser::parse_parenthesized`, T-1017, T-1341, T-1392) and is not snapped. Array operands of `+`/`-` are not snapped (unprobed); SUM over array arguments does snap (below).
Groups and names (T-1137, probes/near-zero-snap-groups, excel-graph 2026-10-08): `=((0.1+0.2-0.3))`, `=(((0.1+0.2-0.3)))` and `=((1-0.9-0.1))` are not snapped (5.551115123125783e-17, 5.551115123125783e-17, -2.7755575615628914e-17), like the single-level `=(0.1+0.2-0.3)`; a group that is an operand of an outer `+` (`=(0.1+0.2-0.3)+0`, `=0+(0.1+0.2-0.3)`) is not snapped either. A defined-name body **is** snapped at its own root: `nz` = `=0.1+0.2-0.3` and `nzb` = `=Sheet1!$A$1-0.3` (A1 = `=0.1+0.2`) give 0 for `=nz`, `=nzb`, `=nz+0`, `=nz*1` and `=SUM(nz)`; the engine snaps a name body at its root through `evaluate_formula_root` (T-1160). Name bodies written as a group or a SUM (T-1164, probes/near-zero-snap-name-bodies, excel-graph 2026-10-08): `nzg` = `=(0.1+0.2-0.3)` and `nzgg` = `=((0.1+0.2-0.3))` are not snapped (`=nzg`, `=nzgg`, `=nzg+0` and `=SUM(nzg)` give 5.551115123125783e-17, like the cell forms), while `nzs` = `=SUM(0.1,0.2,-0.3)` is snapped (`=nzs` and `=nzs+0` give 0). The engine keeps a grouped root on a name body (`Model::parse_internal_formula`, `user_formula_to_internal`, the xlsx import re-parse, sheet copy and rename, name display); a parenthesized reference body stays a reference (`rg` = `=(Sheet1!$B$1:$B$2)`, `ig` = `=(INDEX(...))` and `og` = `=(OFFSET(...))` give ISREF TRUE, ROWS/SUM as without the parentheses: probes/defined-name-grouped-reference, excel-graph 2026-10-08), so `is_reference` looks through the group (`Node::ungrouped`). The same parse also keeps the grouped root of a conditional-formatting formula written wholly in parentheses (`user_formula_to_internal` serves both); whether Excel snaps inside CF formulas is UNVERIFIED and not observable through the oracle (T-1171: excel-graph reads cell values only, the case format and the Graph workbook API carry no conditional-formatting rules or their rendered fill; see docs/architecture/oracle.md, Unobservable features), so it needs desktop Excel (M14, Phase 4, decision 0039, T-1204). Unprobed: a formula typed as `=(X-b` that Excel completes with `)`. FORMULATEXT of `=((0.1+0.2-0.3))` is `=((0.1+0.2-0.3))`: the engine keeps every written level around a whole formula (one root `Group` holding every level, T-1161, T-1341).

SUM forms (T-1131, probes/near-zero-sum-forms, excel-graph 2026-10-11): a SUM call snaps its final addition **wherever it sits**, unlike `+`/`-`: `=(SUM(0.1,0.2,-0.3))`, `=SUM(A1:A3)+0` and `=SUM(A1:A3)*1` (A1:A3 = 0.1, 0.2, -0.3) all give 0, as do `=SUM({0.1,0.2,-0.3})`, `=SUM({0.1,0.2},-0.3)`, `=SUM(0.1,{0.2,-0.3})`, `=SUM(A1:A2,A3)` and a range with a blank or text between the addends. A trailing zero addend stops the snap: `=SUM(0.1,0.2,-0.3,0)` and `=SUM(A1:A3,0)` give 5.551115123125783e-17 (the last two addends are -0.3 and 0). 2-D ranges add in row-major order, so `=SUM(D1:E2)` (0.1, 0.2 / -0.3, 0) is 5.551115123125783e-17 (last addend 0) and `=SUM(J1:K2)` (0.1, -0.3 / 0.2, -0.1) is -0.09999999999999998. The engine snaps in `Model::fn_sum` for every SUM call. AGGREGATE(9, ...) and SUBTOTAL(9/109, ...) snap the same way (T-1336, probes/near-zero-aggregate-forms B1-B6, B10-B12, excel-graph 2026-10-11: `=AGGREGATE(9,4,A1:A3)`, `=AGGREGATE(9,6,A1:A3)`, `=SUBTOTAL(9,A1:A3)`, `=SUBTOTAL(109,A1:A3)` and `+0` of each give 0 over 0.1, 0.2, -0.3; `=AGGREGATE(9,4,{0.1,0.2,-0.3})` is #VALUE!); the engine snaps in `subtotal_sum` and the AGGREGATE dispatch. PERCENTOF snaps both of its sums the same way (probes/percentof-snap-overflow-union, excel-graph, 2026-10-13, T-0416; it was `#NAME?` only because the case builder stored it without `_xlfn.`): `=PERCENTOF(A1:A3,C1)` and `=PERCENTOF({0.1,0.2,-0.3},1)` are 0, and `=PERCENTOF(A1:A2,A1:A3)` and `=PERCENTOF(A1:A3,A1:A3)` are #DIV/0! (the total snaps to 0). The engine sums both arguments through `fn_sum`. 
Tasks: T-0110 (implementation), T-1131 (SUM forms), T-1336 (AGGREGATE/SUBTOTAL), T-1017 (grouped root), T-1137 (probe), T-1160 (name bodies), T-1164 (grouped and SUM name bodies), T-2714 (subnormal bound)

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
Also `ROUND(1.05*(0.0284+0.0046)-0.0284,4)` = 0.0063 though the sum is 0.006249999999999999:
probes/round-float-noise-carry A1–A2 (excel-graph, 2026-10-26, T-2634; the same value Excel
cached in upstream's old ROUND.xlsx `issues!A2`).
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

## FLOOR.PRECISE: a 16th-digit tie in the ratio cut goes away from zero (T-2652)
Rule: the CEILING/FLOOR family cuts `value/significance` to 15 significant digits before rounding, and a tie at the
16th digit goes away from zero, as INT's cut does (below), not to even. FLOOR, FLOOR.MATH, FLOOR.PRECISE, CEILING,
CEILING.MATH, CEILING.PRECISE and ISO.CEILING all agree: with significance 1, `FLOOR(-333333333333266.5,1)` =
-333333333333267 and `FLOOR(333333333333266.5,1)` = 333333333333267 (a tie-to-even cut gives ...266), `x.5` at ...265.5
gives ...266 in both signs and both directions, `±999999999999999.5` gives ±1E+15, and `±100000000000000.5` gives
±100000000000001. A typed 16-digit literal is cut on entry, so the gen-m5 cell builds the tie from exact arithmetic
(`AVERAGE(100,0.5,100-1E+15)`) and the probe stores exact halves.
Unlike INT, the family's cut has no 1E+15 boundary: `FLOOR(-1000000000000000.5,1)` = -1E+15 (INT gives -1000000000000001),
`FLOOR(1000000000000002.5,1)` = `CEILING(1000000000000002.5,1)` = 1E+15, and `FLOOR(±4503599627370495.5,1)` =
±4503599627370500 (INT gives 4503599627370495 and -4503599627370496); without the cut these would keep their integer digits
(probe cells B7-B14). Not probed: a significance other than 1 (every probe row has significance 1, as does the gen-m5 cell),
so "cuts the ratio" rather than "cuts the value" rests on the existing design.
Evidence: gen-m5/048f6b1a0fb9544b Calc!A257 (excel-graph, 2026-10-25, T-2648); probes/t2652-floor-ceiling-ties
(14 values x 7 functions, excel-graph, 2026-10-25, T-2652).
Status: VERIFIED
Implemented in: `snap_ratio` (ceiling_floor.rs) via `significant_15` and `decimal_to_f64`, ties away from zero (T-2652)
Tasks: T-2652

## INT cuts to 15 significant digits first (T-2171)
Rule: INT rounds down after the same 15-significant-digit cut as TRUNC and ROUNDDOWN, so binary noise just below an
integer does not drop a unit: `INT(1-2^-53)` = 1 (0.99999999999999989 reads 1.00000000000000), so does
`FLOOR.PRECISE(INT(1-2^-53))`, and INT of 999999876543311.88 (`(1.005+E11)+(1E+15-A8)`) is 999999876543312.
Implemented in `fn_int` (mathematical.rs, T-2171) via `round_decimal`: Down for a positive value, Up (away from zero) for a negative one. A value of 1E+15 or more is floored as it is (`INT(2^53+1)` = 2^53: gen-m5/d586a1e0c9896942 Calc!A64; a sum of 1000000000000006 stays 1000000000000006: gen-m5/1315170eef6c413e Calc!A332; the cut would drop integer digits).
Evidence (excel-graph, 2026-10-20, T-2114): gen-m5/0362d48f56108faa Calc!A364, gen-m5/043ee562f32eb901 Calc!A62,
gen-m5/1962de140dae0111 Calc!A352, gen-m5/38b1b8e51dbad326 Calc!A212, gen-m5/b0d597ec5af5fb92 Calc!A173.
T-2215 (excel-graph, 2026-10-22; probes/t2215-int-negatives-boundary, 63 cells, engine passes all): the cut is
round-to-nearest (half away from zero) to 15 significant digits, then floor, for negatives too: `INT(-(1-2^-53))` = -1,
`INT(-100000000000000.25)` = -100000000000000 (plain floor would be -100000000000001), `INT(-999999999999999.25)` =
-999999999999999, `INT(-999999999999999.5)` = -1E+15, and the tie `INT(100000000000000.5)` = 100000000000001. At
1E+15 and above there is no cut: `INT(-1000000000000000.1)` = -1000000000000001, `INT(1000000000000005.5)` =
1000000000000005, `INT(-4503599627370495.5)` = -4503599627370496, `INT(999999999999999.9)` = 1E+15 (still cut).
Status: VERIFIED (excel-graph, 5 + 63 cells); implemented (T-2171, confirmed by T-2215).

## ODD and EVEN above 2^53: last digits not explained (T-2173)
Observation: `ODD(9999999999999999)` (the literal reads 9999999999999990) is 9999999999999992; ours is 9999999999999988, from
`ceil((|x|-1)/2)*2+1` in doubles, where `|x|-1` already rounds down. `RADIANS(ODD(9999999999999999)*(-1-B11))` follows it.
`EVEN((0.1/E5)*SERIESSUM(-2.5,B10,3,0.1))` (gen-m5/38b1b8e51dbad326 Calc!A10) was not an EVEN miss: the EVEN input was an
ulp low through SERIESSUM's own power, and the cell passes since T-2163 raised with the `^` kernel. No formulation is fitted
yet for the ODD cells.
Fitting points (66 values 2^52 .. 1.8E+308, ODD and EVEN, excel-graph, T-2172): probes-numeric/trig-odd-even-sumxmy2-sweep Calc!F1:H66.
EVEN matches Excel on all 66 cells. ODD misses 11: every miss is an even double strictly above 2^53 and up to about 1E+16 (2^53+2,
2^53+6, ..., 9999999999999990 .. 1.000000000000001E+16, and -(2^53+2)), where Excel returns the next double away from zero (x+2) and
ours the one toward zero (e.g. G10 `ODD(9007199254740994)` is 9007199254740996, ours 9007199254740992; G30 `ODD(9999999999999998)` is
1E+16, an even result). Between 2^53 and at least 1.000000000000001E+16 (G36; where the band ends below 2^54 is unprobed) the rule is by residue: x = 2 (mod 4) gives x+2 (away from zero), x = 0 (mod 4) returns x unchanged, an even result
(G11, G13, G15, G17, G27, G29, G31, G33, G35, G63). `ODD(2^53)` and values from 2^54 up return the input unchanged (F9, F20:F25, F37:F64, F66).
`EVEN(6.223015277861143E+37)` with the stored double returns its input (H60); the gen-m5 ...144 came from a computed, 1-ulp-low input, not EVEN. T-2173 fits this.
Evidence (excel-graph, 2026-10-20): gen-m5/6b831f5e2a20bc34 Calc!A283, gen-m5/bbbd316c346ce645 Calc!A224 (gen-m5/38b1b8e51dbad326 Calc!A10 fixed by T-2163).
`ODD(EVEN(9999999999999999))` is 9999999999999992 as well (ours 9999999999999988): gen-m5/d88d12a62b726a0a Calc!A232 (excel-graph,
2026-10-22, T-2252 triage).
Status: OBSERVED; T-2173 (after the T-2172 oracle sweep).

## MROUND rounds the quotient up within 5E-15 of a half, and multiplies in binary (T-2201)
Rule: `MROUND(number, multiple)` (after the sign and zero checks) is
1. `q = |number / multiple|`, the double quotient (no x87 or decimal division);
2. `k = floor(q)`, plus 1 when the fractional part `q - k` (exact in doubles) is at least `0.5 - 5E-15`: an absolute
   tolerance on the fraction, not a 15-significant-digit cut of `q` (`fr >= 0.5 - 5e-15` in doubles, or equally the
   fraction rounded half up to 14 decimals reaching 0.5);
3. the result `k * |multiple|` as one binary multiply, with the sign of `number`; a zero result is +0 (rows 211 and 213,
   `MROUND(-0.49999999999999495,-1)`, are 0).
Consequences: `MROUND(0.3,0.2)` = 0.4 (q = 1.4999999999999998); `MROUND(1.15,0.1)` = 1.2000000000000002 (12*0.1);
`MROUND(0.5,0.2)` = 0.6000000000000001 and `MROUND(0.29,0.1)` = 0.30000000000000004 (binary products; the `(x)=0.6`
companions are TRUE only because `=` compares to 15 digits); `MROUND(1234.4999999999998,1)` = 1234 and
`MROUND(98765432.49999999,1)` = 98765432 (one ulp below the half is more than 5E-15 there), while
`MROUND(1.4999999999999956,1)` = 2 (20 ulp below, 4.4E-15). The cutoff: fraction 0.5 - 4.996E-15 rounds up,
0.5 - 5.0515E-15 down, the same at 0.5, 1.5, with multiple 2 and for negatives (rows 184-213). Before T-2167 ours was:
`(value / multiple).round() * multiple`, which misses every within-tolerance cell below a half.
Evidence: probes/t2201-mround-quotient-rounding (excel-graph, 2026-10-20, 213 cells: T-2167's near-half quotients and
negatives, exact quotients at k+0.5 +- 1..48 ulp for four magnitudes, 80 seeded pairs within 4 ulp of a half, product
forming with companions, 1E-15 and 1E+15 controls, and the cutoff sweep); generator docs/research/t2201/gen.py. The rule
predicts all 213 cells. Rivals that miss: a 15-digit cut of `q` (145 or 146 of 213, by cut variant: misses 1234.5 - 1 ulp and most seeded pairs), a
tolerance of 4.5E-15 or 4.9E-15 (196, 207) or 5.3E-15 (203), an x87 quotient, and a decimal product (the products are
binary). gen-m5/043ee562f32eb901 Calc!A199 is `MROUND(0.3,0.2)`.
Status: VERIFIED (excel-graph, 213 cells); implemented (T-2167, `mround_value` in ceiling_floor.rs).

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
2026-10-09, T-0268); ironcalc-calc MATH_AND_TRIGONOMETRY/ROUND_ROUNDUP_ROUNDDOWN.xlsx D6 (desktop Excel cached value).
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
(probes-numeric/4f413cc257ce15c4 A23, A51, A79, A71, A7; T-0980). Same in the integer part
(gen-m5/1962de140dae0111 Calc!A267: `2.5E+29` → "250000000000000000000000000000"; gen-m5/25768de48ae7f2f9 Calc!A99:
`-909090909090902.6` → "(909,090,909,090,903.00)").
Implemented in: `get_fract_part` and the `ParsePart::Number` branch of `format_number` (formatter/format.rs), T-0980.
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
ACOT shows no negative zero either: `ACOT(CEILING.PRECISE(Data!C9))` with C9 = -0.0001 is 1.5707963267948966 (PI()/2), the
value of `ACOT(0)` (gen-m5/6efa59fc1257c475 Calc!A7, excel-graph, 2026-10-22, T-2252 triage). Before T-2376 ours was -1.5707963267948966:
CEILING.PRECISE (and CEILING, CEILING.MATH, ISO.CEILING, ECMA.CEILING, unary minus `-(0)` and `INT(-0)`) yields the double -0.0,
and `fn_acot` tested `f >= 0.0` (true for -0.0) and returned `atan(1/-0.0)` = -PI()/2. Implementation: the zero branch of `fn_acot` (T-2376). Whether Excel's CEILING.PRECISE yields
+0 or ACOT ignores the sign of zero cannot be told from the cell; both give PI()/2.

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
COT, CSC and SEC are the same division of 1 by the rounded TAN, SIN or COS: `COT(PI()/4)` is 1 like `1/TAN(PI()/4)`
(TAN is 0.9999999999999999, whose correctly rounded reciprocal is 1.0000000000000002), `CSC(PI()/6)` and `CSC(5*PI()/6)`
are 2 (SIN 0.49999999999999994), and `SEC(PI()/3)` is 1.9999999999999996 (probes/trig-reciprocal-special-angles, 14
cells, excel-graph, 2026-10-20, and Excel desktop's cached ironcalc-calc TRIGONOMETRIC.xlsx COT_CSC_SEC B47, C48; T-0915).
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
`arithmetic::extended_sqrt` (T-1500, A65-A80 pass). `x^-0.5`, `POWER(x,-0.5)` and `1/SQRT(x)` are the
same: the root rounded twice (as SQRT), then one over it divided like `/` (rounded twice). Neither the
correctly rounded root nor the 64-bit root is used: over operands where the three readings give three
different doubles, Excel gives the twice-rounded one each time (`((1+314268894768158*2^-52))^-0.5` is
0.9668351146104247, where the plain root gives …245 and the 64-bit root …246). A subnormal radicand
(kept by `-`, `(x)=0` FALSE) rounds its root twice too: `SQRT((1+344225268385227*2^-52)*2^-1022-2^-1022)`
is 4.123952952442882E-155 (the correctly rounded root is …881E-155), and its `^-0.5` follows the same
rule. Evidence: probes-numeric/power-neg-half-rounding (40 cells; operands of at most 15 significant
digits, since a 16-digit literal loses its last digit; excel-graph, 2026-10-14, T-1541). Implemented in
`arithmetic::non_integer_power` and `extended_sqrt`, which normalizes a subnormal's significand.
AVERAGE's final division of the sum by the count rounds twice too, like `/`
(probes-numeric/average-large-count-division, 24 cells: one 15-digit value S and n-1 zeros, n = 2051,
2999, 4095, S picked by exact simulation so that once- and twice-rounded S/n differ; every cell is the
twice-rounded value; oracle: excel-graph, 2026-10-15, T-1619). For a 53-bit sum no count n <= 2048 can
discriminate (the remainder fraction cannot come within half an extended ulp of a midpoint), so the
count of 1025-2048 the task guessed is moot; counts 3001, 3333, 4001 gave no pick in our search.
Implemented in `fn_average`, `fn_averagea`, the AVERAGEIFS closure and `subtotal_average` with `arithmetic::extended_quotient`.
The same holds for AVERAGEA, AVERAGEIF, AVERAGEIFS, SUBTOTAL(1) and SUBTOTAL(101) (AGGREGATE(1) routes through AVERAGE): probes-numeric/average-family-large-count-division, 36 cells (the T-1619 columns at n = 2051, 2999, 4095, two each; excel-graph, 2026-10-15, T-1636): Excel gives the twice-rounded quotient for all of them. Ours divided once in AVERAGEA, AVERAGEIF(S) and SUBTOTAL (30 of 36 cells off by 1 ulp); fixed in T-1636. DAVERAGE rounds twice too (probes-numeric/daverage-large-count-division, 6 cells, the same columns with a header row and criterion `>=0`; excel-graph, 2026-10-15, T-1648: all 6 are the twice-rounded value). Ours divided once; fixed with `extended_quotient` in `fn_daverage`.
Accumulators beyond SUM and AVERAGE (probes-numeric/double-rounding-accumulators, 56 cells; 8 operand
sets a, b, c of at most 15 significant digits picked by exact simulation so that a+b is the same under
single and double rounding but (a+b)+c is not; oracle: excel-graph, 2026-10-15, T-1516): `a+b+c`,
`SUM(range)`, `SUM(a,b,c)`, `SUMIF`, `SUMIFS`, `DSUM` and `AVERAGEA(range)*4` (a text cell counts as 0, so
the division is exact) all give the value whose second addition was rounded twice, in every set: the
total is rounded to a double after every addition (to 64 bits, then to a double, as `extended_sum` does); it is not carried at 64 bits between additions. Sets 4, 5 and 7 (numbered from 0; Data!A17:A19, A21:A23, A29:A31) tell the two apart, and the oracle matches the per-addition rounding in all of them. SUMIF,
SUMIFS, DSUM and AVERAGEA used the plain `+` and missed by one ulp in all 8 sets (32 cells); they add
with `extended_sum` since T-1618 and all 104 cells pass. DAVERAGE accumulates the same way (probes-numeric/daverage-accumulation A1:A8, DAVERAGE over a, b, c and a 0 record, times 4; excel-graph, 2026-10-15, T-1649): the engine's plain `+` missed by one ulp in all 8 sets and `extended_sum` fixes it. A subnormal quotient is flushed to 0, as AVERAGE's is (A20:A24: 2^-1022 with a 0 record is 0, not 2^-1023; with three 0 records 0; 2^-1021 over three records 0), while a quotient of exactly 2^-1022 is kept; the engine kept the subnormal until `fn_daverage` applied `flush_subnormal`. DAVERAGE over kept subnormal operands built by subtraction (`3E-308-2.5E-308`, once or three times; A26, A28) or over normal operands whose sum is subnormal (`3E-308` with `0-2.9E-308`, A27; `(x)=0` companion E26 FALSE) is 0 too, like AVERAGE's (a subnormal literal such as `1E-310` reads as 0 in Excel itself, so it cannot serve as an operand). AVERAGEIFS, AVERAGEIF and AVEDEV accumulate the same way (probes-numeric/double-rounding-averageifs, 56 cells: the 8 operand sets with a 0 fourth record, times 4; excel-graph, 2026-10-16, T-1650): the engine's plain `+` missed by one ulp in 21 cells; AVEDEV's mean sum rounds twice (5 of the 21 misses); its sum of absolute deviations rounds twice as well (probes-numeric/double-rounding-avedev-deviations, 8 symmetric sets +-D, +-e whose deviations D, D, e, e discriminate; excel-graph, 2026-10-16, T-1665: AVEDEV*4 equals the twice-rounded `SUMPRODUCT(ABS())` control in all 8; the engine's plain `+` missed by one ulp in all 8, fixed with `extended_sum`; SUMPRODUCT's running total rounds twice too and uses `extended_sum` since T-1676, so all 56 cells of the probe pass; multi-array SUMPRODUCT: probes-numeric/double-rounding-sumproduct-multi, 110 cells, excel-graph, 2026-10-16, T-1697: each per-term product rounds twice like `*` - two arrays equal `a*b` (A1-A8 tell single from double rounding apart), three arrays equal `a*b*c` (A9-A14 show rounding at every step but do not separate single from double rounding; the twice-rounded step is inferred from `*`) - and the running total of 2- and 3-array SUMPRODUCT rounds twice like SUM over the 8 accumulator sets; the engine's plain products missed by one ulp in all 8 two-array pairs and use `extended_product` since T-1697). All 56 cells of the averageifs probe pass with `extended_sum` on the AVERAGEIF(S) total and the AVEDEV mean sum. AVERAGE's division by a small count (3, 5, 7) was not probed: that is reasoning, not oracle evidence. For a
small odd count n the binary expansion of k/n has no run of 11 equal bits after the cut, so the 64-bit rounding
never lands on a midpoint and double rounding would equal single rounding. A discriminating divisor needs n above
2^11 (an earlier random search at n = 1025 found nothing; T-1619 probed counts 2051-4095, below).
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

## Desktop goldens round `*` once (calipers formulas_nested)
Rule (desktop only, observed): the desktop Excel that saved `calipers/roundtrip/formulas_nested/golden.xlsx`
rounds the worksheet `*` once (correctly rounded), not twice as excel-graph does ("Division rounds twice"
above). Over all 5,518 `x*constant` cells of the workbook, simulated exactly from Excel's cached operands,
the singly rounded product matches the cached value in 5,518 cells and the twice-rounded product in 5,514.
The four cells that tell them apart all hold the singly rounded product:
- `Extreme Depth!E1834` `=E1833*1.001` (E1833 49.81856401224343) is 49.86838257625566, where twice
  rounding gives 49.86838257625567;
- `Extreme Depth!E4446` `=E4445*1.001` (E4445 10.161093439757583) is 10.17125453319734, where twice
  rounding gives 10.171254533197338;
- `XSheet Chain A!E919` and `E2225` are the same two products, read across sheets.
The sibling roundtrip goldens are Mac saves (T-2663), and a Mac build on ARM has no x87 double rounding.
Excel for the web stays the target (decision 0039 item 4), so the engine keeps rounding twice. These four
products, and the 296 chain cells after them, are the workbook's 300 calipers known failures.
Evidence: calipers/roundtrip/formulas_nested/golden.xlsx cached values (desktop, gap triage 2026-10-25).
Status: UNVERIFIED (one desktop golden; Windows desktop unprobed: T-2704)
Tasks: T-2704

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

## Stored numbers in an xlsx are not always parsed to the nearest double (T-1778)

When Excel loads a cell value from xlsx (`<v>` text), it occasionally lands one ulp away from the correctly rounded
double. In the T-1778 ERFC/EXP probes this happened at 4 of about 2.1M input cells. For each, Excel's EXP and ERFC
results are bit-identical to the neighbouring double's model value and 10-400 ulp from the written value's:

| `<v>` text | nearest double | double Excel used |
|---|---|---|
| `11.137073` | 0x1.6462e6ea85447p+3 | ...448 |
| `15.803795365599` | 0x1.f9b8b10efd91fp+3 | ...920 |

The other two cells are in `kmyst-erfc-r6clow7` (x ≈ 2.45) and `kmyst-erfc-r6clow8` (x ≈ 3.09). Their written texts
are the shortest round-trip decimals, and Excel used the double one below.

So the parse is not a correctly rounded decimal-to-binary conversion, even for short texts. The rule behind the
rare misreads is not derived. Probe cases that need an exact input double should check Excel's own reading, for
example by returning `=A1` or by testing the model at the neighbouring doubles.

Later rounds found 3 more, at x = 2.2677103947185002, 2.6493240961407998 and 1.3246620480703999; the two x ≈ 2.45
and 3.09 cells above are 2.4500000000004998 and 3.0900000000000998 (statistics.md, T-1778). The curated T-1916
probes leave all seven inputs out.

Evidence (probes-numeric on branch `agent/research/T-1778-kernel-map`): `kmyst-erfc-r4bisect2`, `kmyst-erfc-r6bisN2`, `kmyst-erfc-r6clow7`, `kmyst-erfc-r6clow8` (excel-graph,
2026-10-18). Status: OBSERVED; the parsing rule is OPEN.

## EXP is an x87 2^(a*log2 e) evaluation, not correctly rounded (T-1778)

Excel's EXP(a) behaves like x87 code written for the 8087's F2XM1 domain (model "V6", T-1778 round 8):
1. t = RN64(a * RN64(log2 e)), that is FLDL2E then FMUL in 64-bit precision.
2. n = nint(t), using FRNDINT with ties to even.
3. f = t - n, which is exact, with |f| <= 1/2.
4. u = F2XM1(|f|), modelled as RN64(2^|f| - 1).
5. w = RN64(1 + u).
6. If f < 0, w = RN64(1/w).
7. EXP = RN53(w * 2^n), with gradual underflow.

This matches 2,139,591 of 2,139,684 EXP cells across all kmyst-erfc-* probes, including the near-midpoint maps
`kmyst-erfc-r5expmap1/2`. The 93 misses are all points where 2^|f| - 1 lies within 0.09 ulp of a 64-bit rounding
midpoint. There, Excel's F2XM1 is faithful but not correctly rounded, with an error of about 2^-69, deterministic
and not matched by this machine's Intel F2XM1. The likely cause is a different CPU's F2XM1 microcode (an AMD Zen
hypothesis is untested), so those cells may be CPU-dependent. Confirmed in part (T-1916): a fresh excel-graph fill of the
20 ERFC cells whose E is such a near-tie returned 9 of them with the other rounding (statistics.md, the T-1778
entry), while 37,181 neighbouring ERF/ERFC cells were identical across the two runs. The earlier description below (a floor split, with
F2XM1 applied to f in [0, 1)) is superseded: it had 6,281 misses on the same set.

Earlier description: t = fl64(a * L2E64), then EXP = fl53(fl64(2^t)). Here L2E64 is log2(e) rounded to a 64-bit
mantissa, with relative error +1.92E-20.
- **Where it matches.** EXP(-(x*x)) at 121,790 of 121,805 points (x from 0.5 to 26.5) and EXP(-z) at 30,717 of
  30,720 (z from 0.26 to 702). A variant that applies F2XM1 to the fraction (n = nint(t),
  w = fl64(1 + fl64(2^(t-n) - 1)), EXP = fl53(w * 2^n)) gains 3 more points.
- **What correctly rounded exp gives.** 30,218 of 30,720.
- **How it shows.** Excel's EXP is 1 ulp off the correctly rounded value at about 2.1% of points with large
  |a|, mostly low. The bias grows with |a|, as the error in L2E64 times a predicts. At small |a| the two agree,
  which is why the `exp-ln` sweep above saw correctly rounded values.
- **Where it matters.** ERFC.PRECISE for x >= 0.5 multiplies by this stored EXP value (statistics.md, T-1778).
- **Open.** The remaining 12 to 15 of 152,525 mismatches are unresolved.

Evidence: probes-numeric (on branch `agent/research/T-1778-kernel-map`) `kmyst-erfc-expcol`, `kmyst-erfc-runs1..4`, `kmyst-erfc-stepmap1/2`, `kmyst-erfc-chiruns1..3`
(excel-graph, 2026-10-18). Status: VERIFIED (structure); the last mismatches are OPEN.
Ours: `crates/rustsheet-engine/src/x87_exp.rs` `excel_exp` implements V6 with integer x87 rounding and a double-double
F2XM1 (T-1916). It reproduces the reference model, scoring 62,013 of 62,100 `EXP(A)` cells of `kmyst-erfc-r5expmap1/2`
and every `EXP(-(x*x))` cell up to x = 26.55 except the near-ties. The ERFC kernel and, since T-1931, the worksheet EXP
use it.

Worksheet EXP (T-1931). `EXP(a)` in a cell is model V6 exactly, with no extra step: a subnormal result is kept, not
flushed (probes/subnormal-function-results C4 and C6, `EXP(-745)=0` and `EXP(-720)=0` are FALSE), and an overflow is
`#NUM!` as before. The `kmyst-erfc-cutoff` column C reads 0 for the 18 cells from x = 26.616 where V6 gives a
subnormal (2.19E-308 down to 3.58E-309), but that is the Graph artifact of decision 0042 (Graph reports a kept
subnormal as 0.0), not a flush, so the curated subset leaves those cells out.
The artifact is Excel's own save (T-2204, decision 0089; probes/subnormal-saved-as-zero, excel-graph 2026-10-20): the xlsx
Excel for the web saves holds `<v>0</v>` for `EXP(-745)`, `EXP(-720)` and `=A1` of such a cell, while `(EXP(-745))=0` is
FALSE and `EXP(-745)*2^1000` is exactly 2^-74 (`<v>5.2939559203393771E-23</v>`), so the kept value is 2^-1074. The oracle
reads its values from that file, so every value Excel for the web saves for a kept subnormal is 0, cached values
included: ironcalc-calc `EXP_SIGN.xlsx` EXP_SIGN!B44 (`EXP(-745)`, Excel Online) caches 0 for the same reason. A normal
result is written in full (`EXP(-708)`: `<v>3.3075530036384078E-308</v>`). Desktop Excel's save is not probed.
RustSheet's export does not do this yet (T-2211).
Range edges (T-2203, probes-numeric/t2203-exp-range-edges, excel-graph 2026-10-22, 527 rows): the last finite
argument is 709.782712893384 (ln DBL_MAX; EXP = 1.7976931348622732E308, not quite DBL_MAX), and the first
`#NUM!` is its successor double 709.7827128933841; every argument above it, up to 710.5, is `#NUM!`. At the other
end `(EXP(a))=0` is TRUE for every a <= -745.1332191019412 and FALSE for every a >= -745.1332191019411 (the kept
2^-1074, probes/subnormal-function-results C4), so the switch sits between those two consecutive doubles, at -1075*ln 2 as the midpoint rounding
predicts; -745.5 and -746 are 0. The 20 control arguments in [-708.39, -707] are normal results. The current
engine matches all 780 checked cells.
- Curated evidence on main (probes-numeric, excel-graph fill of 2026-10-19, bit-identical to the T-1778 research
  run at all 22,281 formula cells): `t1931-exp-map` (5,990 near-midpoint EXP(a) cells from `kmyst-erfc-r5expmap1`),
  `t1931-exp-random` (7,084 from `kmyst-erfc-r5expmap2`), `t1931-exp-gauss` (8,096 `EXP(-(x*x))` cells from
  `kmyst-erfc-expcol` and `kmyst-erfc-driftmap`), `t1931-exp-neg` (1,024 `EXP(-z)` cells from `kmyst-erfc-chiruns1`)
  and `t1931-exp-ties` (87). The engine matches every cell of the first four; libm's correctly rounded `exp`, which the
  worksheet EXP used before, misses 6,011 of those 22,194 (most inputs were chosen near a rounding midpoint):
  2,596 map, 3,158 random, 256 gauss and 1 neg cells.
- Known residuals (`t1931-exp-ties`, owner T-1940): all 87 V6 misses of `kmyst-erfc-r5expmap1/2` (33 and 54). Each
  is a 1 ulp miss (engine minus Excel: +1 ulp at 47 cells, -1 ulp at 40), at an F2XM1 near-tie. libm happens to match 43 of
  them (a correctly rounded F2XM1 and a correctly rounded exp can agree), so these 43 cells pass under libm and fail
  under V6; the switch still gains 5,968 cells. Unlike the 20 ERFC near-tie cells of T-1916 (statistics.md), all 87
  came back identical in the 2026-10-19 re-fill.
- POWER and `^` with a non-integer exponent use `excel_exp` since T-0959 (see "Non-integer exponents: exp(y·ln x)").
- Other EXP users stay on libm's `exp`: with SINH, COSH, TANH and FISHERINV switched to `excel_exp` as an experiment, no
  cell of probes, probes-numeric (`m9-hyperbolic`), ironcalc-calc (`TRIGONOMETRIC.xlsx`), poi, fixtures or gen-m9
  changed verdict (the probed arguments are small, where V6 and libm agree), so the evidence cannot tell the two apart
  and the switch was not made. A probe of SINH/COSH at large |x| near an EXP rounding midpoint would decide it.

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
`C` companion cells `(x)=0` tell a true 0 from a kept subnormal, since the saved file, and so the Graph value,
holds 0 (decision 0089) and
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
FALSE excel-graph companion `(x)=0` for the same formula, since Graph reports both as 0. A cached 0 in an xlsx
suite takes the same rule through a `cached_overrides` line naming a probe cell of the same formula on the same
inputs whose own companion is FALSE (decision 0090; EXP_SIGN!B44 against probes/subnormal-saved-as-zero S!A1).
Implemented in: `arithmetic::plus`/`minus` keep subnormals (T-1027); number text below the smallest
normal reads as 0 (`Model::parse_number_text`; VERIFIED for the strings `1e-308` and `1e-309` only, T-1143: the `(x)=0` companions of `("1e-308"+0)`, `("1e-308"-0)`, `VALUE("1e-309")` and `VALUE("1e-308")` are all TRUE, probes/subnormal-text-and-tail-companions); `arithmetic::finite_or_num` flushes `*`, `/`, `^` and POWER; `arithmetic::flush_subnormal`
flushes the results of PRODUCT and SUMPRODUCT (each running product, T-1007), SUMSQ (each square, T-1057), AVERAGE, AVERAGEA, AVERAGEIF(S) and
SUBTOTAL 1/101 (final result; T-0987, T-1007), and of the postfix `%` (T-0988).
Distribution densities (T-2693, VERIFIED at the probed arguments, excel-graph 2026-10-25, companions added to the
probes-numeric cases below and probes/t2693-subnormal-*). Excel KEEPS the subnormal (companion FALSE) of `EXP(-720)`,
`EXP(-725)`, `EXP(-730)` and `EXP(-740)` (t2586-r5-f-edges-a A927:A932, -hold A293:A295, t2586-verify-r5-f-edges-hold
A55-A71), of `BINOM.DIST(0,999,p,FALSE)` (the k = 0 pmf; t2513-verify-r3-binom-pmf-h A229, A238) and of a LOGINV
(gen-m9/899f9555e67da2d6 A386). It returns a TRUE 0 (companion TRUE) from `BINOM.DIST(4000000,10000000,p,FALSE)`
(t2513-r3-binom-pmf-n B1729:B1792), `NEGBINOM.DIST(999,1,p,FALSE)` (t2513-verify-r3-binom-pmf-h B229, B238), the
BETA.DIST density (t2513-r3-m1-residual-hold D863, D875, D957), `GAMMA.DIST(50000000,500,1000000,FALSE)`
(m9-gamma-dist-a500 A22), an EXPON.DIST (gen-m9/165db58d1540f014 A65) and a WEIBULL (gen-m9/eccd03df42e40006 A96).
The flush follows each function's internal path, not the size of the result: `BINOM.DIST(0,999,p,FALSE)` and
`NEGBINOM.DIST(999,1,p,FALSE)` differ only by a factor of p, yet Excel keeps the first and zeroes the second (I229/I238
FALSE, J229/J238 TRUE). Implemented (T-2703): the EXPON.DIST, WEIBULL, BETA.DIST and GAMMA.DIST densities, the
BINOM.DIST pmf off its k = 0 and k = n closed forms, and the NEGBINOM.DIST pmf pass their result through
`arithmetic::flush_subnormal`. The closed forms and the cumulative modes are left unchanged: k = n is EXP-based like k = 0.
More paths (T-2706, VERIFIED at the probed arguments, probes/t2706-subnormal-density-paths, excel-graph 2026-10-25, each
true value between 4.9E-324 and 2.2E-308, row n is the call in An and its companion in Bn). Excel KEEPS the subnormal
(companion FALSE) of the BINOM.DIST pmf at k = n (`BINOM.DIST(1000,1000,0.49,FALSE)`, BINOMDIST, `BINOM.DIST(500,500,
0.24,FALSE)`; rows 1-3), the BINOM.DIST cdf at k = 0 (row 4), the NEGBINOM.DIST cdf at f = 0 (row 8), the NEGBINOM.DIST
pmf at f = 0, p^s (`NEGBINOM.DIST(0,1000,0.49,FALSE)`, NEGBINOMDIST; rows 12-13), the BETA.DIST cdf at shapes (2, 1)
(`BETA.DIST(1E-155,2,1,TRUE)`, BETADIST; rows 14, 16) and the WEIBULL.DIST cdf (`WEIBULL.DIST(1E-155,2,1,TRUE)`, row
34). It returns a TRUE 0 (companion TRUE) from the BINOM.DIST cdf off k = 0 (`BINOM.DIST(1,1000,0.52,TRUE)`,
`BINOM.DIST(3,1000,0.53,TRUE)`, BINOMDIST; rows 5-7), the NEGBINOM.DIST cdf off f = 0 (row 9), the NEGBINOM.DIST pmf
at `NEGBINOM.DIST(999,1,0.51,FALSE)` and NEGBINOMDIST (rows 10-11), the BETA.DIST cdf at shapes (10, 2)
(`BETA.DIST(1E-31,10,2,TRUE)`, row 15), the BETA.DIST density both below and past Cephes' MAXGAM (`BETA.DIST(4E-7,50,2,
FALSE)`, `BETA.DIST(0.00018,100,100,FALSE)`; rows 17-18), the GAMMA.DIST cdf (`GAMMA.DIST(1E-155,2,1,TRUE)`, GAMMADIST;
rows 19-20), the GAMMA.DIST density at shapes 1 and 2 (`GAMMA.DIST(712,1,1,FALSE)`, `GAMMA.DIST(718,2,1,FALSE)`,
GAMMADIST; rows 21-24), the EXPON.DIST density and cdf (`EXPON.DIST(712,1,FALSE)`, EXPONDIST, `EXPON.DIST(356,2,FALSE)`,
`EXPON.DIST(1E-150,1E-160,TRUE)`; rows 26-29), the WEIBULL.DIST density at shapes 1 and 2 under both names (rows 30-33)
and the CHISQ.DIST density (`CHISQ.DIST(1430,2,FALSE)`, row 35). So the closed forms at the ends keep it (k = n and
k = 0, f = 0, p^s) and so do the BETA.DIST (2, 1) cdf, x^2, and the WEIBULL.DIST cdf, while the EXPON.DIST cdf of the
same 1E-310 is 0. `GAMMA.DIST(1E-10,1,1E+307*10,FALSE)` is #NUM! (row 25; T-2738).
The scale is `1E+307*10` because a typed `1E+308` is refused (Graph 400) and a stored one makes Excel refuse the
workbook (createSession 501 FileCorruptTryRepair; both excel-graph 2026-10-25, scratch fills), as for the tiny literals
below.
The BETA.DIST cdf keeps it exactly at beta = 1 (T-2737, probes/t2737-subnormal-cdf-shapes, excel-graph 2026-10-25,
same layout): kept at shapes (3, 1), (10, 1), (1.5, 1), (2, 1) at another x, (2, 1) with bounds 0, 1 and (5, 1)
(rows 2, 3, 6, 7, 11, 12), 0 at (2, 2), (2, 1.5), (2, 3), (2, 1.0000001), (10, 2) at another x and BETADIST at
(10, 2) (rows 1, 4, 5, 8-10). GAMMA.DIST's cdf at shape 5 and at shape 2 with scale 2, CHISQ.DIST's cdf at 4 degrees
and its densities at 4 and 6 degrees are 0 too (rows 13-17). Implemented (T-2737): those cdfs, the BINOM.DIST and
NEGBINOM.DIST cdfs off their closed forms, the EXPON.DIST cdf and the CHISQ.DIST density pass through
`arithmetic::flush_subnormal`, the BETA.DIST cdf does unless beta = 1, and the NEGBINOM.DIST pmf at f = 0 no longer
does (test/statistical/test_subnormal_results.rs). Unprobed and flushed by extension: the CHISQ.DIST cdf at 2 degrees
(`1 - EXP(-x/2)`, subnormal from x below 4.4E-308) and at 1 degree (an ERF form, never subnormal from a normal x), the
BETA.DIST cdf at alpha = 1, and BETADIST at beta = 1 off (2, 1). Row 25's #NUM! is GAMMA.DIST's subnormal x / beta
(statistics.md "GAMMA.DIST's density is #NUM! at a subnormal x / beta", T-2738).
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
and the engine is a few ulp off. T-1778 found the tail form that reproduces it (statistics.md, "ERFC core
factor"); T-1916 implements it and closes T-1186. Swapping `normal_pdf`'s constant for `(2*pi).sqrt()` regresses NORM_DIST.xlsx NORM.INV!E13 and E17, so NORM.DIST keeps its order.
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
A subnormal cell constant reads as 0 (T-2016, T-2228, VERIFIED, probes/subnormal-cell-constants, excel-graph 2026-10-21): a
number constant of magnitude below 2.2250738585072014E-308 (5E-324, 2.2E-308, 1E-310, and the negatives -5E-324, -1E-310) in a loaded workbook (an xlsx `<v>`; typing
one in is unprobed) is `=0` TRUE and `ISNUMBER` TRUE, `*1E+300`
gives 0, and `LOG10` and `LN` of it are `#NUM!`; the smallest normal is kept. So LOG10 and LN never see a subnormal
argument from a cell. Implemented at load (T-2228): xlsx import flushes a `<v>` number constant with
`arithmetic::flush_subnormal`, and so does the eval harness for a spec case's number (it stands for the same xlsx load). A
spill's cached value and any formula result keep their subnormal; a constant set through the API is kept too (typing one is
unprobed).
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

## MOD of a small number by a huge opposite-sign divisor is 0 (T-2654)
Rule: when the signs differ and the shifted result `fmod(n,d) + d` rounds (in doubles) back to `d` itself, Excel's MOD
is 0, not `d`. The test is that one rounded sum, not a quotient or relative threshold: `MOD(1,-(2^53))` =
-9007199254740991 but `MOD(1,-(2^54))` = 0; `MOD(0.5,-(2^52))` = -4503599627370495.5 but `MOD(0.5,-(2^53))` = 0 (the
tie rounds to even, which is `d`) while `MOD(0.75,-(2^53))` = -9007199254740991; `MOD(1.005,-1E+16)` =
-9999999999999998 but `MOD(1.005,-1E+17)` = 0; `MOD(0.5,-5E+15)` = 0 but `MOD(0.5,-2E+15)` = -1999999999999999.5;
`MOD(1,-1E+30)`, `MOD(1,-9.99999999999999E+307)` and the mirrored `MOD(-1,1E+30)` are 0. Same-sign controls keep the
remainder (`MOD(1,2^60)` = 1, `MOD(-0.5,-(2^54))` = -0.5). Over all 168 opposite-sign cells, the cells that differ from
"fmod plus d" are exactly those where the sum equals `d`.
In passing: `MOD(0.75,-(2^53))+2^53` is 0, not 1 (-9007199254740991 + 2^53; see "Near-zero snapping in addition and subtraction").
Evidence: probes/mod-huge-opposite-divisor (183 cells, excel-graph, 2026-10-25); gen-m5/abef960804539149 Calc!A400
(T-2648).
Status: VERIFIED
Implemented in: `excel_mod` in crates/rustsheet-engine/src/functions/util.rs
Tasks: T-2654

## MOD near the smallest normal (T-2714)
Rule: with f = fmod(n, d) (the exact remainder) and D = 2^-1022 (the smallest normal double), a remainder that passes
below D is `#NUM!`:
- same signs: f subnormal is `#NUM!`, else f;
- opposite signs, s = f + d: f or s subnormal (but not both) is `#NUM!`; both subnormal gives `d - f` when |d| is exactly D,
  and `#NUM!` for any other d; neither subnormal keeps s (0 where s rounds to d, "MOD of a small number by a huge
  opposite-sign divisor is 0");
- f = 0 is 0, and the huge-quotient `#NUM!` and the `#DIV/0!` of d = 0 come first.
For example `MOD(10^-307,-10^-307)` and `MOD(-10^-307,10^-307)` are `#NUM!` (f is 12·2^-1074 in the first, s in the
second), as are `MOD(1E-300*(1+2^-52),1E-300)` and `MOD(1E-300,-1E-300*(1+2^-52))`, while
`MOD(-(2^-1021*(1+2^-52)),2^-1022)` is 2^-1022 + 2^-1073 (f = -2^-1073, s = 2^-1022 - 2^-1073; the result mirrors s
about D) and `MOD(-(2^-1022*(1+4*2^-52)),2^-1022*(1+2^-52))` is `#NUM!`. A normal shift is untouched:
`MOD(1E-290,-1E-290*(1+2^-52))` is -2.848094538889218E-306. The `d - f` arm holds from |n| = |d|·(1+2^-52) to
|n| = 2^22·|d| (probe -c). No mechanism is known; the rule is fitted. This settles the calipers desktop goldens
`formulas_math_basic` and `formulas_datetime` I88 and H89 (`#NUM!`). Unprobed: a subnormal f whose shift rounds back
to d (cell constants are flushed on load, so only a formula's subnormal can reach MOD).
Evidence: probes/t2714-mod-near-smallest-normal (131 pairs in both argument orders), -b (70) and -c (66 pairs in both orders, the both-subnormal
arm), excel-graph, 2026-10-25: all 464 MOD cells match the rule, with `ISERROR` and `(x)=0` companions.
Status: VERIFIED (excel-graph) as a fitted rule
Implemented in: `excel_mod` in crates/rustsheet-engine/src/functions/util.rs
Tasks: T-2714

## CEILING and FLOOR of a subnormal ratio are 0 (T-2715)
Rule: CEILING, CEILING.MATH (both modes), CEILING.PRECISE, ISO.CEILING, ECMA.CEILING, FLOOR, FLOOR.MATH (both modes) and
FLOOR.PRECISE flush the ratio number/significance to 0 when it is subnormal (|ratio| < 2^-1022), as `/` does ("Subnormal
results become 0"), so the result is 0. A ratio at 2^-1022 keeps its step. `CEILING.MATH(10^-307,10)` and
`FLOOR.MATH(-10^-307,100)` are 0, while `CEILING.MATH(1E-307,4)` (ratio 2.5E-308) is 4, `CEILING(2^-1022,1)` is 1, `FLOOR.MATH(-(2^-1022),1)` is -1
and `CEILING(2^-1022,1.0000001)` is 0. (Write a negative input as `-(2^-1022)`: `-2^-1022` is `(-2)^-1022`, positive.) The sign and argument checks are unchanged (`CEILING(1E-307,-10)` is still `#NUM!`).
MROUND already matched (its quotient rounds to 0 there). ROUNDUP does not follow this rule: `ROUNDUP(1E-307,-2)` is 100,
but `ROUNDUP(2^-1022,0)` is a subnormal ("ROUNDUP and ROUNDDOWN of a number whose 15-digit cut is subnormal", T-2756). This settles the calipers goldens `formulas_math_basic` and
`formulas_datetime` E169, F169, E183 and F183.
Evidence: probes/t2715-ceiling-floor-subnormal-ratio (excel-graph, 2026-10-25): 76 number/significance pairs (±1E-307,
±2^-1022, ±1E-300 against significances above, at and below the boundary, both signs, and the calipers pairs) times 11
functions, all matching the rule; probes/t2715-roundup-near-smallest-normal (the ROUNDUP and ROUNDDOWN controls).
Status: VERIFIED (excel-graph)
Implemented in: `snap_ratio` in crates/rustsheet-engine/src/functions/math_and_trigonometry/ceiling_floor.rs
Tasks: T-2715, T-2756 (ROUNDUP)

## ROUNDUP and ROUNDDOWN of a number whose 15-digit cut is subnormal (T-2756)
Rule: the ROUND family first cuts x to 15 significant digits (round to nearest, "ROUND family rounds in two decimal
stages"). When that cut, as a double, is subnormal (below 2^-1022), ROUNDUP, ROUNDDOWN and TRUNC return a nonzero
subnormal whatever num_digits, so `ROUNDUP(2^-1022,0)`, `ROUNDUP(-(2^-1022),-2)`, `ROUNDUP(2^-1022,300)` and
`ROUNDDOWN(2^-1022,307)` all show 0 but `(x)=0` is FALSE. This holds for x = 2^-1022·(1+k·2^-52) with k = 0..7 (x up to
2.2250738585072048E-308, cut 2.22507385850720E-308); from k = 8 (cut 2.22507385850721E-308, normal) ROUNDUP takes its step
again (`ROUNDUP(x,0)` 1, `ROUNDUP(-x,-2)` -100, `ROUNDUP(x,2)` 0.01, `ROUNDUP(x,300)` 1E-300), so the threshold is the
round-to-nearest cut, not a truncation (which would put it at k = 18). Otherwise every decimal the family converts to a
double is flushed to 0 when subnormal: ROUND's result at any k (`ROUND(2^-1022,309)` 2.2E-308 is 0, and so is `ROUND(x,330)`
with nothing dropped for k ≤ 7, whose kept cut is subnormal, while for k = 8 it is the normal 2.22507385850721E-308 and
`ROUND(2^-1022,310)` is 2.23E-308), ROUNDDOWN's (`ROUNDDOWN(x,308)` for k = 8, 2E-308, is 0)
and ROUNDUP's rounded-down part and unit (`ROUNDUP(x,308)` to `ROUNDUP(x,321)` for k = 8 are 0, though the decimal
results 3E-308 to 2.2250738585073E-308 are normal: both 2E-308 and the unit 1E-308 are flushed; `ROUNDUP(x,307)` is
1E-307). The subnormal returned is 2.2250738585071984E-308 for every k = 0..7 and num_digits probed (read through `*2^100`:
2.8206162122887925E-278), 3 ulps below the double nearest the cut (2.2250738585071999E-308); no product or quotient of
the 15 digits by powers of ten (two or three exact-power factors, each order) reproduced it, so the conversion is not
known. The engine returns the nearest double, and misses those 7 read-back cells (rows 52-58). The rule is evidenced for num_digits
from -300 to 330 only; the engine applies it at every num_digits (a huge one such as -400 or 2^31, which is `#NUM!` or
wraps for a normal x, is not probed here; T-2771).
Evidence: probes/t2756-roundup-smallest-normal-threshold (excel-graph, 2026-10-26): rows 1-28 bisect k = 0..20 and
controls (2^-1021, 2^-1020, 3E-308, 9.99999999999999E-308, 1E-307) at num_digits 0, -2, 2, 300 and 330; rows 30-50 take
k = 0, 7, 8 at num_digits 307 to 322 with `=0` companions; rows 52-60 read the subnormals back. probes/t2715-roundup-near-smallest-normal.
Status: VERIFIED (excel-graph), except the subnormal's last bits
Implemented in: `round_function` and `round_decimal` in crates/rustsheet-engine/src/decimal_round.rs
Tasks: T-2756

## SINH / COSH / TANH are exp-based, not libm
Fit against Excel-cached values in `ironcalc-calc/MATH_AND_TRIGONOMETRY/TRIGONOMETRIC.xlsx`
(sheet `SINH_COSH_TANH`; T-0914): COSH = `(e^x + e^-x)/2` (34/34 cells; libm 33/34); SINH uses the
same exp form for |x| >= 1; TANH = `(e^x - e^-x)/(e^x + e^-x)` for |x| >= 1 (all cells), ±1 for |x| > 20.
Below 1, SINH and TANH are built from expm1 (T-2134, "SINH and TANH below 1 are built from expm1" below),
replacing T-0914's Taylor series and sinh/cosh. Excel's expm1 is Kahan's formula over Excel's EXP and LN (T-2240). COTH/CSCH/SECH are 1 over these (T-0917, "COTH, CSCH and SECH" below); ASINH/ACOSH
(T-2031) and ATANH/ACOTH (T-0917) are fitted separately.
T-0111's e^2x TANH on 0.9 <= |x| < 1 is superseded: see "SINH and TANH below 1 are built from expm1" (T-2134).

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
  true value 1.4142764231805424E-6). The same two formulas also give all 33 ASINH and 17 ACOSH cells of ironcalc-calc
  TRIGONOMETRIC.xlsx sheet ASINH_ACOSH_ATANH (gap triage 2026-10-19). Implemented (T-2031): `fn_asinh`/`fn_acosh` evaluate
  these formulas.
- Where `x*x` overflows a double (|x| above about 1.3407807929942596E154) ASINH and ACOSH return `#NUM!` (ln of infinity), for
  both signs of ASINH: probes-numeric/t2062-asinh-acosh-overflow (excel-graph, 2026-10-20). ASINH(+-1E154) = +-355.291251501643
  and ASINH(+-1.3E154) = +-355.55361576611045 are still finite; 1.35E154, 1.4E154, 1E200, 1E308 and 1.7976931348623157E308 are
  all `#NUM!`; the exact cutoff inside 1.3E154..1.35E154 is inferred (the double overflow point), not probed. ACOSH is the same (1E154 and 1.3E154 finite, the rest `#NUM!`). Implemented (T-2062).
- The `ln` in those formulas is the worksheet LN kernel (FLDLN2; FYL2X, "LN is FLDLN2 then FYL2X"), not libm (T-2133).
  At 120 ASINH and 120 ACOSH points picked so that libm `ln` and `excel_ln` of the argument `x + sqrt(x*x +- 1)` differ
  (probes-numeric/t2133-asinh-acosh-ln, excel-graph 2026-10-20), `excel_ln` matches 238 of 240 cells and libm `ln` 2: the
  earlier fitting sets (which libm and `excel_ln` both pass) could not tell them apart. The two misses are ACOSH(20.82932287508296)
  (Excel 3.7289322075948337, ours ...332) and ACOSH(180.0774330018289) (Excel 5.886526412787136, ours ...137), one ulp each;
  sqrt((x-1)(x+1)) and an fma `x*x-1` do not change them, so the argument or an 80-bit intermediate (unprobed) or an FYL2X near-tie
  is the cause (follow-up T-2141). Negative ASINH is `-excel_ln(-x + sqrt(x*x+1))`, as before; the new probe has no negative x. Implemented in `fn_asinh`/`fn_acosh` (`excel_ln`).
- ACOTH for large x (A101-A110, x = 15 .. 1.5E10) tracks libm `atanh(1/x)` within 1 ulp: bit-for-bit on A105 and
  A107-A110, 1 ulp off glibc `atanh(1/x)` on A101-A104 and A106 (A103 matches Rust's `f64::atanh`), so Excel is not
  correctly rounded there either; the formula is the x87 atanh series ("ATANH and ACOTH" below, T-0917). Before T-0917 the engine used `0.5*ln((x+1)/(x-1))`
  there and lost digits (fixed): ACOTH(1.5E7) engine 6.666666662802374E-8 vs Excel 6.666666666666676E-8 (6e-10 relative),
  ACOTH(1.5E10) engine 6.666667217824696E-11 vs Excel 6.666666666666667E-11 (8e-8 relative), both then.
- Near 1, ACOTH(1 + 1E-12) = 14.162039699929037 and ATANH(1 - 1E-12) = 14.162095209226402 differ only because the stored
  inputs sit at different distances from 1 (x-1 = 1.0000889E-12, 1-y = 0.99997788E-12); both are correctly rounded and the
  engine matches both cells. Not a quirk.
- COTH and CSCH (A117-A156) are close to, but not exactly, libm `1/tanh(x)` and `1/sinh(x)`. COTH differs by 1 ulp at
  1E-4, 0.01, 0.1, 2 and 5 (A123, A127, A129, A135, A137) and by 2 ulp at 1E-3 (A125); CSCH differs by 1 ulp at 1E-8 and
  0.01 (A120, A128). Notably CSCH(1E-8) = 1E8 exactly, where libm `1/sinh(x)` (and the engine) gives 99999999.99999999.
  The other COTH/CSCH cells equal the libm expressions bit-for-bit, except CSCH(710) (below); they are 1/TANH and 1/SINH
  ("COTH, CSCH and SECH" below, T-0917).
- Overflow: SINH(710) and SINH(-710) are `#NUM!` (engine: +-1.117E308), but CSCH(710) = 0 (not an error; engine 8.95E-309),
  CSCH(709) = 2.433561501246846E-308, COTH(709) = COTH(710) = 1, TANH(x) = 1 for every probed x >= 19 (19, 19.5, 19.99, 20, 20.01,
  21, 22, 25, 30, 350, 700, 709, 710). SINH(709.78) = 8.964113971972578E307 is finite.
- SINH/TANH values on 0.40..1.0 (step 0.01 / 0.02 / 0.025, plus 0.9950546550253772 and 0.9999999999) are in the case; the
  engine's 0.5 (SINH) and 0.9 (TANH) cutoffs are fitted against them by T-0917. Not probed: SINH in (709.78, 710).
Evidence: `probes-numeric/m9-hyperbolic` (`rustsheet-eval show probes-numeric/m9-hyperbolic.case.json`).
Status: VERIFIED (excel-graph); ASINH and ACOSH implemented by T-2031; ATANH, ACOTH, COTH, CSCH and SECH by T-0917 (next two sections); the SINH/TANH kernel misses remain open (T-0917 Outcome).

### ATANH and ACOTH: an x87 half log on one range, an x87 atanh series on the other (T-0917)
Rule. With RN64 a round to a 64-bit significand, "twice-rounded" an x87 operation stored to a double (RN64, then to 53 bits;
"Division rounds twice" above), and `H(q)` = FLDLN2; FYL2X of the double q (one RN64 of `RN64(ln 2)*log2 q`), halved, stored:
- ATANH(x), |x| < 1: for |x| <= 1E-4 the series `S(x)` below; else `H(q)` with `q` = twice-rounded `(1+x)/(1-x)` and both
  sums twice-rounded. Negative x is evaluated as written (not as `-ATANH(-x)`): ATANH(-0.0002) and ATANH(-0.0003) need it.
  The log branch keeps the cancellation in `1+x`: ATANH(0.0005) = 0.0005000000416666375, 327 ulp under the true value,
  ATANH(0.005) 102 ulp under; the series branch is within an ulp (ATANH(5E-5) = 5.000000004166667E-5).
- ACOTH(x), |x| > 1, odd (the sign is applied last; ACOTH(-3.5) = -ACOTH(3.5), which `H((-2.5)/(-4.5))` misses): for
  |x| < 3.69665 `H(q)` with `q` = twice-rounded `(|x|+1)/(|x|-1)`; else `S(y)` with `y` = RN64(1/|x|) kept in a register.
  The series branch is up to 2 ulp off (ACOTH(4.74) is 2.1 ulp under the true value) and Excel follows it exactly.
- `S(y)` = `y + y^3/3 + y^5/5 + ...` the x87 way: `y` and the running power `y^(2k+1)` (multiplied by an RN64 `y^2`)
  stay in 80-bit registers, each term `power/(2k+1)` is stored to a double, each partial sum is twice-rounded, and the
  loop stops when a term leaves the sum unchanged. A plain double series misses ACOTH(4.7), ACOTH(4.92) and
  ACOTH(3.69734) by an ulp (and fits the other 322 series cells); every variant tried with y or the power stored to a double
  misses at least one of them.
Fit: all 402 ACOTH and 62 ATANH cells of ironcalc-calc TRIGONOMETRIC.xlsx (cached), `probes-numeric/m9-hyperbolic`,
`t0917-hyperbolic-switchover`, `t0917-acoth-switch` and `t0917-acoth-switch-edge` (excel-graph, 2026-10-20). The literal
`4.9999999999999996E-06` (m9-hyperbolic A94) is read as 4.99999999999999E-06 (15 digits) and fits at that value.
Switch points: ATANH's series reaches 1E-4 (A62, A76) and the log starts by 1.1E-4 (A63); ACOTH's log reaches 3.69662
and the series starts by 3.69668 (t0917-acoth-switch-edge A11, A12). Neither constant is pinned closer.
Rejected: libm `atanh`/`atanh(1/x)`, `0.5*ln((x+1)/(x-1))` in doubles (24 of 44 cached ACOTH cells), `0.5*log1p(2/(x-1))`
(32), the correctly rounded `0.5*ln` of the twice-rounded quotient (the x87 FYL2X rounding matters at ACOTH(1.0001)),
the RN64 quotient instead of the stored double one (misses ACOTH(1.9), 2.5, 3.35, 3.39, 3.49).
Status: VERIFIED (excel-graph). Implemented in `hyperbolic::atanh`, `hyperbolic::acoth` (`0.5 * x87_log10::excel_ln`, the worksheet LN kernel).

### COTH, CSCH and SECH are 1 over Excel's own TANH, SINH and COSH (T-0917)
Rule: COTH(x) = 1/TANH(x), CSCH(x) = 1/SINH(x), SECH(x) = 1/COSH(x), Excel's own TANH/SINH/COSH, so wherever those are an ulp
off libm the reciprocals follow them. All 66 pairs of `probes-numeric/t0917-hyperbolic-switchover` (x = 0.001 .. 5 and two
negatives, including x where TANH or SINH is 1 to 1.8 ulp from the true value) agree bit for bit; the earlier note that the
exp forms regressed COTH_CSCH_SECH B10, B47 and C54 came from the engine's TANH/SINH misses, not from the reciprocal.
The division is the worksheet `/` (twice-rounded, like COT/CSC/SEC); no probed cell tells it from a plain division.
Status: VERIFIED (excel-graph). Implemented in `fn_coth`, `fn_csch`, `fn_sech` (mathematical.rs) on `hyperbolic::{tanh, sinh, cosh}`.

### SINH and COSH overflow is `#NUM!` at once; SECH there is 0 (T-2169)
Rule: SINH and COSH are `#NUM!` once `e^|x|` overflows a double, although the true sinh(710) = 1.117E+308 is finite:
`SINH(710)`, `SINH(-710)`, `COSH(710)` and `COSH(-710)` are `#NUM!` (SINH(709.78) is a number, above; the cut in
(709.78, 710) is unprobed). SECH of the same arguments is 0, not an error, as CSCH(710) is: `SECH(710)` = `SECH(-710)` = 0, so
`(x)^SECH(710)` is `x^0` = 1 (`CEILING(-999999999999999.9,(-2.5)^SECH(710))` is -1E+15). The error is the function's own result
and survives nesting, as for EXP: `ISERR(COSH(-710))` is TRUE, `ISNUMBER(SINH(134217728))` FALSE, and `ISODD(EXP(2^53+1))`,
`ATAN(COSH(1E+15))`, `MOD(2^53+1,SINH(1E+9))`, `MOD(ACOT(SINH(1E+15)),<#REF!>)`, `GCD(EXP(1E+15),..)`,
`ARRAYTOTEXT(SINH(134217727),2)`, `TEXTJOIN(..,COSH(1E+9))`, `VALUETOTEXT(x-SINH(-710),0)` and
`SUMXMY2(<TRUE A4:A4>,EXP(2958465))` (a boolean one-cell argument beside the error) are all `#NUM!`.
Evidence (excel-graph, 2026-10-20, T-2114): gen-m5/1962de140dae0111 Calc!A30 and gen-m5/1b30e4d075b252ea Calc!A318 (SECH),
gen-m5/25768de48ae7f2f9 Calc!A315 (SINH(710)), gen-m5/399071ff6782ab16 Calc!A3, gen-m5/9284730052aff46c Calc!A396, gen-m5/90bf45bea539723d Calc!A86,
gen-m5/38b1b8e51dbad326 Calc!A331, gen-m5/bbbd316c346ce645 Calc!A327, gen-m5/97737fad5004ba83 Calc!A21, gen-m5/6a511beadb56b932 Calc!A107,
gen-m5/4febabc66cfdf9a1 Calc!A156, gen-m5/6ba5e4aceeef8209 Calc!A316, gen-m5/d9fb1e918971111b Calc!A108, gen-m5/fc81c2f704bef138 Calc!A46,
gen-m5/ecd1b828aebad1e8 Calc!A185; the full list of 37 cells is in T-2169. A156 still fails, for ARRAYTOTEXT's own reason
(text.md, T-2250).
Unprobed: COSH and SECH on (700, 710). libm's cosh was finite up to 710.47 and ours now overflows just above 709.78, as SINH does.
Status: VERIFIED (excel-graph) for SINH, COSH and EXP. Implemented (T-2169): `hyperbolic::{sinh,cosh}` overflow to infinity and
`number_fn` maps a non-finite result to `#NUM!`. That mapping applies to every one-number math function through `number_fn`
(DEGREES, RADIANS, FACT, the reciprocal trig functions, ...); outside SINH, COSH and EXP it is unprobed (hand: Excel has no
infinities).

### SINH and TANH on 0.4 .. 1: no formula found yet (T-0917, negative result)
Excel's SINH and TANH are not correctly rounded on (0.4, 1): errors up to 1.45 ulp (SINH(0.7)) and 1.8 ulp (TANH(0.54)) against
the true value (probes-numeric m9-hyperbolic A157-A248), so the kernel works in doubles somewhere. None of these fits all cells
(tested 2026-10-20 against the 85 SINH and 87 TANH cells): libm sinh/tanh, `(e^x - e^-x)/2` and `(e - 1/e)/2` with libm exp or
Excel's x87 EXP (model V6), the same in 80-bit, the engine's Taylor series (forward, reverse, Horner, 10 or 16 terms), and for
TANH `(e^2x - 1)/(e^2x + 1)`, `1 - 2/(e^2x + 1)`, `sinh/cosh`, `expm1` forms, each with libm or V6 exp. The engine keeps T-0914's
piecewise forms (SINH 60/85, TANH 67/87 cells); COTH/CSCH inherit every one of their misses (see the failing COTH/CSCH cells
of probes-numeric `m9-hyperbolic` and `t0917-hyperbolic-switchover`; decision 0084 lists those that libm used to pass).
Dense fitting set (T-2178): probes-numeric `t2178-hyperbolic-unit-sweep` (1035 arguments on (-1.5, 1.5): k/500, seeded random,
log-spaced 1E-8 .. 1E-2, the cached misses' arguments, 100 negatives, 20 control points past 1; SINH/TANH/COSH each; excel-graph).
The current engine matches 2236 of 3105 cells: SINH misses 403, TANH 464, COSH 2. Engine-vs-Excel gaps over the 869 misses: 735 at
1 ulp, 123 at 2, 10 at 3, 1 at 4 (Calc!C659, TANH(0.5345135074151877): Excel 0.48882367384055525). The sweep shows the quirk is not confined to 0.4 .. 1: an
mpmath check during the T-2178 review found Excel up to 2.57 ulp from the true value for TANH (x = 0.2533858999427424) and 1.93 ulp
for SINH (+-0.8541474891480176), with misses also below 0.4; COSH stayed within 0.98 ulp (looks correctly rounded here).
Calc!A1035 stores 0.995054655025377: m9-hyperbolic A245/A246 read their 16-digit literal as this 15-digit value
(engineering.md, 16-digit literal quirk), so their results differ from the sweep's 0.9950546550253772 cells.
Status: superseded by "SINH and TANH below 1 are built from expm1" (T-2134), which explains the shape; the expm1
kernel is Kahan's formula ("Excel's expm1 is Kahan's formula", T-2240).

### SINH and TANH below 1 are built from expm1 (T-2134)
Rule. For |x| < 1, with a = expm1(x) and b = expm1(-x), each a double:
- SINH(x) = (a - b)/2;
- TANH(x) = (a - b)/((a + b) + 2), summed in that order.
COTH and CSCH, as 1 over TANH and SINH, follow. COSH is not part of it: it stays `(e^x + e^-x)/2` (1033 of the
sweep's 1035 COSH cells).
- Evidence: probes-numeric `t2178-hyperbolic-unit-sweep` (excel-graph), 1015 arguments with 0 < |x| < 1. With libm's
  (glibc) expm1 the two forms match 773 SINH and 781 TANH cells. T-0914's forms (series SINH below 0.5, exp form above,
  sinh/cosh TANH, e^2x TANH from 0.9) match 612 and 551. The expm1 forms win or tie in every band from 1E-8 to 1.
  - At x of 1E-8 to 1E-4, Excel's SINH is up to 0.98 ulp from the true value, and `SINH(1.519911E-8)` is x plus one ulp
    where x^3/6 is 0.18 ulp. So the result is not x plus a small correction; it is the difference of two rounded
    values of size x, as in a - b.
  - TANH below 1E-4 shows the same: `TANH(2.656088E-8)` is x minus 3 ulp where x^3/3 is 1.9 ulp.
- Excel's expm1 is not libm's. For every one of the 1015 arguments, some pair (a, b) within 3 ulp of libm's
  (expm1(x), expm1(-x)) gives both Excel's SINH and Excel's TANH. On 747 arguments libm's own pair does. These do not
  reproduce Excel's expm1 either, being as close to correctly rounded as libm:
  - x87 F2XM1 of t = RN64(x log2 e) (or t rounded to a double), stored to a double, with or without a reciprocal
    for t < 0;
  - expm1 kept in 64 bits and the forms rounded once.
  At x = PI()/4 and +-0.9999999999 (the cached `TRIGONOMETRIC.xlsx` B51 and B15 rows and the gen-m5 PI()/4 cells) libm's
  expm1 lands an ulp off Excel's (decision 0092).
- Rejected on the same 1015 arguments (SINH / TANH cells matched):
  - Cephes `sinh`/`tanh`: 713 / 545;
  - Cody-Waite rational forms: 707 / 541;
  - fdlibm `sinh` and `tanh`, built on expm1 too: 686 / 450;
  - (a - b)/(2 + a + b): TANH 644;
  - (a + a/(a + 1))/2 for SINH: 703;
  - expm1 replaced by exp(x) - 1 above any cut from 0.2 to 1: never better than expm1 throughout.
- At |x| >= 1 the exp forms stay: they match all 20 of the sweep's control points in (1, 1.5), and the expm1 forms 17.
Ours: `functions/math_and_trigonometry/hyperbolic.rs` `sinh`, `tanh` (`expm1_pair`, `excel_expm1`).
Status: VERIFIED (excel-graph); the expm1 kernel is below.

### Excel's expm1 is Kahan's formula (T-2240)
Rule. The expm1 under SINH and TANH below 1 is Kahan's: with u = EXP(x) (Excel's own EXP, model V6, "EXP" above),
expm1(x) = x if u = 1, and otherwise (u - 1) * x / LN(u), with LN Excel's own (FYL2X, `excel_ln`), each
operation a double and evaluated left to right: (u - 1) * x first, then the divide. (Kahan's -1 for u - 1 = -1 is kept
as a guard of ours; it cannot occur below |x| = 1 and no cell probes it.)
- The u = 1 branch (|x| under about 1E-16) is covered only by cached cells: m9-hyperbolic `SINH(1E-10)`, `SINH(3E-10)`,
  `TANH(5E-10)` and TRIGONOMETRIC.xlsx `SINH(1E-12)`/`TANH(1E-12)` pass; the sweep starts at 1E-8.
- Evidence: probes-numeric `t2178-hyperbolic-unit-sweep` (excel-graph): all 1015 arguments with 0 < |x| < 1 match in both
  SINH and TANH (libm's expm1: 773 and 781). With libm's exp and log in the same formula it matches 1014 of 1015;
  Excel's EXP with libm's log also 1014.
- Rejected: the other association (u - 1) * (x / ln u), 828 SINH / 820 TANH, and (u - 1) / (ln u / x), 830 / 824; EXP(x) - 1
  alone, 486 / 484. Rounding the product and quotient to 64 bits before the double (x87 double rounding) also fits all
  1015, so the sweep does not tell the two apart; the engine keeps plain doubles. Earlier negative results (x87 F2XM1,
  64-bit expm1) are in the entry above.
Ours: `excel_expm1` in `x87_exp.rs` (moved from hyperbolic.rs by T-2242). PPMT also uses it, for `1-EXP(-n*L)` with |x| = n*L up to
hundreds (financial.md "PPMT discounts with an atanh-series log", probes-numeric/ppmt-log-rate-sweep), where the u - 1 = -1 guard
is reachable (n*L above about 36.7).
Status: VERIFIED (excel-graph).

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
247 of 666 cells passed. The formulation is the x87 64-bit pi reduction ("Trig of a huge angle", T-1563), and COT, CSC and SEC
divide as "Division rounds twice" does (T-0915); all 666 cells pass since then.
Status: VERIFIED (excel-graph); implemented (functions/math_and_trigonometry/trig.rs `reduce`, mathematical.rs `reciprocal_trig`).

## Non-integer exponents: exp(y·ln x)
Rule: for an exponent that is not an integer, Excel computes `x^y` as follows:
- `y` > 0: `exp(y·ln x)`. `6^2.5` = 88.18163074019438, 2 ulps below libm `pow`, and `10^33.5` = 3.162277660168401E+33.
- `y` < 0: one over the positive power, `1/exp(-y·ln x)`. `10^-33.5` = 3.1622776601683576E-34 and `27^-2.5` = 0.0002639918926335737.
- `y` = 0.5: `sqrt(x)`. `2^0.5` = 1.4142135623730951, where `exp(0.5·ln 2)` is 1.4142135623730950.
- `y` = -0.5: `1/sqrt(x)`. `3^-0.5` = 0.5773502691896258 and `0.5^-0.5` = 1.414213562373095.
- Each `1/` is the operator `/`, which rounds twice (see "Division rounds twice"): `8^(-1/3)` = 0.5.
- `exp` is EXP's own kernel (see "EXP is an x87 2^(a*log2 e) evaluation"), and `y·ln x` is the twice-rounded `*`: `1E300^(1/3)` = `EXP(LN(1E300)*(1/3))` =
  9.999999999999827E+99, where libm `exp` gives 9.999999999999825E+99, and `8^(1/101+28·2^-59)` = 1.0208019359564402
  (probes-numeric/power-ulp-sources A13-A18, power-odd-root-tolerance A72/A74; excel-graph, 2026-10-19, T-0959).
- `ln x` is LN's own x87 FYL2X kernel (see `excel_ln`), not a correctly rounded libm `ln` (T-2694). Over the T-2513 and
  T-2586 density studies (half-integer exponents -50.5 to 50.5, over 7,000 POWER cells) libm `ln` left six cells 1 to 10
  ulp off from Excel, which is below the correctly rounded value in all six; Excel's LN matches all six to the bit and changes no other cell of
  probes-numeric's 611,774. `POWER(12/(12+A563*A563),6.5)` with A563 = 7.71216677000003 is 9.174378504956638E-06 (libm
  `ln` 9.174378504956655E-06; the correctly rounded value is 4.4 ulp above Excel's) (probes-numeric/t2513-r3-tdensity-hold I563,
  I475-I477, t2513-r3-tdensity-a2 D653, t2586-r4-f-small-df-a D4143; excel-graph). MIRR's root keeps libm `ln` and
  `exp` (`power_with_libm_exp`, decisions 0030 and 0039).

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
Implemented in: `non_integer_power` in crates/rustsheet-engine/src/arithmetic/power.rs.
Status: VERIFIED (excel-graph)
Tasks: T-1319, T-1421, T-0221, T-0959, T-2694

## POWER and `^`: odd roots of negatives, zero and overflow
Rule: `POWER(-8, 1/3)` is -1.9999999999999998 (an odd root of a negative number is real; see "Non-integer exponents: exp(y·ln x)"); `POWER(0,0)` is #NUM!;
`POWER(0,-1)` is #DIV/0!; text operands give #VALUE!; array operands work element by element.
Evidence: ironcalc-calc/MATH_AND_TRIGONOMETRY/POWER.xlsx (Excel-authored): Sheet1!D2 (#DIV/0!),
D3 and F9 (-1.9999999999999998, matched since T-1319: see "Non-integer exponents"), sheets 2-8 for ranges.
The operator `^` shares the zero case: `0^-0`, `+0^0`, `(-0)^0` and `0%^0` are all `#NUM!`, while
`-0^0.5` = 0 (gen-m1/6462bf910ef9f28f Calc!A19, A30, A205, A222; gen-m1/1c62c4bff4ed1976 Calc!A60;
oracle: excel-graph, 2026-10-05, T-0109).
Which exponents count as 1/odd: Excel rounds `1/y` to 15 significant digits and takes the root when that is an odd
integer below 2^31 - 1 in magnitude; any other non-integer exponent of a negative base is #NUM!. The root then uses `y`
itself, not 1/n: `(-8)^(1/3+8·2^-54)` is -2.0000000000000018.
- `1/3 + 10·2^-54` counts (1/y = 2.9999999999999951 → 3) but `1/3 - 10·2^-54` does not (3.0000000000000053 → 3.00000000000001),
  so the cut is not symmetric in y and is not a relative tolerance on y: `1/5 + 8·2^-55` is #NUM! while `1/3 + 8·2^-54` counts.
- `1/101 ± 28·2^-59` counts and `± 29·2^-59` does not (the 15th digit of 101.000000000000).
- Literals: `0.333333333333333` counts (-1.9999999999999984), `0.33333333333333` does not; `0.0909090909090909` (1/11) counts,
  `0.142857142857143` (1/7, 1/y = 6.999999999999993) does not.
- Size: `1/(2^31-3)` and `1/(2^31-5)` count; `1/(2^31-1)`, `1/(2^31+1)`, `1/(2^40+1)` and `1/(2^52+1)` are #NUM!, although
  1/y is exactly the odd integer there. Not probed: negative exponents at the limit (`-1/(2^31-1)`, `-1/(2^31-3)`); the engine
  takes the limit on |1/y|.
- Even reciprocals (`1/4 + 2^-55`, `1/2^20`) are #NUM!; `3/9`, `2/6` and `1/(-3)` are odd roots like 1/3 and -1/3.
Overflow of a non-zero base is #NUM! (`10^309`, `(-10)^309`, `1E300^1.5`, `(-1E300)^3`, `2^1024`) and underflow follows
"Non-integer exponents" (`1E-300^-1.5` #DIV/0!, `10^-309` 0). The operator `^` and POWER agree on every cell probed: the
odd-root cut, overflow, zero (`0^-1` #DIV/0!, `0^-0.5` #DIV/0!), negative bases (`(-8)^0.5` #NUM!), the 2^32 - 1 exponent limit,
text (`"2"^2` = 4, `"a"^2` #VALUE!, `""^2` #VALUE!), logicals (`TRUE^2` = 1), blank cells (`Z99^Z99` #NUM!, `Z99^-1` #DIV/0!) and
errors (`NA()^0` #N/A, `(1/0)^0` #DIV/0!).
Evidence: probes-numeric/power-odd-root-tolerance (A is `^`, B is POWER; rows 1-59 the cut, 60-101 the special cases) and
power-odd-root-tolerance-2 (the cut's edges and the size limit); oracle excel-graph, 2026-10-19, T-0959. A workbook holding
the literal `1E308` cannot be filled by excel-graph (engineering.md "CONVERT"), so `1^9E307` stands in for `1^1E308`.
Status: VERIFIED (excel-graph).
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

## ACOS is PI()/2 - ASIN(x)
Rule: Excel's `ACOS(x)` is `π/2 - ASIN(x)` in doubles, on ASIN's kernel ("ASIN's formulation"). So `ACOS(0.5)` =
1.0471975511965976, the double nearest π/3, where libm gives 1.0471975511965979.
Evidence: all 240 ACOS cells of probes/asin-atan-sign-source (`ACOS(x)` and `ACOS(-x)`, H and I; excel-graph, 2026-10-18, T-1797),
where libm `acos` fits 157; poi/48539.xlsx Instance1!E23 (oracle: excel-graph, 2026-10-04); poi/FormulaEvalTestData_Copy.xlsx Q104.
Implemented in: `fn_acos` (functions/math_and_trigonometry/mathematical.rs) as `FRAC_PI_2 - excel_asin(x)` since T-0939.
Status: VERIFIED
Tasks: T-0939

## ATAN2 is ATAN(y/x) plus or minus π
Rule: for `ATAN2(x_num, y_num)` Excel forms the quotient `q = y/x` with its own `/` ("A subnormal divisor is zero",
overflow is #NUM!) and returns `ATAN(q)`, not libm `atan2(y, x)`. For x < 0 it adds π when y >= 0 (so `ATAN2(-1,-0)` = π)
and subtracts π when y < 0. x = 0 gives ±π/2 by the sign of y (a kept subnormal y counts as non-zero), and
`ATAN2(0,0)` is #DIV/0!. A subnormal x (kept, e.g. `3E-308-2.5E-308`) is #DIV/0! for any y, even `ATAN2(S,1E300)`;
a quotient that overflows (`ATAN2(1E-300,1E300)`, `ATAN2(1E-8,1E305)`) is #NUM! while `ATAN2(1E-8,1E300)` is π/2.
An underflowing quotient (`ATAN2(1E300,-1E-10)`) gives 0, or ±π for x < 0 by y's sign; a subnormal y with x < 0
keeps its sign (`ATAN2(-1,-S)` = -π). Literal subnormal inputs read as 0 first ("Subnormal results become 0"), a separate
quirk. The ATAN kernel is the engine's `f64::atan`, as in `fn_atan`; ulp parity depends on it.
Evidence: probes/atan2-quadrants (all 123 numeric cells equal this model; libm `atan2` misses 25 by one ulp, as does
the 34-cell G column of probes/asin-atan-sign-source), probes/atan2-error-edges, probes/atan2-kept-subnormal (with `=0`
companions, decision 0042); excel-graph, 2026-10-19, T-1930.
Implemented in: `fn_atan2` (functions/math_and_trigonometry/mathematical.rs). IMARGUMENT is the same kernel (77 of 77 cells of probes-numeric im-last-digit-sweep; engineering.md "IM* reciprocal kernels, IMABS and IMARGUMENT", T-2044).
Status: VERIFIED
Tasks: T-1930, T-1797

## LOG with one argument is LOG10
Rule: `LOG(x)` with the base omitted returns the same double as `LOG10(x)`, not ln(x)/ln(10): `LOG(1000)` is exactly 3
(ln/ln gives 2.9999999999999996). That log10 is an x87 `FLDLG2; FYL2X` result stored to a double: `RN53(RN64(y * log2 x))`
with `y = RN64(log10 2)` = `0x9A209A84FBCFF799 * 2^-65`, two roundings. It is therefore neither glibc's `log10` nor always
the correctly rounded log10: `LOG10(295344268)` is 8.47032854654939 (correctly rounded: 8.470328546549387) and
`LOG10(616.3769781444394)` 2.7898464093268007 (correctly rounded: ...003), while glibc misses `LOG10(11)` (Excel
1.0413926851582251, glibc ...225). On k = 1..170 the model equals the correctly rounded log10 (the MULTINOMIAL fit, "MULTINOMIAL
is inexact", needed exactly those terms). The doubles nearest 10^k (k = -20..20) give k exactly.
Evidence: probes/log10-kernel (excel-graph, 2026-10-19, T-2004): 92 inputs (the workbook's 7 numbers, 10^-20..10^20,
32 doubles where glibc is not correctly rounded, 16 where the correctly rounded value and the x87 models differ) for LOG10 and
LOG: LOG = LOG10 in all 92, and the model matches 92 of 92 (correctly rounded 78, one rounding of the exact y·log2 x 85,
glibc 56, ln/ln 42). The web agrees with the desktop cell `LOG10(1.3)` = 0.11394335230683679.
ironcalc-calc/MATH_AND_TRIGONOMETRY/LOG_LOG10_LN.xlsx (Excel cache, Microsoft Excel 16.0300; the 3-sheet Macintosh file of
T-1930 was replaced by upstream's single-sheet version in T-2350): one-argument LOG(x) equals LOG10(x) bit for bit in
C2/D2 (1), C5/D5 (86 -> 1.9344984512435677), C6/D6 (EXP(1) -> 0.4342944819032518), C7/D7, C10/D10 (1000 -> 3), C22/D22
(1E-5 -> -5), C27/D27 and C31/D31 (EXP(709) -> 307.91478766940554). "the workbook's 7 numbers" and the desktop cell
`LOG10(1.3)` above are from that earlier 3-sheet version.
Two-argument LOG is ln(x)/ln(b) in doubles (`f64::log`): it matches all 92 `LOG(x,2)` and 92 `LOG(x,3)` cells of the probe
(columns D and E). The workbook adds other bases: `LOG(1000,10)` is 2.9999999999999996 (C9, C67) while
`LOG10(1000)` is 3 (D9), `LOG(0.004,10)` -2.397940008672037 against `LOG10(0.004)` -2.3979400086720375 (C21, D21), `LOG(1E-3,10)` -2.9999999999999996 (C20) and `LOG(1E300,10)` 299.99999999999994 (C29), so
two-argument LOG with base 10 is not LOG10.
Implemented in `x87_log10.rs` (`excel_log10`: ln x in double-double, about 2^-104 relative, times `y / ln 2`, then the two
roundings; 30,000 random inputs equal a decimal reference of the model), used by LOG10, one-argument LOG and MULTINOMIAL's
`log10(k)` terms (which replaced their 170-entry table). It also fixes probes/geomean-root-models Root!I6 and I9
(`10^(LOG10(E)/C)`).
Domain and coercion of LOG (Excel cache of LOG_LOG10_LN.xlsx, T-2350; the engine matches all 227 cells): base 1 gives
`#DIV/0!` even for `LOG(1,1)` and `LOG(100,TRUE)` (C36, C37, C59); base 0 or a negative base gives `#NUM!` (C38, C39);
an omitted argument counts as 0, so `LOG(,b)`, `LOG(x,)` and `LOG(,)` give `#NUM!` (C48-C50); with both arguments errors the
first argument's error wins (C53, `#DIV/0!`); a direct boolean is a number (`LOG(TRUE,10)` = 0, C58; `LOG10(FALSE)` `#NUM!`,
D59); text is coerced when it reads as a number (`LOG("100","10")` = 2, C69; C66, C68) and is `#VALUE!` otherwise
(`LOG("abc")`, `LOG("")`, C70, C71; the text cells A63, A64, A65 in C63-C65); a whole-column argument takes the implicit
intersection (`LOG(G:G,B75)`, C75, C76).
Status: VERIFIED (excel-graph, and the desktop cache for the workbook cells)
Tasks: T-2004, T-2350

## LN is FLDLN2 then FYL2X
Rule: `LN(x)` is the x87 `FLDLN2; FYL2X` result stored to a double, `RN53(RN64(y * log2 x))` with `y = RN64(ln 2)` =
`0xB17217F7D1CF79AC * 2^-64`: the same two-rounding evaluation as LOG10 above, with the other constant. It is not glibc's `log`
and not always the correctly rounded ln: `LN(1234.5617000014759)` is 7.11847128728637 and `LN(2.7182911000046204)`
1.000003410805188 in Excel, one ulp from the model, and the model in turn differs from the correctly rounded ln elsewhere.
Evidence: the T-1941 research runs (excel-graph, 2026-10-19/20; `kmyst-gam-*` on branch
`agent/research/T-1941-crack-gammaln-gamma-ln`, every cell whose formula is `=LN(A<row>)`): 2,887,551 cells, 2,677,612
distinct x from 1.37e-9 to 7.8e299. The model (`docs/research/t1941/lnmodels.py`) matches 2,887,521; the 30 misses are each
one ulp, at points where Excel's FYL2X rounds the other way (the research's first 114,688-cell run had one, x =
0.612354100000274). At those near-ties Excel is not even consistent with itself: probes/ln-fyl2x-kernel (excel-graph,
2026-10-20; the 30 points plus 223 sampled ones) returned the model's value at 3 of the 30 (2.426003200000124,
9.612345678912595, 1.2345670000006508e100) where the research run of 2026-10-19 returned the other neighbour, and agreed with
the research run on the other 250 cells, so the last-bit rounding of an FYL2X near-tie varies between runs (plausibly the
server's CPU), and following the model is the stable choice. glibc's `log` matches 2,674,450 of the 2,677,612 distinct x (the engine used it before).
The probe keeps the 27 near-ties of that run as known failures in eval/baseline/probes.txt.
T-2143 sweep (probes-numeric/t2143-ln-near-ties-fill1..3, excel-graph, three fills of one workbook of 380 x on 2026-10-22 within
35 s of each other, fills 2 and 3 with a label cell so the oracle cache could not return fill 1; "d" is the distance of the exact
product `RN64(ln 2) * log2 x` from the nearest 53-bit midpoint, in 53-bit ulps, 1 ulp64 = 2^-11 = 4.9e-4):
- Near-ties: of the 30 probe points (d 4.7e-5 .. 2.7e-4), 12 return the same bits in all three fills and 18 do not; the fills match
  the model at 3, 3 and 17 of the 30. So Excel's bit at a near-tie is not a fixed function of x: the same workbook, calculated
  three times minutes apart, gave different bits (fill 3 mostly the model's, fills 1 and 2 mostly the other neighbour). The
  disagreement is always between two adjacent doubles.
- Window: all 23 disagreeing points have d between 2.19e-4 and 3.86e-4, around half a 64-bit ulp (2.44e-4), where RN64 of
  the product is itself a coin flip. Of the 330 near-tie points (30 + 300 new), the 119 with d below 2.19e-4 and the 142 above
  3.87e-4 return the model's bits in all three fills; the 69 in between split into 34 stable matches and 35 that differ from the
  model in at least one fill (23 differ between fills).
  The smallest d probed is 3.3e-5 (4.7e-5 for the old points); nothing closer than 2.19e-4 to a midpoint was unstable.
- Controls: all 50 points at 0.1-0.4 ulp53 from a midpoint agree across fills and equal the model.
- Consequence for T-2144: no single model can match every fill at the unstable points; fit only on points that agree
  across fills (357 of 380 here); 12 of those 357 (11 old near-ties and one new point, d 2.36e-4 to 2.71e-4) stably return the
  neighbour the model does not, so inside the window Excel can be stable and still differ from the model.

T-2144 fit (docs/research/t1941/ln_guard_bits.py): a k-bit internal product, truncated (tz) or rounded (rn), before RN64
and RN53. Matches out of the 357 fill-stable points of T-2143's case / the fixture's 4,000 sampled points / its 30 near-ties
(one research run) / probes/ln-fyl2x-kernel's 253 cells:

| model | stable | sample | near-ties | kernel probe |
|---|---|---|---|---|
| shipped (no k) | 345 | 4000 | 0 | 226 |
| k=66 tz / rn | 350 / 351 | 3998 / 3999 | 10 / 18 | 232 / 242 |
| k=67 tz / rn | 353 / 354 | 3999 / 4000 | 9 / 18 | 233 / 242 |
| k=68 tz / rn | 354 / 352 | 4000 / 4000 | 9 / 15 | 233 / 239 |
| k=69 tz / rn | 352 / 350 | 4000 / 4000 | 8 / 10 | 232 / 236 |
| k=70 tz / rn | 350 / 348 | 4000 / 4000 | 5 / 5 | 231 / 231 |
| k=72 tz / rn | 346 / 346 | 4000 / 4000 | 1 / 1 | 227 / 227 |

No candidate matches every fill-stable point. k=67 rn beats the shipped model on every column and loses no
fill-stable point; its one loss is kernel probe B231 (x = 9.612345678912595), a near-tie whose Excel bit already differs between
this probe's fill and the research run. It is the candidate T-2651 tests on the full T-1941 near-tie set before shipping.
The model is unchanged until then (decision 0112).
Implemented in `x87_log10.rs` (`excel_ln`, sharing `excel_log10`'s FYL2X evaluation), used by the worksheet LN. A committed
sample of 4,000 points plus every miss pins it to the Python model and to Excel (`x87_log10.rs` tests,
`test/fixtures/ln_kernel.fixture`); `ln_matches_the_t1941_runs` reruns the full set when `RUSTSHEET_T1941_ROWS` names the extracted rows.
GAMMALN (T-2128) and GAMMA (T-2129) still use the platform ln until they ship.
Status: VERIFIED (excel-graph)
Tasks: T-2123, T-1941, T-2143, T-2144

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

## POWER when y·ln x overflows (T-2661)
Rule: for a positive base and a huge non-integer-path exponent (|y| > 2^53 or not an integer), a product `|y|·ln x` that
overflows the doubles counts as an overflow of the positive power, even when it overflows to -∞ (a base below 1): the
positive power is `#NUM!` and the negative one is 0, the reverse of the underflow results. `POWER(0.1,9.99999999999999E+307)`
is `#NUM!` and `POWER(0.1,-9.99999999999999E+307)` is 0 (|y|·ln 0.1 is about -2.3E+308), likewise for bases 1E-10, 1E-300
and 1E-307; but `POWER(0.5,9.99999999999999E+307)` is 0 and `POWER(0.5,-9.99999999999999E+307)` `#DIV/0!` (a finite
product, -6.9E+307, that underflows), and so are 0.9, 0.999999 and every base below 1 to `±1E+300`. Bases above 1 overflow
as before (`#NUM!` up, 0 down), 1 to any power is 1, and 0 keeps `0` / `#DIV/0!` (ln 0 is -∞ already). `^` agrees on every
cell. This settles the calipers desktop goldens `formulas_math_basic` / `formulas_datetime` J75 (`POWER(10^-307,
9.99999999999999E+307)`, `#NUM!`) and K75 (the negative exponent, 0).
Evidence: probes/power-huge-exponent (POWER) and power-op-huge-exponent (`^`), 15 bases by 20 exponents each, oracle
excel-graph, 2026-10-25.
Edge (T-2696): the switch is consistent with the double product `|y|·ln x` overflowing (not a fixed exponent cutoff). For
y = 9.99999999999999E+307 it lies between 0.165 (`#NUM!`; 0 for the negative exponent) and 0.166 (0; `#DIV/0!`), where
the predicted ln x = -1.797 gives x about 0.1659. Bases 0.15 to 0.2 with y = 1.5E+308 and 1.7E+308 are all `#NUM!` (0
negative), while 0.5 and 0.9 with the same exponents, and every base with y = 1E+307 or 5E+307, are 0 (`#DIV/0!`
negative). `^` agrees on every cell. Evidence: probes/power-overflow-edge (172 cells, POWER and `^`), oracle
excel-graph, 2026-10-26; the engine already matched all of them.
Status: VERIFIED (excel-graph)
Implemented in: `positive_power` in crates/rustsheet-engine/src/arithmetic/power.rs
Tasks: T-2661, T-2696

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
Implemented in: `integer_power` in crates/rustsheet-engine/src/arithmetic/power.rs (T-1302), for finite integer
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
With products rounded once (the engine before T-1495) it gives 2 of the 42 (…6807E+34,
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
Implemented in: `power` and `integer_power` in crates/rustsheet-engine/src/arithmetic/power.rs;
`square_and_multiply` rounds each product with `extended_product` (T-1495).
Status: VERIFIED (the 2^32 - 1 threshold is bracketed by 4294967294 and 4294967295 exactly; the base-10
rule is sampled at about 20 exponents in 23..=308 and inferred between them)
Tasks: T-1311, T-1495

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
SIN(4) (T-1876, decision 0074): `ironcalc-calc/LOGICAL/IF_ARRAY.xlsx` Sheet1!C4, E4 and C11 cache the Mac 16.0300 value
-0.7568024953079283. The web and the engine give -0.7568024953079282, the correctly rounded value: the true sin(4) is 0.06 ulp
from the midpoint, and the desktop value is 0.56 ulp off. The 38 desktop SIN/COS cells of TRIGONOMETRIC.xlsx `SIN_COS_TAN` are all
correctly rounded. Fit over those 38 and 224 web cells (m9-trig-reduction and trig-huge-argument, |x| < 2^27), desktop / web out of 38 / 224:
- The engine model (x87 π₆₄ reduction, then a correctly rounded sin or cos of the remainder) 38 / 224. Rounding the remainder, or the result, to 64 bits first: 38 / 224 each.
- A correctly rounded sin 38 / 115. glibc and fdlibm/musl 38 / 115 each.
- The π₆₄ remainder rounded to a double 31 / 219. fdlibm `rem_pio2` with libm on the head only 31 / 113. A double-π/2-plus-tail reduction 31-33 / 105.
- An idealized x87 `fsin` (the true sine at 64 bits, then a double) 38 / 115. A double-π reduction 20 / 94.
- All of these give ...282 except the double-π reduction (...284).
So no formulation reaches the desktop value. The three cells stay known failures as an Excel-vs-Excel last-digit conflict, and the
desktop target waits for T-1075 (M14).
ATAN2 (gap triage 2026-10-20, T-2142): the two ATAN2 cells of `ironcalc-calc/MATH_AND_TRIGONOMETRY/ATAN2_POWER.xlsx`
(Mac 16.0300) where desktop and web differ have the same shape as SIN(4). Sheet1!D14 is `ATAN2(1,3.141592653589793)`:
desktop 1.2626272556789115, web and engine ...118. D26 is `ATAN2(TRUE,2)`: desktop 1.1071487177940906, web and engine ...904.
The exact atan(pi) is 1.26262725567891167... and the exact atan(2) is 1.10714871779409050.... So the web values are
correctly rounded (0.41 and 0.42 ulp off). The desktop values are 0.59 and 0.58 ulp off, and both points lie within
0.09 ulp of a rounding midpoint. The other 18 numeric desktop ATAN2 cells of the workbook match the web model.
T-2142 ran a fit (`docs/research/t2142/fit_atan2.py`, run command in its docstring) over the 20 numeric desktop cells and 257 web cells: probes/atan2-quadrants (120), atan2-error-edges (17)
and column G of asin-atan-sign-source (120). The quotient is Excel's `/` (rounded to 64 bits, then to a double), and atan is exact to
90 digits. atan2-kept-subnormal was left out, because its inputs are formulas of subnormal differences; the engine passes it.
Results, desktop / web:
- The engine model, a correctly rounded atan of the quotient with ±π added in doubles: 18 / 257. Rounding atan to 64 bits first
  scores the same. So does π/2 − atan(1/q) for |q| > 1, done entirely in x87 (64-bit) steps.
- fdlibm/musl `atan` of the quotient: 19 / 248. It gives D14's ...115 but not D26.
- π/4 + atan((q−1)/(q+1)) for |q| > 0.4375, in doubles: 19 / 231. It also gives D14 but not D26.
- π/2 − atan(1/q) in doubles, for |q| > 1: 18 / 232. It misses D17 and D26.
- The quotient rounded to 64 bits only: 18 / 217. x87 `FPATAN` of the exact (y, x): 18 / 198. Adding ±π in x87: 18 / 238.
- asin(q/sqrt(1+q²)): 18 / 169. 2·atan(q/(1+sqrt(1+q²))): 12 / 175.
No candidate gives both desktop cells, and the two that give D14 lose web cells. The engine keeps the web value, and decision 0086
names D14 and D26 as an Excel-vs-Excel last-digit conflict. Their desktop target waits for T-1075 (M14), as SIN(4) does.
MIRR (T-0959): Mac 16.0300 caches `MIRR({40,-5,-5,-5,-5,-5},0.1,0.2)` as 0.3933280241536672, the web gives ...674. The engine keeps
the desktop value with MIRR's own root (financial.md "MIRR's evaluation order"); `^` follows the web.

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
exactly r. Excel's own LN and EXP equal glibc's `log` and `exp` bit for bit on every probed GEOMEAN P
(not in general: T-1778 finds Excel's EXP is its own routine, model V6), and `EXP(LN(P)/n)` in a cell is r, so the remaining misses are not a libm difference.
Remaining misses (r and c both wrong, one ulp): the 20-value mixed set (geomean-in-range-many A2,
0.61 ulp from the exact root) and 25 of the 48 sets of probes/geomean-root-refinement: 22 of rows
19-48 (picked because r and a Newton-corrected root differ there), A17 (GEOMEAN(7,7,1)), and A7 and
A8. A7 and A8 refute the snap's condition as Excel's own: GEOMEAN(0.01,1) and GEOMEAN(1,0.01) are
exactly 0.1, the correctly rounded root, although 0.1*0.1 is not the double 0.01, so c does not
multiply back to P there. Also probes/subtotal-reference-operands-d H28: GEOMEAN(5,7) is
5.916079783099616, the correctly rounded sqrt(35), where r is 5.916079783099615, 0.90 ulp below the
exact root (T-1284). The distance of r from the exact root does not decide the outcome: over the 70
in-range literal GEOMEAN probe cells, Excel keeps r at 0.548 ulp (geomean-same-product A1-A7, P = 3 as in
C23) and at 0.809 ulp (A16-A20) and gives the correctly rounded root at 0.546 ulp (geomean-root-refinement A22).
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
scored above, plus 8 in geomean-same-product and subtotal-reference-operands-d H28).
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
2^48 or more (2^48-1 is accepted). BITOR and BITLSHIFT agree (engineering.md "Engineering edge-pool smoke check
(T-2249)", UNVERIFIED until T-2421 commits the probe).
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
- An `options` below 0 is `#VALUE!` before it is truncated: `AGGREGATE(15,-0.0001,E1:E6,1)`, `AGGREGATE(9,-0.5,E1:E6)` and
  `AGGREGATE(15,-1,..)` are `#VALUE!`, while `AGGREGATE(15,7.9,..)` is option 7 and `AGGREGATE(14.5,0,..)` is function 14. A
  multi-cell range k for functions 14-19 reads its first cell wherever the formula sits (`AGGREGATE(15,0,E1:E6,G1:G2)` is the
  smallest, 1, in rows 1, 2 and 6, and `@` gives the same), and the array `{1;2}` lifts. So
  `SERIESSUM(MINVERSE(0.2),-AGGREGATE(15,Data!$C12,..),2,1E+15)` with C12 = -0.0001 is `#VALUE!` from the option
  (gen-m5/c634f6ec7b14fe08 Calc!A322).
  Evidence: probes/aggregate-negative-option-range-k (excel-graph, 2026-10-22, T-2381). Status: VERIFIED (excel-graph).
  Implemented in: crates/rustsheet-engine/src/functions/aggregate.rs (`aggregate_scalar`).

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
`Model::lift_values` (functions/lift/driver.rs), and the spill shape is `aggregate_static_result` in static_analysis/aggregate.rs (T-1038).

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
(`MULTINOMIAL(2.5,3)` = `MULTINOMIAL(2,3)`). A single argument is not 1 either: `(4)` 0.9999999999999998,
`(6)` 0.9999999999999982, `(50)` 1.000000000000018, `(170)` 1.0000000000001974. Argument order matters from
large sums on: `(20,150)` 5.221123511164219e+25 but `(150,20)` 5.221123511166111e+25, `(10,20,30)`
3.553261127085186e+24 but `(30,20,10)` 3.5532611270850594e+24; `(2,3)` = `(3,2)` and `(1,169)` = `(169,1)`.
The algorithm (fit, T-1190, T-1600): one double running sum adds `log10(k)` (correctly rounded) for k = 1..sum, then
subtracts `log10(k)` for k = 1..n for each argument, last argument first. With `y = sum * ln 10`, the result is `exp(y)`
when `y >= 0` and `1 / exp(-y)` when `y < 0`, which only a lone argument's residue (alone or padded with zeros) can be.
So a single argument below 1 is always 1 minus an even number of ulps (2^-53 each; the inverse of a value just above 1):
`(5)` 0.9999999999999996 where `exp(y)` would give the nearer 0.9999999999999994. That reproduces all 240 probed cells
bit-exactly; `exp(y)` alone gets 188 (every miss a single argument below 1). Rejected candidates, scored by T-1190
on the 72 cells it had: `exp(GAMMALN(s+1))` over exact factorials (Excel's own GAMMALN values, at most 17/72),
natural-log sums (at most 16/72), `10^x` in place of `exp(x * ln 10)` (30/72), the arguments in their given order
(60/72), glibc `log10` in place of a correctly rounded one (57/72), `ln(k)/ln 10` or `ln(k)*log10(e)` terms (at most
33/72), x87 80-bit terms or accumulator (at most 54/72). Scored by T-1600 on all 240: `10^y` (156/240), descending
subtraction (35/240), a separate denominator sum (21/240), `1 + expm1(y)` (184/240), `exp(y/log10 e)` (172/240), and a
`2^t` split at floor/round/ceil of `t = sum * log2 10` (at most 156/240).
Numeric text in a range cell after a negative is `#NUM!` (a range holding -1, "2", 3), as typed `(-1,"2",3)` is;
non-numeric text wins over a negative (`(-1,"x",3)` and `(D1:D1,"x")` with D1 = -1 are `#VALUE!`). Formatted numeric
text in a cell reads as its number, by direct reference and in a range alike (typed literals other than
plain "2" and "4" are unprobed): " 2" and "2 " are 2, "1e1" 10, "$3" 3, "50%" 0.5 (truncated to 0),
"(2)" -2 and "1,000" 1000 (so `#NUM!`), and date text "1/2/2000" 36527 (`#NUM!`, over 170).
Evidence: probes/math-argument-modes Sheet1!C9, C35, C47, C49:C53 (excel-graph, 2026-10-08);
probes/multinomial-large-arguments A1:A20 (2026-10-09); probes/multinomial-algorithm-sweep A1:A60 (excel-graph, 2026-10-15);
probes/multinomial-single-arguments A1:A170 (every single argument), B1:C13 (coercion; excel-graph, 2026-10-17).
Ours: the fitted algorithm (`multinomial_value`, functions/math_and_trigonometry/multinomial.rs), bit-exact on every probed cell.
Status: VERIFIED (values); algorithm a fit, 240/240 bit-exact
Tasks: T-1082, T-1190, T-1600

## COMBIN is inexact
Rule: Excel's COMBIN is not the exact binomial coefficient either, and its rounding differs from MULTINOMIAL's:
`COMBIN(10,4)` = 209.99999999999997 where `MULTINOMIAL(4,6)` = 209.99999999999977 (exact 210 for both);
`COMBIN(9,3)` 83.99999999999999, `COMBIN(15,3)` 455.00000000000006, `COMBIN(40,20)` 137846528819.99997, `COMBIN(100,50)`
1.0089134454556413e+29, `COMBIN(170,85)` 9.144841845131565e+49. It is symmetric (`COMBIN(n,k)` = `COMBIN(n,n-k)` bit for bit),
k = 0, 1 and 2 are always exact, and it stays finite up to `COMBIN(1029,514)` 1.4298206864989032e+308; `COMBIN(1030,514)` is `#NUM!`.
The algorithm (fit, T-1598): with `j = min(k, n-k)`, one double running product multiplies the rounded quotients
`(n-j+i)/(i+1)` for i = 1..j-1 in that order, and the result is that product times `n` (j = 0 gives 1). The numerators
pair with the denominators shifted by one: `COMBIN(9,3)` is `((7/2)*(8/3))*9`. This reproduces all 841 numeric cells
of probes/combin-algorithm-sweep bit-exactly. Rejected candidates (scores out of 841; the correctly rounded value scores
572): the textbook loop `r*(n-j+i)/i` (569), `r*((n-j+i)/i)` and the other natural pairings and orders over min, max or k
(at most 570), the same shifted pairing with `n` first (624) or over k instead of min(k, n-k) (636), the quotient not
rounded on its own (`r*(n-j+i)/(i+1)`, 569), and natural-log, log10 and lgamma sums (at most 6 of the 87 j = 3 cells).
Evidence: probes/multinomial-algorithm-sweep B1:B10 (excel-graph, 2026-10-15, T-1190); probes/combin-algorithm-sweep
A1:A844 (every k for n = 1..32, and small, n/3, n/2 and near-n k for n = 40..1030; excel-graph, 2026-10-17, T-1598).
Ours: the fitted algorithm (`combin`, functions/math_and_trigonometry/mathematical.rs). Every factor is at least 1, so the
product overflows within a few hundred factors for a large j, which bounds the work (directive 5).
Status: VERIFIED (values); algorithm a fit, 841/841 bit-exact
Tasks: T-1190, T-1598

## COMBINA is COMBIN(n+k-1, k)
Rule: COMBINA rounds exactly as COMBIN of `n+k-1` and `k` does ("COMBIN is inexact"): `COMBINA(10,10)` is 92378,
`COMBINA(20,19)` 35345263799.999985 and `COMBINA(1000,200)` 1.6676833067813678e+233, all three bit for bit (a running
product of `(n+i)/(i+1)` over k gives 92377.99999999999 and 35345263800 instead). `COMBINA(0,0)` is 1.
Evidence: ironcalc-calc/MATH_AND_TRIGONOMETRY/COMBIN_COMBINA.xlsx D4, D5, D16, and D11 for `COMBINA(0,0)` (Excel-cached,
Excel Online). A huge k is past the bound below (`COMBINA(2,1E300)` is `#NUM!`).
Ours: `fn_combina` calls `combin(n+k-1, k)`. Its loop runs min(k, n-1) times and stops at overflow; the bound below
keeps `n+k-1` under 2^31-1, so `COMBINA(1,1E10)` is `#NUM!` at once, as in Excel (probes/t2291 A5); the loop is bounded as a side effect.
Status: VERIFIED (3 cached cells; the rule is a fit to them)
Tasks: T-0905
Bound (T-2162): a large `number` is `#NUM!`: `COMBINA(9007199254740992,1)`, `COMBINA(6227020800,10)` (`FACT(13)`),
`COMBINA(1E+15-0.0001,2)`, `COMBINA(1000000000000107,..)` and `COMBIN(1E+15,0)` (from `COMBIN(1E+15,1E-15)`) are `#NUM!`
(ours: the coefficient, e.g. 2.4156840630757255E+91), while `COMBINA(25,123456789.123)` and `COMBIN(123456789.123,10)` are
numbers. Cut (probes/t2199-text-length-and-combin-cuts A20-A37, B20-B34, excel-graph, T-2199): COMBIN is `#NUM!` when `number` >= 2^31-1
(2147483647), whatever `k` (0, 1 and 2 all `#NUM!` there; at 2^31-2 they are 1, 2147483646 and 2.3058430038449848E+18),
and `COMBIN(2^31+1, 2^31+1)` is `#NUM!`. COMBINA is `#NUM!` when `n+k-1` >= 2^31-1, not on `n` alone:
`COMBINA(2^31-1,0)` is 1 and `COMBINA(2^31-2,1)` is 2147483646, while `COMBINA(2^31-2,2)`, `COMBINA(2^31-1,1)`,
`COMBINA(2^31-1,2)` and `COMBINA(2^31-10,20)` are `#NUM!`; with k = 0 it is `#NUM!` from `n` = 2^31. Both agree with COMBINA(n,k) being COMBIN(n+k-1,k).
Evidence (excel-graph, 2026-10-20): gen-m5/34f69d858a39f497 Calc!A231, gen-m5/50d8a085335c05e6 Calc!A173,
gen-m5/e0087a1fb9a36188 Calc!A379, gen-m5/fc81c2f704bef138 Calc!A265, gen-m5/6b831f5e2a20bc34 Calc!A5; numbers from gen-m5/1b30e4d075b252ea Calc!A213,
gen-m5/c688ca172fe533dc Calc!A288. For a small `n` with a huge `k` the rule holds too
(probes/t2291-combina-small-n-huge-k A1:A7, excel-graph, 2026-10-23, T-2291): `COMBINA(1,2^31-1)`, `COMBINA(1,2^31)`,
`COMBINA(2,2^31-2)`, `COMBINA(2,2^31-1)`, `COMBINA(1,1E10)` and `COMBINA(2,1E300)` are all `#NUM!` (n+k-1 >= 2^31-1), and
`COMBIN(2^31-2,2^31-2)` is 1. Ours matches. Bounded from below too (probes/t2517-combina-below-cut-small-n A1:A4,
excel-graph, 2026-10-25, T-2517): with n+k-1 at 2^31-2 (A3: 2^31-3) the cell is a number, not `#NUM!`: `COMBINA(1,2^31-2)` is 1,
`COMBINA(2,2^31-3)` is 2147483646, `COMBINA(1,2^31-3)` is 1 and `COMBINA(3,2^31-4)` is 2.3058430038449848E+18 (each is COMBIN(n+k-1,k)),
so the cut is on n+k-1, not on k a little lower. Ours matches. Status: VERIFIED (excel-graph); implemented (`COMBIN_MAX_NUMBER` in
functions/math_and_trigonometry/mathematical.rs, T-2162).

## SEQUENCE accumulates its step
Rule: SEQUENCE adds `step` to the previous value in row-major order, not `start + i*step`: `SEQUENCE(5,1,10,2.2)` is 10,
12.2, 14.399999999999999, 16.599999999999998, 18.799999999999997 and `SEQUENCE(1,5.3,12.7,0.1)` is 12.7,
12.799999999999999, 12.899999999999999, 12.999999999999998, 13.099999999999998 (the product form gives 14.4, 16.6, 18.8,
13 and 13.1). In a 2-D result the sum runs on across rows, in row-major order: in `SEQUENCE(3,3,0.1,0.1)` the first
cell of row 3 is 0.7 (the product form gives 0.7000000000000001) and the last 0.8999999999999999, and
`SEQUENCE(2,4,10,2.2)` ends 25.399999999999995.
Evidence: ironcalc-calc/MATH_AND_TRIGONOMETRY/SEQUENCE.xlsx I7:I11, I14:M14 (Excel-cached, Microsoft Macintosh Excel);
probes/sequence-step-accumulation A1:A11 (excel-graph, 2026-10-19, T-0905).
Ours: `fn_sequence` (functions/math_and_trigonometry/sequence.rs).
Status: VERIFIED
Tasks: T-0905

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

## ASIN's formulation
Rule: Excel's ASIN is `atan(x / sqrt(y*(2-y)))` with `y = 1-x`, in doubles, where the square root rounds twice (to a
64-bit significand, then to a double) like Excel's SQRT. So the factor 1+x is formed as `2-y`. In -1..-0.5, rounding
`y = 1-x` drops the last significand bit of x (ties to even), so ASIN is not odd. 31 ± pairs of the sweep differ in the last
digit (`ASIN(0.09)` = 0.09012194501459526, `ASIN(-0.09)` = -0.09012194501459525; `ASIN(1E-8)` = 1.0000000000000002E-8,
`ASIN(-1E-8)` = -1E-8), and `ASIN(-0.999)` = -1.5260712396261655 is 11 ulp from the correctly rounded value, while
`ASIN(0.999)` = 1.526071239626163 is correctly rounded. There is no tiny-argument branch: `ASIN(±1E-13)` = ±1E-13 is the
formula's own value once the root rounds twice.
The formulation fits all 570 cells of probes/asin-argument-sweep (268 cells) and probes/asin-negative-band-sweep (302 cells: ±0.800..±0.950 at a 0.001 step). In
that band every positive argument fit `atan(x/sqrt((1-x)(1+x)))`. Every negative one with an even significand did too, while
69 of the 74 with an odd significand did not.
Other candidates over the 570 cells (bit-exact; fit lab `asin`, T-1797): the same with a plain `sqrt` 567, the engine before
T-1797 (`atan(x/sqrt((1-x)(1+x)))` with x itself below 2^-28) 468, without the cut 466, `atan(x/sqrt(1-x*x))` 408, libm `asin` 390,
`atan2(x, sqrt((1-x)(1+x)))` 388, `2 atan(x/(1+sqrt((1-x)(1+x))))` 340, `pi/2 - acos(x)` 217. Rounding `(1-x)`, `(1+x)`, the product, the
root and the quotient in doubles, at 64 bits or twice, in every combination with glibc `atan` or a correctly rounded one, never beat 218 of the sweep's 251
non-tiny cells (T-1797). T-1259 and T-1605 rejected more, all symmetric in x: x87 extended intermediates, an extended atan, and
table reductions `atan(c) + atan((q-c)/(1+qc))`.
Ruled out by probes/asin-negative-input-read (127 rows, excel-graph, 2026-10-18, T-1797): the asymmetry is not Excel's read of
stored negatives. `=A1` echoes every stored negative exactly, and `ASIN(-D1)` of the stored positive equals `ASIN(A1)` in every row,
as does `ASIN` of the formula literal. Excel's ATAN is odd and equals glibc `atan` on all 120 quotients of
probes/asin-atan-sign-source (columns B-E, excel-graph, 2026-10-18).
With a positive first argument, `ATAN2(s, x)` is glibc `atan(x/s)` of the double quotient: column G,
`ATAN2(SQRT((1-x)*(1+x)),x)`, matches it in all 120 rows, against 86 for libm `atan2` (the engine's; T-1930 covers the other quadrants).
ACOS shares the kernel: on all 240 ACOS cells of probes/asin-atan-sign-source (`ACOS(x)` and `ACOS(-x)`, H and I), Excel's ACOS
equals `PI()/2 - ASIN(x)` exactly. glibc `acos` matches 157 of them and `pi/2 - atan(x/sqrt((1-x)(1+x)))` 225 (T-0939).
Evidence: probes/asin-argument-sweep (excel-graph, 2026-10-15, T-1259); probes/asin-negative-band-sweep,
probes/asin-negative-input-read and probes/asin-atan-sign-source (excel-graph, 2026-10-18, T-1797); gen-m6/2f6b96385ed3facd Calc!A38
(excel-graph, 2026-10-09, T-1239); gen-m7/0e40208f51c2454c Calc!A27 and gen-m7/cb748b6a9b81fbb7 Calc!A38 (excel-graph, 2026-10-11, T-1275).
Implemented in: `excel_asin` (functions/math_and_trigonometry/mathematical.rs), with `extended_sqrt`, since T-1797. ACOS uses it
since T-0939 ("ACOS is PI()/2 - ASIN(x)"). The result depends on
the platform `atan`, and glibc's fits.
Status: VERIFIED (excel-graph)
Tasks: T-1259, T-1605, T-1797; ACOS: T-0939

## MULTINOMIAL of a huge argument
Rule: MULTINOMIAL is `#NUM!` once its arguments add up to more than 170, where `sum!` overflows a double. That holds
even when the ratio is small: `MULTINOMIAL(171)`, `(170,1)`, `(86,86)`, `(100,100)`, `(1000)`, `(2^31)`, `(2^53)`,
`(1E+15)`, `(1E+15,0)`, `(1E+15,1)`, `(3,1E+15)` and `(1E+300)` are all `#NUM!`. Sums up to 170 have values:
`(170)` 1.0000000000001974, `(169,1)` 170.0000000000337, `(85,85)` 9.144841845133712e+49, `(150,20)` 5.221123511166111e+25, `(100,70)` 6.491922168402309e+48. `(100,100)` is `#NUM!` though its exact value (about 9e58) fits a double, so the limit is on the sum. `(0)` and `(0,0)` are 1.
The non-integer values are the inexactness in "MULTINOMIAL is inexact" (fitted by T-1190).
Evidence: probes/multinomial-large-arguments A1:A20 (excel-graph, 2026-10-09, T-1197); gen-m5/21b9ae9affd6b38c Calc!A25.
Status: VERIFIED (excel-graph)
Implemented in: `fn_multinomial` (functions/math_and_trigonometry/multinomial.rs) checks the saturating sum against 170
before any factorial, so the work is bounded (directive 5).
Tasks: T-1197

## FACT and FACTDOUBLE of a huge argument
Rule: FACT is `#NUM!` once n! overflows a double and FACTDOUBLE once n!! does: `FACT(170)` 7.257415615308004e+306 (`FACT(170.9)` the same, the argument is floored), `FACT(171)`, `(2^53)`, `(1E+15)`, `(1E+300)` `#NUM!`. `FACTDOUBLE(297)` 1.2552756225993057e+304, `(298)` 2.718138023126864e+305, `(300)` 8.154414069380598e+307, `(301)`, `(2^53)`, `(1E+15)`, `(1E+300)` `#NUM!`. Negative: `FACT(-0.5)` and `FACT(-1)` `#NUM!`; `FACTDOUBLE(-1)` is 1, `FACTDOUBLE(-1.5)` and `(-2)` `#NUM!`.
Evidence: probes/fact-large-arguments A1:A18 (excel-graph, 2026-10-09, T-1233); probes/fact-product-order-sweep (rounding, T-1249).
Status: VERIFIED (excel-graph)
Implemented in: `fn_fact`, `fn_factdouble` (functions/math_and_trigonometry/mathematical.rs) stop at the first infinite product, so the work is bounded (directive 5).
Rounding: Excel multiplies from n down (`n*(n-1)*...*2`, and `n*(n-2)*...` for FACTDOUBLE), one double running product, so its last digits
differ from both the ascending product and the correctly rounded value: `FACT(42)` is 1.4050061177528801e51 (correctly rounded 1.40500611775288e51,
ascending ...8798e51), `FACT(170)` 7.257415615308004e306 (ascending ...7994e306). The descending product matches all 420 cells of
probes/fact-product-order-sweep (FACT 0..170, and FACTDOUBLE for every n in 0..300 where the three candidates disagree; excel-graph, 2026-10-15,
T-1249); the ascending product matches 64 and the correctly rounded value 76. `DEGREES(FACT(42))` = 8.05009207373008e52 follows with no change to DEGREES.
Ours: matches since T-1249 (`descending_product`, mathematical.rs).
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

## BASE of zero with min_length 0 is empty; BASE past 2^53 is #NUM! (T-2373, T-2374)
Rule: BASE pads its digits to `min_length`, and a zero number has no digits of its own, so an explicit `min_length` that
truncates to 0 gives empty text: `BASE(0.2,29,0.3)` is "" (`LENB` of it is 0; ours "0"). Without `min_length` a zero is "0"
(`MID(BASE(0,22),1,MONTH(43194))` = "0"), and with `min_length` n it is n zeros (`BASE(0.1,15,9)` "000000000",
`BASE(0.1,24,10)`), which is what a default `min_length` of 1 would give.
A number past 2^53 is `#NUM!`: `BASE(ROUNDUP(2^53,-6),10,5)` (9007199255000000) and BASE of about 7.47E+19 (a SUM of products of
123456789.123) in radix 34 are `#NUM!` (ours "9007199255000000" and "3TDTK1V8J6TPP"; above 2^63 ours prints i64::MAX's digits).
The cut is exact: `number` of 2^53-1 is accepted (radix 2 prints 53 ones, radix 16 "1FFFFFFFFFFFFF", radix 36 "2GOSA7PA2GV"),
while 2^53, 2^53+2, 2^63, 2^64 and 1E+300 are `#NUM!` in radix 2, 16 and 36, with or without `min_length`; `BASE(-2^53,10)`
is `#NUM!` too (negatives are `#NUM!` anyway). `BASE(9007199254740993,10)` is accepted and
gives "9007199254740990" (15 significant digits, below 2^53); whether the cut happens on entry or inside BASE is unprobed.
Evidence (excel-graph, 2026-10-22): probes/base-number-upper-bound (T-2289); T-2252 triage: gen-m5/dde5ea295a38fb16 Calc!A254 (zero, min_length 0.3);
gen-m5/55afc805e00a1509 Calc!A101, gen-m5/08eafc9e61602bcb Calc!A98 (past 2^53); the "0" and zero-padded cells, which pass:
gen-m5/cd883013bea3b305 Calc!A22, gen-m5/9a7a94353845ea12 Calc!A45, gen-m5/90bf45bea539723d Calc!A251.
A written-empty `min_length` is an omitted one, but a blank cell is 0: `BASE(0,2,)` is "0" (`LEN` 1) and `BASE(5,2,)` "101",
while `BASE(0,2,Z99)` is "" (`LEN` 0), `BASE(5,2,Z99)` "101"; `BASE(0,2,"")` is `#VALUE!`; `BASE({0,5},2,{0,0})` is {"", "101"}.
Ours reads the written-empty argument as 0, so `BASE(0,2,)` gives "" (follow-up T-2691).
Evidence (excel-graph, 2026-10-25): probes/base-zero-empty-min-length (T-2416).
Status: VERIFIED (excel-graph) for the cells above; engine implements both halves (T-2373, T-2374) except a written-empty `min_length` (T-2691).
Tasks: T-2289, T-2373, T-2374, T-2416, T-2691

## RANDBETWEEN and RANDARRAY argument checks
Rule: RANDBETWEEN compares the raw `bottom` and `top` (`bottom > top` is `#NUM!`, even
`RANDBETWEEN(5,4.5)`), then draws a whole number from `CEILING(bottom)` to `FLOOR(top)`, and gives
`CEILING(bottom)` when `FLOOR(top)` is below it: `RANDBETWEEN(5.5,5.7)` and `RANDBETWEEN(5.5,6.2)`
are always 6, `RANDBETWEEN(-5.5,-5.2)` always -5. It rejects booleans (`RANDBETWEEN(TRUE,TRUE)` is
`#VALUE!`), reads number text (`"5"`), lifts over an array argument (one draw per element), and
either bound omitted (`RANDBETWEEN(,)`, `(,5)`, `(5,)`) is `#N/A`, not 0, and a span `top - bottom` that overflows a double is `#NUM!` (error precedence with an omitted bound next to an error argument is unprobed). RANDARRAY truncates `rows` and
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
T-1584 (excel-graph, 2026-10-15): a boolean reached through a reference is rejected like the literal (`RANDBETWEEN(J1,2)`, J1 TRUE, is `#VALUE!`: probes/volatile-invariant Calc!I1); RANDARRAY errors apply in argument order (`RANDARRAY(0,1,1.5,2.5,TRUE)` is `#CALC!`, I2); an omitted bound on either side is `#N/A`, not 0 (`RANDBETWEEN(,5)` and `RANDBETWEEN(5,)`, I5:I6, like `RANDBETWEEN(,)`). A RANDBETWEEN whose span `top - bottom` overflows a double is `#NUM!` (`RANDBETWEEN(-9*10^307,9*10^307)`, `RANDBETWEEN(-(10^308),10^308)`), while a span of 1E308 or equal bounds at +-1E308 return a value (probes/randbetween-huge-span A1:A7). RANDARRAY has no such error: with a span that overflows (`-(10^308)` to `10^308`, whole or not) it returns a finite number inside the bounds (B2:B3, B14:B15), and equal bounds at +-1E308 return that bound. `-10^308` is presumably `(-10)^308` (unary minus binds tighter than `^`), positive and a hair above 1E308, which is consistent with `RANDARRAY(1,1,-10^308,10^308)` `min > max`: `#VALUE!` (B12) and `RANDBETWEEN(-10^308,10^308)` is `#NUM!` (A8). A literal `1E308` pair in the volatile-invariant workbook made excel-graph refuse the file ("unsupportedWorkbook"), so those probes use `10^308`.
Status: VERIFIED. Implemented (T-1318, T-1584): A2, A6, A8, A12, A13, B9, B10, B18, I1:I6 and randbetween-huge-span pass. The array lift F1:F9 is implemented too (T-1316, `randbetween_draw` plus `lift_number_operands`).
Operands of the lift (probes/randbetween-lift-operands, 18 cells, all scalar-returning so nothing spills; excel-graph, 2026-10-16, T-1316): a boolean element of an array bound is an error element, not 1 or 0 (`COUNT(RANDBETWEEN({TRUE;1},{2;2}))` = 1, `SUM(...)` `#VALUE!`), like a scalar boolean; text elements read as number text (`{"5";"6"}` gives 2 numbers) and `"x"` or `""` elements are errors; a scalar error beside an array bound spreads over the broadcast shape (`ROWS(RANDBETWEEN(SEQUENCE(3),NA()))` = 3). A multi-cell range bound is `#VALUE!` as a whole (`RANDBETWEEN(D1:D3,10)`, even from a row the range intersects: no lift and no implicit intersection), so only array values lift, never ranges; a single-cell reference is a scalar.
T-1782 (probes/randbetween-bound-errors, 22 cells, excel-graph, 2026-10-17): a scalar `TRUE` or `FALSE` beside an array bound is `#VALUE!` in every element, in either position (`ERROR.TYPE(INDEX(RANDBETWEEN(SEQUENCE(3),TRUE),1))` = 3, likewise `FALSE`, `RANDBETWEEN(TRUE,SEQUENCE(3,1,5))`, `RANDBETWEEN({1;1},{TRUE;TRUE})`), not 1 or 0 (so `COUNT(RANDBETWEEN(SEQUENCE(3),TRUE))` = 0); `ROWS` keeps the broadcast shape. When both bounds are errors the **top's error wins**, whatever the kinds and whether lifted or scalar: `RANDBETWEEN({#N/A},{"x"})` is `#VALUE!` but `RANDBETWEEN({"x"},{#N/A})` is `#N/A`; `({#DIV/0!},{#N/A})` is `#N/A`, `({#N/A},{#DIV/0!})` is `#DIV/0!`; `({TRUE},{#N/A})` `#N/A`, `({#N/A},{TRUE})` `#VALUE!`; scalars `(#N/A,"x")` `#VALUE!`, `("x",#N/A)` `#N/A`, `(#DIV/0!,#N/A)` `#N/A`; a scalar `NA()` beside `{"x";"y"}` follows the same order. Per element, `({"x";#N/A},{#DIV/0!;"x"})` is `#DIV/0!` then `#VALUE!`. Ours: implemented (T-1782) by reading the top first in `fn_randbetween`.
T-1793 (probes/randbetween-range-error-bounds, 20 cells, excel-graph, 2026-10-17): a multi-cell range bound is an operand that is `#VALUE!` and takes its turn in the same top-first order: `RANDBETWEEN(D1:D3,NA())` is `#N/A` (the top error wins over the range), `RANDBETWEEN(D1:D3,1/0)` `#DIV/0!`, `RANDBETWEEN(NA(),D1:D3)` and `(1/0,D1:D3)` `#VALUE!` (the range top wins); a range beside text or a boolean is `#VALUE!` either way. Beside an array bound the range is broadcast, not a whole-result error: `ROWS(RANDBETWEEN(D1:D3,{5;6}))` = 2 (also with the range on top and for `SEQUENCE(4)`: 4), and per element the top's error wins, so `RANDBETWEEN(D1:D3,{5;#N/A})` is `#VALUE!` at row 1 and `#N/A` at row 2, `RANDBETWEEN({5;#N/A},D1:D3)` is `#VALUE!` at row 2 (A19; the range on top is always `#VALUE!`). The range counts as one scalar error, not by its own shape: `RANDBETWEEN(D1:D3,{5,#N/A})` is a 1x2 result whose element 2 is `#N/A` (A20). Ours: implemented (T-1793): a range bound is an error operand of `lift_number_operands`.

## SERIESSUM with x = 0
Rule: SERIESSUM with x = 0 (a literal 0 or a blank) is `#NUM!` when any term's power `n + i*m` is zero or negative, whatever its
coefficient (a 0 coefficient too), and 0 when every power is positive: `SERIESSUM(0,1,1,{1,2})` and `SERIESSUM(0,0.5,1,1)` are 0;
`SERIESSUM(0,0,2,{1,2,3})`, `(0,2,-1,{1,2,3})` (powers 2, 1, 0), `(0,1,-2,{1,2})` (1, -1), `(0,-1,1,1)`, `(0,-0.5,1,{5})`, `(0,0,1,{0})`,
`(0,1,-1,{5,0})` and `(0,-1,2,{5,6})` are `#NUM!`. With step 0, `(0,3,0,{1,2})` is 0 and `(0,0,0,{1,2})` is `#NUM!`.
`SERIESSUM(<blank>,123456789.123,-0.5,123456789.123)` is 0 (one positive power), so the IFS around it in gen-m7 goes on.
Evidence: probes/seriessum-x-zero A1:A22 (excel-graph, 2026-10-15, T-1359); ironcalc-calc MATH_AND_TRIGONOMETRY/SERIESSUM.xlsx C1, H7 (Excel-cached);
gen-m7/d38b52bfa1a9f9e6 Calc!A27 (excel-graph, 2026-10-11, T-1275).
Ours: implemented (T-1359) in `fn_seriessum`.
Precedence (probes/seriessum-x-zero-error-precedence E1:E27, excel-graph, 2026-10-15, T-1653; the engine matched every cell before T-1653): the coefficients are read left
to right and the first problem wins, so with x = 0 a bad-power term before a text or error coefficient gives `#NUM!` (`(0,0,1,{4,"a"})`), but a text or error coefficient
before or on the bad-power term gives `#VALUE!` or the error (`(0,0,1,{"a"})`, `(0,1,-1,{4,"a"})`, `(0,1,-1,{"a",4})`, `(0,1,-1,{4,#N/A})`, `(0,-1,1,{#N/A})`), the same as for x = 2
(`(2,0,1,{"a"})`). Between two bad coefficients the left one wins (`{#N/A,"a"}` is `#N/A`, `{"a",#N/A}` is `#VALUE!`, `{#DIV/0!,#N/A}` is `#DIV/0!`). A text or error `x`, `n` or `m`
(`SERIESSUM(0,"a",1,{1})`, `(0,NA(),1,{1})`, `(0,0,"a",{1})`, `("a",0,1,{1})`) gives `#VALUE!` or the error, with no `#NUM!`. A referenced range of coefficients is read in order like the array form. The probe's
range cells B1, C1 and D2 hold the text "4" (the spec string `"4"`), not the number, so `(0,0,1,B1:B2)`, `(0,1,1,C1:C2)` and
`(0,0,1,C1:C2)` are `#VALUE!` from their leading text cell and `(0,1,-1,D1:D2)` with D1 = #N/A is `#N/A` (T-2163 re-read; an earlier
note took them for "a range is not read in order"). A range read in order is also what gen-m5 shows: an overflowing term is
`#NUM!` before a later text or boolean cell ("SERIESSUM: blank coefficient cells are dropped" below). Other cell orderings in a
range (a number before an error cell) are unprobed.
Status: VERIFIED
Tasks: T-1359, T-1653

## SERIESSUM: blank coefficient cells are dropped; an overflowing term beats a text coefficient (T-2163)
Rule: a blank cell in the coefficient range is skipped, and the next coefficient takes its power:
`SERIESSUM(97,0,0.5,B11:E11)` with B11:E11 = 42, 2.5, blank, 2 is 42 + 2.5*97^0.5 + 2*97^1 = 260.6221445044903 (ours reads the
blank as a 0 term and gives 2*97^1.5, 1977.3005580529345); likewise `SERIESSUM(-0.5,1,3,B5:C6)` over blank, 1E+15, 1E+15, blank
is -437500000000000 and `SERIESSUM(16,-1,2,B11:E11)` over -1, blank, 42, 0.5 is 2719.9375.
A term that overflows is `#NUM!` even when a text, boolean or numeric-text cell of the range comes later:
`SERIESSUM(64,1E+15,0,E3:E12)` (E3 = 1, E4 = "1.5", E5 = "ü") and `SERIESSUM(-2.49995,0,5040,A4:A6)` (A6 = TRUE, the second term
overflows) are `#NUM!`; ours `#VALUE!`. That agrees with the x = 0 precedence above: the coefficients are read in order.
Edges (T-2218, probes/seriessum-blank-and-underflow, excel-graph 2026-10-22; the engine matches all 9 cells): an all-blank
multi-cell range has no terms: `SERIESSUM(0,0,1,J1:J2)` and `SERIESSUM(2,0,1,J1:J2)` (J1:J2 blank) are both 0, even at x = 0 with
n = 0 where 0^0 would be `#NUM!` (C8, C9). A blank one-cell reference is different: `SERIESSUM(0,0,1,F1)` is `#NUM!` and
`SERIESSUM(2,0,1,F1)` is 0 (C1, C2), consistent with the blank read as a 0 coefficient at power 0 (the cells cannot tell that
from Excel rejecting x = 0 with n = 0 before reading the coefficient). `SERIESSUM(0,0,1,G1:G2)` (blank, 4) and
`SERIESSUM(0,1,-1,H1:H3)` (4, blank, 5) are `#NUM!`, which a dropped blank and a 0 term both give. A negative power whose
reciprocal underflows, `SERIESSUM(1E-10,-33.5,1,1)` (the term would be 1E+335), is `#NUM!`, also with a later text element
(`{1,"a"}`), while the `^` operator on the same operands, `1E-10^-33.5`, is `#DIV/0!`.
The power is Excel's `^`: `SERIESSUM(1.005/42,3,1,C5)` = 1.3769443103134108E-05 and `SERIESSUM(-100.0001,42,2,42)` =
4.200176403616261E+85 equal `C5*(1.005/42)^3` and `42*(-100.0001)^42` bit for bit, where ours is an ulp off.
`SERIESSUM(LOG10(999999999999999),-2.5,1,10.005)` is 0.011481243964090428, a non-integer power: the old SERIESSUM `powf` gave
...431; through `arithmetic::power` (the `^` kernel on main at T-2163) the cell matches.
Evidence (excel-graph, 2026-10-20): gen-m5/34f69d858a39f497 Calc!A170, gen-m5/5704666097cbc5b2 Calc!A5, Calc!A267,
gen-m5/6b831f5e2a20bc34 Calc!A130, gen-m5/461d1c8dbb044769 Calc!A47, gen-m5/6a8bb7cd6d7769ad Calc!A373, gen-m5/ee1667411bbe2ccc Calc!A29,
gen-m5/1d22a002d4f380b3 Calc!A157.
Status: VERIFIED (excel-graph); implemented (T-2163) in `fn_seriessum`: blank cells are filtered out before the powers are
numbered, each term is `coeff * arithmetic::power(x, n + i*m)` and is `#NUM!` at once when the power fails or the term is not
finite. All 8 cells pass, and so does gen-m5/38b1b8e51dbad326 Calc!A10 (`EVEN(0.1/E5*SERIESSUM(-2.5,B10,3,0.1))`).

## SERIESSUM: a negative x to a non-integer power is a term of 0 (T-2380)
Rule: in SERIESSUM, a term whose power `x^(n+i*m)` has a negative x and a non-integer exponent (which `^` refuses with `#NUM!`)
counts as 0, and the read goes on to the later coefficients. So `SERIESSUM(-2,0.5,1,{1,2})` and `SERIESSUM(-2,0.5,2,{1,1})` are 0;
`SERIESSUM(-2,1,0.5,{1,1,1})` (powers 1, 1.5, 2) is -2+0+4 = 2. A later text or boolean coefficient is `#VALUE!` (`{1,"a"}`,
`{0,"a"}`, `{1,TRUE}`, and the same with a range of cells), and a later error is that error (`{1,#N/A}`); the first bad
coefficient wins (`{1,#N/A,"a"}` is `#N/A`, `{1,"a",#N/A}` `#VALUE!`). An overflowing term stays `#NUM!` at once, before or after a
zero term (`SERIESSUM(-10,0.5,399.5,{1,1,"a"})`, powers 0.5 and 400, is `#NUM!`; the T-2163 entry above).
So `SERIESSUM(-308641972.8075,30864197.28075,2,D1:E8)` is `#VALUE!` (gen-m5/bf8903e05ec369ca Calc!A326): its first term is 0 and the
read reaches the text "#N/A" in D2.
Unlike `^`, an odd root is a zero term too: `SERIESSUM(-8,1/3,1,{1})` is 0 (`(-8)^(1/3)` is -1.9999999999999998), and so is a
non-integer exponent past 2^32-1 (`SERIESSUM(-2,4294967295.5,1,{1,1})` is 0, with a later "a" `#VALUE!`), which `^` refuses.
Evidence: probes/seriessum-negative-x-fractional-power (25 cells, excel-graph, 2026-10-22, T-2380); gen-m5/bf8903e05ec369ca Calc!A326.
Implemented in: `seriessum_of` (crates/rustsheet-engine/src/functions/math_and_trigonometry/seriessum.rs, T-2380)
Status: VERIFIED (excel-graph)
Tasks: T-2380

## SUMXMY2 is 1 ulp above the correctly rounded sum on a 1.5E+16 result (T-2384)
Observation: `SUMXMY2(C10:E12,C6:E8)`, whose numeric pairs are (-1, 123456789.123), (7, 1E-15) and (-1, 0.3), is
1.5241579027474522E+16; the exact sum of the squared differences rounds to 1.524157902747452E+16, and so do the left-to-right
double sums of `(x-y)^2`, of `x^2-2xy+y^2` and of `x^2+y^2-2xy` (ours). `SUMX2-2SUMXY+SUMY2` gives ...518. So Excel's
accumulation is none of these; one cell cannot fit it.
Evidence: gen-m5/bf8903e05ec369ca Calc!A248 (excel-graph, 2026-10-22, T-2252 triage).
Sweep points (77 rows x SUMXMY2, SUMX2MY2, SUMX2PY2, excel-graph, T-2172): probes-numeric/trig-odd-even-sumxmy2-sweep Calc!J1:X77 (row 1 is the A248 pairs).
SUMX2MY2 and SUMX2PY2 match on all 77 rows. SUMXMY2 misses only V1, V3, V10, V12 and V16 (ours 1 ulp low). Excel's SUMXMY2 is order-dependent: rows 1-6 are the three A248
pairs in all six orders, and V1 and V3 are 1.5241579027474522E+16 while V2, V4, V5 and V6 are 1.524157902747452E+16 (row 16, x/y
swapped, gives ...522 like row 1; row 17 gives ...520); a plain sequential sum gives ...520 in every order, so no order-independent
accumulation fits. V10 and V12 (the big pair with (7,1E-15)) are ...520 where ours is ...518; V11 and V14 match. Every mixed,
cancelling and large-difference sum matches.
Status: OBSERVED; the T-2172 sweep holds the points, T-2384 fits them.

## ARABIC of a blank cell, and padded numerals (T-2153)
Rule: ARABIC of a blank cell is 0, as `ARABIC("")` is, whether the blank comes as a reference (`ARABIC(C3)`), through
`IF(B4,E5)`, or as the blank cell VLOOKUP or HLOOKUP lands on. Leading and trailing spaces are ignored: `ARABIC(" x ")` = 10,
also from `RIGHT(" x ",7)` and `INDEX(D6:D12,1,0)` with D6 = " x ". A lone tab is empty: `ARABIC(UPPER(CHAR(9)))` = 0.
Ours (since T-2153): as Excel; `fn_arabic` reads a blank (or a lookup's empty value) as `""` and trims space, tab, line feed and carriage return
from both ends. Probed (excel-graph, 2026-10-21, T-2200, probes/t2200-blank-and-padded-m5-args, Calc!A47:A70):
`ARABIC(0)` and `ARABIC(FALSE)` are `#VALUE!`, unlike a blank, which is 0; an inner space is `#VALUE!` (`ARABIC("X X")`,
`"X"&CHAR(9|10|13|160)&"I"`); `ARABIC(" ")` and a lone CHAR(9), CHAR(10) or CHAR(13) are 0; CHAR(9), CHAR(10) and CHAR(13)
padding either side of "XI" is trimmed (11); CHAR(160) is not whitespace: `#VALUE!` as padding and alone.
Evidence (excel-graph, 2026-10-20): gen-m5/1315170eef6c413e Calc!A37, gen-m5/399071ff6782ab16 Calc!A256, gen-m5/97737fad5004ba83 Calc!A43,
gen-m5/e0087a1fb9a36188 Calc!A25, gen-m5/6a511beadb56b932 Calc!A106, gen-m5/b0d597ec5af5fb92 Calc!A365, gen-m5/fc81c2f704bef138 Calc!A174 and the
other cells listed in T-2153.
Status: VERIFIED (excel-graph); implemented in `fn_arabic` (T-2153).

## ROMAN truncates before its sign check; BASE checks the sign first (T-2154)
Rule: ROMAN truncates its number before refusing a negative, so `ROMAN(-0.5025)` is "" (it truncates to 0); it reads and
checks its form even when the number is 0: `ROMAN(0,<#N/A>)` is `#N/A` and `ROMAN(0.2,8.09)` (form `LOG10(123456789.123)`,
above 4) is `#VALUE!`. BASE does the opposite: a negative number is `#NUM!` before truncation, so `BASE(-0.0001,5,4)`,
`BASE(-0.0001,7,10)` and `BASE(AVERAGE(-0.0001),9,1)` are `#NUM!`. Ours: ROMAN `#VALUE!`, "", "" and BASE "0000",
"0000000000", "0".
Evidence (excel-graph, 2026-10-20): gen-m5/49bfbf67a1409563 Calc!A179, gen-m5/0362d48f56108faa Calc!A39, gen-m5/1962de140dae0111 Calc!A206,
gen-m5/a6fec430ac350c1a Calc!A31, gen-m5/c0651f0e0fb544d0 Calc!A253, gen-m5/d1b251fd6e9c4b4b Calc!A24.
Which error wins when both are bad: see the T-2423 probe below.
ROMAN lifts over an array form, element by element: `TOROW(ROMAN(561,HSTACK(Data!B7:B7,{1,2;3,4},Data!A$6)),2)` is five "DLXI" (forms
1, 2, 0.5, 3, 4; the 123456789.123 element is `#VALUE!` and the HSTACK pads `#N/A`, both dropped by ignore 2: gen-m7/f7e94a817e5385ac
Calc!A1563, excel-graph, 2026-10-22). Implemented (T-2410): both parameters take the `Lift` kind (`LIFT_1_OF_2`), `fn_roman` runs per element. The array number, a multi-cell range and an array of booleans as the form are evidenced below (T-2423). Before T-2410 ours was `#N/IMPL!` (no lifting-table entry).
ROMAN probed (T-2423, probes/roman-lift-forms, excel-graph, 2026-10-23): `ROMAN({1;4},0)` is the 2x1 "I", "IV"; `ROMAN(4,Data!A1:A2)` (forms 0, 4) is "IV", "IV"; `ROMAN(Data!A1:A2,0)` (numbers 0, 4) is "", "IV"; `ROMAN(4,{TRUE,FALSE})` is "IV", "IV" and `ROMAN(4,TRUE)` is "IV". `ROMAN(499,{TRUE,FALSE})` is "CDXCIX", "ID" (TRUE is form 0, FALSE is form 4) and `ROMAN(499,Data!A1:A2)` (forms 0, 4) is "CDXCIX", "ID"; `ROMAN(499,{1,2,3,4})` is "LDVLIV", "XDIX", "VDIV", "ID" (the 4-based results above cannot tell the forms apart). When both are bad the form's error wins over a negative number for `ROMAN(-5,NA())` (`#N/A`; ours `#VALUE!`, T-2523), and the text form `ROMAN(-5,"x")` is `#VALUE!`; `ROMAN(4000,9)` is `#VALUE!`.
COMBINA lifts over arrays (T-2414, probes/t2414-gen-m7-unexplained, excel-graph, 2026-10-23): `COMBINA({7;1},4)` is the 2x1 209.99999999999997 (COMBIN(10,4), as above), 1; `COMBINA(Data!E7:E8,4)` and `COMBINA(4,{1;2})` are
2 rows; `COMBINA(WRAPROWS(E6:E12,1,1),4)` is 7x1: #NUM! (blank is 0), 209.99999999999997, #NUM!, 5, #VALUE! ("abc"), #NUM!, #VALUE!. The gen-m7 `#VALUE!` of
`VSTACK(COMBINA(..),CHOOSE(3,..))` (f19581ac44da1653 A3357) is the CHOOSE: a scalar-error argument makes VSTACK that error (probes/vstack-scalar-error, T-1695)
(`VSTACK({1;2},CHOOSE(3,1,2))` is `#VALUE!`, ours agrees). Implemented (T-2453): COMBINA lifts both arguments.
Status: VERIFIED (excel-graph); implemented (T-2154).

## QUOTIENT, FACTDOUBLE and SERIESSUM argument modes
Rule: QUOTIENT (both arguments), FACTDOUBLE and SERIESSUM's `x`, `n` and `m` are Analysis ToolPak arguments. They lift
over an array, one scalar call per element, and refuse a multi-cell range with `#VALUE!`, even on the range's own rows.
SERIESSUM's coefficients, and every argument of MULTINOMIAL, GCD and LCM, are read whole, over an array as over a range.
- Lifts: `INDEX(QUOTIENT({7;9},2),2)` = 4, `INDEX(FACTDOUBLE({5;6}),2)` = 48 and `INDEX(SERIESSUM({1;2},0,1,{1,2}),2)` = 5,
  with ROWS 2 each (C1-C6). The later arguments lift too: `QUOTIENT(10,{3;4})`, `SERIESSUM(2,{0;1},1,{1,2})` (element 2
  = 10) and `SERIESSUM(2,0,{1;2},{1,2})` (element 2 = 9) (C7-C12). A column `x` against a row `n` is a 2×2 grid (C15-C16).
- Coefficients whole: `INDEX(SERIESSUM({1;2},0,1,B1:B3),2)` = 17 beside a lifted `x`, with ROWS 2 (C13-C14);
  `ROWS(SERIESSUM(2,0,1,{1;2;3}))` = 1 (C40). A blank coefficient cell is dropped, not read as 0: the coefficients after it move up one power (C44, a trailing blank, cannot tell the two apart; "SERIESSUM: blank coefficient cells are dropped" below).
- Elements: an element error stays in its position (`QUOTIENT({7;9},{0;2})` is `#DIV/0!`, 4; `FACTDOUBLE({5;-2})` 15,
  `#NUM!`; C17-C20), short arrays pad with `#N/A` (C33), and a scalar error beside an array is broadcast
  (`QUOTIENT(NA(),{1;2})` is 2 rows of `#N/A`; C25-C26). Inside one element the leftmost error wins:
  `QUOTIENT({#N/A},{#DIV/0!})` is `#N/A`, the reversed one `#DIV/0!`, SERIESSUM's `x` error comes before a coefficient error
  and before an `n` error (C29-C32).
- A range beside an array: `ROWS(QUOTIENT(A1:A3,{1;2}))` = 2, every element `#VALUE!` (C27-C28). An operation on a range is
  an array and lifts (`ROWS(QUOTIENT(A1:A3+0,2))` = 3, C41-C42); a one-cell reference is its value (C43).
- Booleans: FACTDOUBLE refuses a boolean in every form, typed (`FACTDOUBLE(TRUE)`, `(FALSE)`), in a cell, or as an array
  element (`{TRUE;5}`, `{FALSE;5}`): all `#VALUE!` (-b E1-E3, E8; C22). Numeric text is read (`FACTDOUBLE("5")`, a text
  cell "5", `{"5";5}`: 15) and a blank cell is 0 (`FACTDOUBLE(<blank>)` = 1). QUOTIENT's and SERIESSUM's booleans are
  `#VALUE!` the same way (C21, C24, -b E11-E15).
- MULTINOMIAL, GCD and LCM over an array give one value: `GCD({4;6},2)` = 2, `LCM({4;6},2)` = 12,
  `MULTINOMIAL({1;2},2)` = 29.99999999999999, all ROWS 1 (C34-C39).
Evidence: probes/math-atp-array-lift C1-C44 and probes/math-atp-array-lift-b E1-E18 (excel-graph, 2026-10-17, T-1802);
probes/scalar-lift-open-a Y19-Y30 (the range refusals, T-1777).
Ours: implemented (T-1802): QUOTIENT and FACTDOUBLE are `LiftArrays` entries of the SPEC-0011 table, SERIESSUM's
`x`, `n`, `m` too with its coefficients `WholeByCaller` (read once by `series_coefficients`, each element charged their
cells); MULTINOMIAL, GCD and LCM have no entry. Before T-1802 FACTDOUBLE read a boolean as 1 or 0 and lifted over a range,
and QUOTIENT and SERIESSUM's `x`, `n`, `m` intersected a range and were `#N/IMPL!` over an array.
A one-cell 3-D range (`Sheet1:Sheet2!A1`) is `#VALUE!` in QUOTIENT's first argument, FACTDOUBLE's, SERIESSUM's `x` and
SERIESSUM's coefficients (`SERIESSUM(2,0,1,Sheet1:Sheet2!A1)`; ours was `#REF!` there until T-1832, now `#VALUE!` in
`series_coefficients`), and a legacy bare range, `=QUOTIENT(A1:A3,2)` or `=FACTDOUBLE(A1:A3)`, gets no `@` (FORMULATEXT shows it
as written) and is `#VALUE!` (probes/quotient-factdouble-seriessum-3d-legacy C1-C8, excel-graph, 2026-10-19, T-1832).
Status: VERIFIED (excel-graph)
Tasks: T-1777, T-1802, T-1832

## ECMA.CEILING
Rule: `ECMA.CEILING(number, significance)` is a real Excel function with exactly the two-argument `CEILING` semantics (rounds away from zero for negative `number` with negative `significance`; `#NUM!` when `number` is positive and `significance` negative; zero significance gives 0; number text `"2.5"` and `TRUE` are read as numbers; text significance is `#VALUE!`). It is stored in a file as `_xlfn.ECMA.CEILING`, and `FORMULATEXT` prints it as `=ECMA.CEILING(2.5,2)` (no prefix). Written bare (`ECMA.CEILING(...)`) in a file, Graph rejects the whole workbook (`createSession` 501 `unsupportedWorkbook`); the one-argument form `_xlfn.ECMA.CEILING(2.5)` is rejected the same way (arity is exactly 2, like CEILING).
Evidence: probes/ecma-ceiling A1:A6, A8:A11, A13:A14; probes/ecma-ceiling-formulatext A12 (excel-graph, 2026-10-12, T-1301).
Status: VERIFIED. Ours: implemented (T-1301) as its own `Function::EcmaCeiling` (not an alias of CEILING, so FORMULATEXT keeps the name) dispatched to `fn_ceiling`, with an `_xlfn.ECMA.CEILING` entry in `functions/xlsx_names.rs`. The name is `ECMA.CEILING` in every `language.bin` locale (as upstream IronCalc 8a82396a has it).
Tasks: T-1301

## RADIANS, SQRTPI and MUNIT
Rule: all 94 cells of probes/radians-sqrtpi-munit agree with `x * (PI / 180)` for RADIANS and `sqrt(x * PI)` for SQRTPI to the last bit (`RADIANS(180)` = PI(), `RADIANS(1E-300)`, `SQRTPI(1E-155*1E-155)` and `SQRTPI(10^308)` included), so there are no ulp differences to file.
- **SQRTPI rejects booleans, RADIANS reads them.** `SQRTPI(TRUE)`, `SQRTPI(FALSE)`, `SQRTPI(1=1)`, a reference to a TRUE cell and `SQRTPI({TRUE})` are `#VALUE!`, while `RADIANS(TRUE)`, `RADIANS(D1)` and `RADIANS({TRUE})` are 0.01745...; numeric text (`SQRTPI("4")`) is read, `-TRUE` is a number (`SQRTPI(-TRUE)` is `#NUM!`). In an array the boolean element is the error: `SQRTPI({1,TRUE})` starts with 1.77... (the second element is not probed; we give `#VALUE!`). A negative argument is `#NUM!`.
- **A multi-cell range is one `#VALUE!`, not a spill.** `SQRTPI(D1:E1)` (D1 TRUE, E1 empty) is a single `#VALUE!` (probes/radians-sqrtpi-munit A80, no B80), and so is every multi-cell range whatever it holds: two numbers (`H1:I1`), a boolean and a number, text and a number, a vertical pair, two blanks, a 2-D block (probes/sqrtpi-range-operands, excel-graph, 2026-10-16, T-1663). An array constant lifts: `SQRTPI({4,9})` is 3.54..., 5.31... and `SQRTPI({4,TRUE})` is 3.54..., `#VALUE!`. A one-cell range such as `H1:H1` is unprobed (ours: a scalar). A reference a function returns is refused like a written range (`SQRTPI(TAKE(E1:E3,2))` and `ROWS(SQRTPI(DROP(E1:E3,1)))` are `#VALUE!`), a 3-D reference is `#VALUE!`, and a legacy `SQRTPI(E1:E3)` takes no `@` (`#VALUE!` on the range's rows), as for the other Analysis ToolPak functions (probes/date-math-lift-driver-edges G21-G23, G28, K2; T-1785). Ours: `fn_sqrtpi`.
- **MUNIT truncates its dimension** (`ROWS(MUNIT(2.9))` is 2), is `#VALUE!` below 1 (`MUNIT(0)`, `MUNIT(0.5)`, `MUNIT(-1)`) and for text that is not a number, and reads numeric text and TRUE as 1.
- **MUNIT lifts over an array or range argument**: each element gives the 1 at the top left of its own matrix, so `MUNIT({1,2})` is {1,1} (`ROWS` 1, `COLUMNS` 2, `SUM` 2), `MUNIT({2;3})` is 2 rows by 1 column, and an element of 0 or text makes that element, and so `SUM`, `#VALUE!` (`INDEX(MUNIT({2;0}),2)` is `#VALUE!`, `INDEX(MUNIT({2;TRUE}),2)` is 1, `ROWS(MUNIT(E1:E2))` is 2 over a range; probes/date-math-lift-driver-edges G33-G35). A 3-D reference is `#REF!`, as in a worksheet function's lifted argument (G29). A one-cell range or one-cell reference is its value, not a 1x1 array: `ROWS(MUNIT(E1:E1))` and `ROWS(MUNIT(OFFSET(E1,0,0)))` are 5 with E1 = 5, the whole matrix (N1-N2).
- excel-graph rejects the workbook (`createSession` 501 `unsupportedWorkbook`) for a literal `1E+308` (as numbers.md "Formula literals of 1E+308 or more" says) and also for the denormal literals `1E-310` and `-1E-310`; the probe computes them (`1E-155*1E-155`). `MUNIT(1048577)` and `MUNIT(1E+10)` were left out of the case for the huge-dimension reason in text.md, unprobed.
Evidence: probes/radians-sqrtpi-munit (excel-graph, 2026-10-13, T-1282).
Status: VERIFIED. Ours: `fn_sqrtpi` (mathematical.rs) is a `LiftArrays` table entry of the SPEC-0011 driver whose kernel reads booleans as errors; `fn_munit` (matrix.rs) reads its argument with `read_lift_arg` and lifts an array or range through `lift_values` (it has no table entry: its scalar result is a matrix, so its static shape stays unknown; T-1785).
Tasks: T-1282, T-1785

## Trig of a huge angle
SIN, COS, TAN, COT, SEC and CSC return `#NUM!` when |x| >= 2^27 (134217728); 2^27-1 still gives a
number, and the cut is the same for negative arguments and for 1E+300. The error is a real error
(`ISERROR(N(SIN(1E+15)))` is TRUE). Evidence: probes/trig-huge-argument (T-1201). T-1201 probed only the integers 2^27-1 and 2^27 (and their negatives) at the cut; T-2172 adds fractions: the
cut for fractional arguments is also |x| >= 2^27: 134217727.25, .5, .75 and 2^27-2^-26 give numbers, 134217728.25 and -2^27 are `#NUM!`, negative fractions below 2^27 in magnitude give numbers (-134217728.25 itself unprobed) (probes-numeric/trig-odd-even-sumxmy2-sweep Calc!A131:D142, T-2172). COSH(1E+15) is also
`#NUM!` (probe cell A85).
Excel reduces every trig argument by multiples of `π₆₄/2`, where `π₆₄` is π rounded to a 64-bit
significand (the x87 `FLDPI` constant `0xC90FDAA22168C235·2^-62`, π₆₄ − π ≈ 5.0166e-20), and
then rounds the true sine, cosine or tangent of that remainder once. So SIN(x) is the correctly
rounded sin(x + n·δ/2), n = round(2x/π), δ = π − π₆₄. Visible consequences: SIN(PI()) is
1.22514845490862E-16 (libm: 1.2246467991473532E-16), COS(PI()/2) is 6.1257422745431E-17 (the
dropped tail of π₆₄/2), TAN(PI()/2) is 1.6324552277619072E+16 (IEEE: 1.633123935319537E+16), and
near the cut the 10th-12th digit moves (SIN(2^27-1) is -0.9559933287726706, libm
-0.9559933287732995). SEC, CSC and COT are 1 divided by the *rounded* COS, SIN and TAN: COT is
1/TAN, not COS/SIN (COS/SIN misses 17 cells of m9-trig-reduction by 1 ulp, e.g. F6 COT(PI()+1E-06)
= 999999.9999824036), and SEC/CSC from the unrounded cosine/sine miss 5. This model reproduces all
672 numeric cells of probes-numeric/m9-trig-reduction and probes/trig-huge-argument bit for bit
(T-1563). Reducing by π₆₄ itself (shift round(x/π)·δ) misses 137 of them, at the odd multiples of
π/2. Whether the final division of SEC/CSC/COT rounds once or twice (x87 extended) is not
distinguished by any probed cell.
TAN misses this model by 1 ulp at arguments near 1.2E+8: `TAN(123456789.123)` is 56.41575748007382 (model and ours
56.415757480073815, also `SIN(x)/COS(x)`) and `TAN(123456788.123)` is 0.6173408433901317 (ours ...318). Evidence (excel-graph,
2026-10-20): gen-m5/d0419fa09d6a756a Calc!A311, gen-m5/f0f205865a3a4328 Calc!A227, gen-m5/c688ca172fe533dc Calc!A284. T-2172 sweep (145 arguments over 1E+6 .. 2^27, probes-numeric/trig-odd-even-sumxmy2-sweep Calc!A1:D145): SIN and COS match the
pi64 model on all 145 cells; TAN misses only B121, B123, B126 and B130 (+-123456788.123 and +-123456789.123, 1 ulp), its other 141 cells
pass. So the miss is TAN-only and confined to arguments near 1.2E+8 (T-2147 fits).
T-2147 fit: Excel's TAN is the pi64-reduced tangent rounded twice, to a 64-bit significand and then to a double, RN53(RN64(tan)),
as an x87 tangent is stored. That model matches every one of the 311 earlier TAN cells with a constant argument (gen-m5, gen-m6,
probes-numeric/trig-odd-even-sumxmy2-sweep, m9-trig-reduction and the others), the 5 misses above included, where the single
rounding misses 5. probes-numeric/t2147-tan-double-rounding (excel-graph, 2026-10-25, filled twice with identical results) holds
150 arguments picked where the tangent lies near a 64-bit tie, so the candidates disagree: RN53(RN64(tan)) matches 129, the
correctly rounded tangent 47, and the quotient of RN64 sine and cosine rounded to 64 bits (an FPTAN-style result) 63. Guard-bit
variants (the tangent rounded or truncated to 65..72 bits first) match 118..134, so none is better beyond noise. The 21 misses are
not all at the tightest ties: x = 8242 lies 0.04 of a 64-bit ulp from a 64-bit value that is a 53-bit midpoint, and Excel still
rounds it the other way. So Excel's own tangent carries an error up to about half a 64-bit ulp, and its last bit at such a near-tie
is not predictable from the exact value. Those 21 cells stay known failures of that probe (as decision 0112 keeps LN's near-ties).
Ours: implemented (`excel_tan` in functions/math_and_trigonometry/trig.rs, T-2147). COT, which is 1/TAN, inherits the change.
COT at the same 150 arguments (probes-numeric/t2664-cot-double-rounding, excel-graph, 2026-10-26, T-2664): ours, `1/` the
double-rounded `excel_tan` (quotient rounded twice), matches 131 of 150 Excel cells, against 129 for TAN itself, and every miss is 1 ulp
(rows 8, 22, 49, 58, 59, 66, 73, 81, 106, 107, 111, 115, 128, 130, 132, 135, 140, 145, 149). So COT inherits TAN's double rounding
and its unpredictable near-tie noise; no alternative COT model was fitted (no arbitrary-precision tool was at hand), and with TAN's own
misses at this level a better fit would be noise-level. The 19 misses stay known failures of that probe.

## Out-of-domain ACOS/ASIN/ACOSH/ATANH are `#NUM!` at once, even nested (T-1202)

An argument outside the domain of ACOS, ASIN, ACOSH or ATANH gives `#NUM!` as the function's own
result, so an enclosing function never sees a number: `SERIESSUM(ACOS(1E+15),0,...)` is `#NUM!`
(not `x^0 = 1`), `ADDRESS(ACOSH(-1),4)` is `#NUM!` (not `#VALUE!` from row 0),
`CHOOSECOLS(0,ASIN(100))` is `#NUM!`, and `x<ASIN(PI())` is `#NUM!` (not FALSE).
Evidence (excel-graph): gen-m5/cd883013bea3b305 Calc!A23, gen-m6/3ee7ffc8d6ab061f Calc!A26,
gen-m7/2bb922eef72e785e Calc!A30, gen-m7/93976dd9a3578ed9 Calc!A13. Implemented in
`Model::number_fn` (functions/math_and_trigonometry/mathematical.rs), which maps any NaN result to
`#NUM!`; ATANH is covered by the same helper, not by its own oracle cell (except +-1, below).

Just below -1 as well: `ASIN(-1-2^-52)` and `ACOS(-1-2^-52)` (-1.0000000000000002) are `#NUM!`, also nested
(`LOG(ACOS(-1-2^-52))` is `#NUM!`, and `COUNT(..,ASIN(-1-2^-52))` does not count it). Ours: `excel_asin` forms `1-x`, which
rounds to exactly 2, so the radicand `y*(2-y)` is 0 and the result is -pi/2 (ACOS pi) instead of NaN. Evidence (excel-graph,
2026-10-20): gen-m5/38b1b8e51dbad326 Calc!A222, gen-m5/3b813881f835cd90 Calc!A273, gen-m5/90bf45bea539723d Calc!A236,
gen-m5/b0ecb7ca78460b77 Calc!A320 and the other 10 cells listed in T-2170. Implemented (T-2170): `excel_asin` returns NaN for `|x| > 1` before the kernel.

ATANH(1) and ATANH(-1) are `#NUM!` too, at once: `IFERROR(ATANH(1),"err")` is "err", `ISERR(ATANH(1))` TRUE,
`SUM(ATANH(1),1)` `#NUM!`; ATANH(0.9999999999999999) is 17.616361586450413 and ATANH(1.0000000000000002) `#NUM!`.
ACOTH(1) and ACOTH(-1) are `#NUM!`. Evidence: probes/atanh-unit-argument (excel-graph, 2026-10-17, T-1569).
`f64::atanh` returns an infinity at +-1 (not NaN), so ATANH has its own check beside the macro's NaN mapping.

## SUBTOTAL 6 over no numbers is 0 (T-2269)
Rule (excel-graph, gen-m6, 2026-10-21): SUBTOTAL's PRODUCT (function_num 6) of ranges holding no numbers is 0, as `PRODUCT` is:
`DATE(2071,6,SUBTOTAL(6,Data!C5:C7))` over "?b", "abc", "abc" is 62609 (`DATE(2071,6,0)`) (gen-m6/216e3c05f407ca79 Calc!A21).
Function_num 106 is 0 too (excel-graph, probes/subtotal-106-no-numbers, T-2302): over text only
(Hide!C1), blanks only (Hide!C2, with `SUBTOTAL(6,...)` as control in C3), several such references (C7),
and over ranges whose only numbers sit in manually hidden rows (Hide!C4, C6, C8, C9; the
function_num 6 control C5 still multiplies the hidden 3 and 5 to 15) or in rows a table filter hides
(Filter!C2, C3; function_num 6 skips filtered rows too, so Filter!C1 is 0). AGGREGATE's PRODUCT
(function 6) gives 0 the same way under options 1, 4, 5 and 6 (Hide!C10-C12, Filter!C4): an empty
product is 0 whenever no number is included, never 1.
Ours: implemented (T-2269); 106 and the AGGREGATE cells share the code path and match.
Status: VERIFIED
Tasks: T-2269, T-2302
