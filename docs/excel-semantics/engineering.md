# Engineering functions

## BESSELJ / BESSELY (T-0900)

Evidence: Excel-cached cells in `ironcalc-calc/ENGINEERING/BESSEL.xlsx`
(`make eval-show CASE=ironcalc-calc/ENGINEERING/BESSEL.xlsx`).

- For `|x| < 8` Excel is **not** fdlibm. Its values (accurate only to ~1e-8, e.g.
  `BESSELJ(0,0) = 1.00000000283141`) are bit-identical to the Numerical Recipes rational
  approximations `bessj0/bessj1/bessy0/bessy1` (with the truncated constant `0.636619772`),
  upward recurrence for higher orders, and Miller downward recurrence for `J_n(x)` when `n >= x`.
  `Y_n` recurrence is only bit-exact as `b * (2j * (1/x)) - a`.
  Implemented in `functions/engineering/transcendental/bessel_excel.rs`.
- For `|x| >= 8` (cells at x = 10 and 32) Excel differs from both fdlibm and the NR asymptotic
  branch (e.g. `BESSELJ(10,1)` = 0.04347224965 vs true 0.04347274617), so another algorithm is
  used there. UNRESOLVED; tracked in T-0908. fdlibm is kept for `|x| >= 8`.

Sweep (T-1004, `probes-numeric/m9-bessel`, excel-graph; 364 cells = 308 grid cells (14 x values in 8..100 x
orders 0-10, both functions) + 24 fractional-order + 32 large-order cells):
- For x in [8,100] Excel is NOT uniformly off by one amount; its relative error against the true value
  (measured with the engine's fdlibm value as the reference) has this profile, so fit against the cells, not a band:
  BESSELJ is worst near x = 8..10 where 1 <= order < x (up to 7e-5, e.g. `BESSELJ(8,1)` = 0.2346283865070744
  vs true 0.2346363468539146), falls to ~2e-7 at x = 30 and up to about 3e-9 at x >= 50; BESSELJ with order >= x is
  nearly exact (5e-15 for `BESSELJ(8,10)`, up to 5e-10 elsewhere), which fits Miller downward recurrence for
  n >= x and upward recurrence from an inaccurate J1 below. BESSELY stays at about 2e-7 or below (worst at
  x = 10; 6e-9 at x = 8, 3e-9 at x = 100). Values are at the oracle sidecar's cells, e.g. `BESSELJ(30,1)` =
  -0.11875106091123117, `BESSELY(8,0)` = 0.22352148924968093.
- At x = 10 the sweep equals the Excel-desktop cached values of `ironcalc-calc/ENGINEERING/BESSEL.xlsx`, at the orders the workbook covers (0, 1, 7, 10).
- The engine's fdlibm fallback is bit-exact on only 14 of the 364 cells; `rustsheet-eval show` lists the other
  350 as non-exact (164 within tolerance, 184 beyond it, 2 type mismatches).
- A fractional order truncates: `BESSELJ(8,9.99)` equals `BESSELJ(8,9)`, `BESSELY(30,1.99)` equals `BESSELY(30,1)`.
- Large orders: `BESSELJ(x,n)` is `0` (not an error) for n = 1000, 10000, 100000 and 1000000 (x = 10, 50), and
  `BESSELY(x,n)` is `#NUM!` for those n. The engine matches this up to n = 100000 but gives `#NUM!` for
  `BESSELJ(10,1000000)` (see "BESSEL order cap"). J underflows to 0, so no cap shows for J; Y is `#NUM!` from n = 1000 (where Y starts to give `#NUM!`
  between 101 and 1000 is not probed).
Status: VERIFIED for these cells; fitting the algorithm is T-0908.

Provenance: the coefficients are the published Numerical Recipes (2nd ed., §6.5) rational
approximations, reproduced only because Excel's output matches them bit-for-bit.

## BESSEL order cap
Rule: RustSheet returns `#NUM!` for BESSELI, BESSELJ, BESSELK and BESSELY orders above 100,000 to bound per-cell work on
untrusted input (CLAUDE.md directive 5). Excel gives `BESSELJ(10,1000000)` = `0` (probes-numeric/m9-bessel, T-1004), so the engine differs above 100,000 for J.
BESSELI and BESSELK took the cap in T-1383 (before it, `BESSELI(2.5,2E9)` looped about 4·10^9 times); Excel's values above
100,000 are unprobed for them.
A lifted Bessel call is also charged 2^16 units per element against the lifting visit cap (SPEC-0011
`KernelWork::Weighted`), so one call lifts at most 2,048 elements: `ROWS(BESSELJ(2.5,SEQUENCE(2100)))` is `#ERROR!`
where Excel gives 2100. At the order cap a BESSELI element runs about 2·10^5 recurrence steps; 2,048 such elements
take about 2 s in a release build (8,192 took 8 s, T-1383).
Evidence: probes-numeric/m9-bessel (T-1004) shows Excel's behavior for huge orders; the cap itself is a resource limit, not an Excel observation.
Status: DEVIATION (decision 0014)
Tasks: T-1004, T-1383; T-1819 narrows the lifted cap to an order-based work budget.
Implemented in: functions/engineering/transcendental/bessel_excel.rs (`MAX_ORDER`), bessel_i.rs, bessel_k.rs;
functions/lift/kinds.rs (`BESSEL_WEIGHT`)

## DECIMAL of unreadable text
Rule: DECIMAL reads its first argument as text (a number or TRUE goes through its text form, so `DECIMAL(0.5,13)`, `DECIMAL(1E+20,10)` and `DECIMAL(TRUE,10)` read "0.5", "1E+20", "TRUE"). Leading spaces and tabs are skipped; empty or all-blank text is 0. Any other unreadable character is `#NUM!` (not `#VALUE!`): a sign (`"-1"`, `"+1"`), a point, trailing space, a non-breaking space, a digit not below the radix. Text over 255 UTF-16 units (200 emoji is too long) is `#VALUE!`, checked before the radix (`DECIMAL(REPT("1",256),37)` is `#VALUE!`), as is a non-numeric radix. Newline and fullwidth digits are unreadable (`#NUM!`); the overflow is `#NUM!` inside a larger formula too (`ERROR.TYPE` 6, `IFERROR` falls through). Error arguments: the text argument's error wins (`DECIMAL(#N/A,#DIV/0!)` is `#N/A`). Radix 16 only also skips leading zeros and one `x`/`X` (`"0x1F"`, `"x1F"`, `"00x1F"` are 31; `"0x0x1"`, `"1x1F"`, `"0x 1F"` are `#NUM!`; in radix 10 or 15 `"0x1F"` is `#NUM!`, in radix 36 the x is a digit). The sum is a float with no 2^53 limit (255 ones in radix 2 is 5.78960446186581E+76; 198 `Z`s in radix 36 is 1.4057081148316923E+308), but a result past the largest double is `#NUM!` (199 `Z`s in radix 36, 255 `Z`s).
Evidence: probes/decimal-unreadable (84 cells, excel-graph, 2026-10-10); gen-m5/98ed0b6354e3394a Calc!A9.
Ours: matches.
Status: VERIFIED.
Not probed: form feed and other control characters at the start; an error radix with over-long text; an empty-cell reference or an array as the text; blanks after radix-16 leading zeros.
Tasks: T-1198

## CONVERT constants and evaluation order
Rule: CONVERT computes `value * F_from / F_to` and only then applies the prefixes
(`* P_from / P_to`). Each unit's factor `F` is a full-precision double in its class's internal
base unit, which is not always the SI unit: force is in dynes (N = 1e5), energy in ergs
(J = 1e7), volume in litres (m3 = 1000), area in square angstroms (m2 = 1e20). Everything else is
in the SI unit (g, m, Pa, W, s, m/s, T, bit, K). The base and the order are visible in the last
digit: `CONVERT(5,"cal","J")` is 20.934, but 5 × 4.1868 is 20.933999999999997;
`CONVERT(5,"eV","J")` is 8.010882434999999E-19 (5 × 1.602176487E-12 / 1E7);
`CONVERT(0.0002,"Mang3","m3")` is 2.0000000000000002E-16 (prefix last; prefix first gives 2E-16);
`CONVERT(0.05,"Mdyn","N")` is 0.5. The factors are not the rounded 15-digit decimals published in
Excel's help: `CONVERT(1,"sg","g")` is 14593.902937206363 (not 14593.9029372064),
`CONVERT(1,"parsec","m")` is 3.0856775812815532E+16. `CONVERT(1,"F","K")` is
255.92777777777775, which is `(1-32)*5/9+273.15` (`5/9*(1+459.67)` is …778).
Evidence: ironcalc-calc ENGINEERING/CONVERT.xlsx, 509 cells (Microsoft Macintosh Excel cached
values, T-0922). Each unit has 2–3 cells (1 → base unit, n → base unit, prefixed), so each factor
is the double that fits all of that unit's cells under the rule. No other base per class (powers
of ten from 1E-32 to 1E+32) and no other order fits every class: reciprocal tables
(`value / K_from * K_to`, ATP/LibreOffice style) fit 143–156 of 159 units, and the SI base with
prefix first fits 150.
Conversions to a non-base unit, to a prefixed unit and between two non-base units follow the same
rule (probes/convert-targets, excel-graph, T-1297): `CONVERT(5,"cal","eV")` is 1.3065976295281882E+20,
`CONVERT(1,"uk_pt","gal")` 0.15011874068810688, `CONVERT(1,"tsp","tbs")` 0.3333333333333333. The
target prefix is applied as a reciprocal multiply: `CONVERT(0.0002,"m3","Mm3")` is
2.0000000000000003E-22 (`* 1/1E18`; dividing gives 2E-22). The lbf factor is 444822.16152605
(`CONVERT(1,"lbf","dyn")` is exactly that).
Overflow: `value * F_from` is not allowed to overflow when the answer is representable:
`CONVERT(1E290,"m2","m2")` is 1E+290, `CONVERT(1E300,"ha","m2")` 1.0000000000000001E+304,
`CONVERT(1E302,"J","J")` 1E+302. The engine divides first (`value / F_to * F_from`) when the
product is infinite; every probed value fits, but whether Excel does this or uses extended
precision is not distinguishable from the probes.
Backend: excel-desktop cached values + excel-graph (probes/convert-targets).
Status: VERIFIED (for the evidenced conversions)
Implemented in: `fn_convert` in functions/engineering/convert.rs (T-0922, T-1297)
Tasks: T-0922, T-1297

## CONVERT temperature
Rule: the temperature pairs do not share one pivot unit; the last digits only fit these forms
(probes/convert-targets, 40 cells, excel-graph, T-1297). Celsius output:
`CONVERT(68,"F","C")` is exactly 20 (the old 273.5 constant, which gave 19.65, was wrong).
From F: C `(v-32)*5/9`; K that plus 273.15; Reau `C*0.8`; Rank `5/9*(v+459.67)*1.8`
(`CONVERT(68,"F","Rank")` is 527.6700000000001). From the others, Kelvin is the pivot
(C `v+273.15`, Rank `5v/9`, Reau `5v/4+273.15`) and out of it C is `k-273.15`, Rank `9k/5`,
Reau `(k-273.15)*0.8`, F `(k-273.15)*9/5+32`, except Rank to F which is `9k/5-459.67`.
Each pair is fitted to two values (68, -40.5); prefixed temperature units are unprobed.
Backend: excel-graph.
Status: VERIFIED (for the two probed values per pair)
Implemented in: `fn_convert` in functions/engineering/convert.rs (T-0922)
Tasks: T-0922, T-1297

## Engineering function argument modes
Rule: every argument of the M9 engineering functions lifts elementwise over an array literal. All
`ROWS(F(..{v;v}..))` cells are 2, including the `places`, `suffix`, unit, order and shift arguments.
IMSUM and IMPRODUCT are the exception.

Over a two-cell range only BITAND, BITOR, BITXOR, BITLSHIFT and BITRSHIFT lift
(`ROWS(BITAND(K15:K16,25))` is 2). Every other function is `#VALUE!` when its first argument is a
multi-cell range. That covers the Bessel functions, the base conversions, COMPLEX, CONVERT, DELTA,
GESTEP, ERF, ERFC and their .PRECISE forms, and IM* (`ROWS(BESSELI(K1:K2,1))`). This is the Analysis
ToolPak split seen in the financial functions. Three later arguments were probed and follow the same split (probes/scalar-lift-shapes H7-H9,
T-1767): `ROWS(BITAND(13,D1:D60))` is 60, while `ROWS(BESSELJ(2.5,B1:B60))` and `ROWS(DEC2BIN(9,D1:D60))` are `#VALUE!`,
even on the range's own rows. Probes/scalar-lift-open-a (T-1777) adds
more later arguments, all `#VALUE!` over a range with a working scalar control: CONVERT's `from_unit`, `to_unit` and number
(Y8-Y10), COMPLEX's `suffix` (Y11), IMSUB's and IMDIV's second argument (Y12, Y13), IMPOWER's `n` (Y14) and IMSUB's first (Y15).
Probes/eng-later-argument-lift (T-1383) covers the remaining later arguments, each on its range's own rows with an array
element (column I) equal to its scalar control (column J):
- `#VALUE!` over a range: BESSELI's, BESSELK's and BESSELY's `n` (H1-H3; L70 off the rows), the `places` of BIN2HEX,
  BIN2OCT, DEC2HEX, DEC2OCT, HEX2BIN, HEX2OCT, OCT2BIN and OCT2HEX (H4-H11), COMPLEX's `i_num` (H16), DELTA's
  `number2` (H17; L72 off the rows), GESTEP's `step` (H18) and ERF's `upper_limit` (H19).
- 60 rows over a range: BITOR's and BITXOR's `number2`, BITLSHIFT's and BITRSHIFT's `shift_amount` (H12-H15; L71 off
  the rows) and BITAND's first argument on its own rows (H20). The elements are the scalar calls:
  `INDEX(BITLSHIFT(13,C1:C60),5)` with C5 = 2 is 52 (K12-K15).
- Error order inside one element: `INDEX(F({"x";..},NA()),1)` is the first argument's `#VALUE!` (or IMPOWER's `#NUM!`)
  for BESSELJ, BITLSHIFT, CONVERT, COMPLEX, IMPOWER and ERF (M1, M3-M7), but `#N/A` for DEC2BIN and DELTA (M2, M8):
  those two read their second argument before the first fails. Ours: `#VALUE!` for both (T-1816).
QUOTIENT, FACTDOUBLE and SERIESSUM's x, n, m are `#VALUE!` too, while MULTINOMIAL, GCD and LCM give one result (ROWS 1)
(Y19-Y30). See arrays.md "Scalar arguments lift: shapes, element errors, and the Analysis ToolPak
range rule".

IMSUM and IMPRODUCT take their arguments whole (the section below has the full rule):
- A range or array literal is folded into the result, not lifted: `IMSUM({"1+2i","3"})` and
  `IMSUM(M1:M2)` are "4+2i", and `ROWS(IMSUM({"1+2i";"3"}))` is 1.
- A blank cell is 0. `IMPRODUCT(M1:M3)` with M3 blank is "0".
- A boolean cell or a typed TRUE is `#VALUE!`.
- Non-complex text is `#NUM!`.
- A range holding both a boolean and text is `#VALUE!`.

Typed coercion follows the ToolPak rule:
- Numeric text is read: `BIN2DEC("101")` is 5, `DEC2BIN("9")` is "1001", and `BITAND("13",25)`,
  `CONVERT("2.5","m","ft")`, `ERF("0.5")` and `BESSELJ("2.5",1)` all work.
- A boolean is `#VALUE!`: `BIN2DEC(TRUE)`, `DEC2BIN(TRUE)`, `IMABS(TRUE)` and `DELTA(TRUE,1)`. Every IM*
  argument refuses one, typed, in a cell or as an array element, and so does COMPLEX's `real_num`
  (probes/eng-imsum-improduct-whole C1-C15: IMREAL, IMSUB's both arguments, IMDIV's second, IMPOWER's both,
  IMCOS, IMCONJUGATE, IMSQRT and IMABS of a TRUE cell, `INDEX(IMREAL({TRUE;"3"}),1)`, `COMPLEX(TRUE,1)`).
  Unprobed, inferred from the shared parse path (T-1828): IMDIV's first argument, IMAGINARY, IMARGUMENT,
  IMCOSH, IMCOT, IMCSC, IMCSCH, IMEXP, IMLN, IMLOG10, IMLOG2, IMSEC, IMSECH, IMSIN, IMSINH, IMTAN, and
  COMPLEX's `i_num`. The 255-argument limit of IMSUM and IMPRODUCT is Excel's documented one, not probed.
- A number is read as its digits: `IMABS(3)` is 3, and `HEX2DEC(255)` is 597 (0x255).
- COMPLEX's suffix is case-sensitive: `COMPLEX(3,4,"I")` is `#VALUE!`.
- Non-complex text is `#NUM!` (`IMSUB("x","1")`), and a number cell is a real number (`IMDIV("3+4i",M2)` with M2 = 3 is
  "1+1.33333333333333i").

These set the generator modes in eval/catalog/functions.yaml. The scalars are `value`, and IMSUM's and
IMPRODUCT's inumbers are `array`.

Evidence: probes/eng-argument-modes (231 cells, excel-graph, 2026-10-11, T-1085). The test data is
M1 "1+2i", M2 3, M3 blank, M4 TRUE, M5 "x" and M6 "2-i".

Ours:
- Matches the argument modes (T-1383): every engineering function but IMSUM and IMPRODUCT is in the SPEC-0011 lifting
  table, BIT* as `Lift` and the rest as `LiftArrays`.
- DEC2BIN's and DELTA's error order inside an element (above, T-1816).
- IMSUM and IMPRODUCT match (T-1384, the section below).
- IMCOT, IMSECH and IMTAN of "3+4i" are off in the last printed digit (T-0907).

Status: VERIFIED (excel-graph)
Tasks: T-1085, T-1383, T-1384, T-0907, T-1816

## A ToolPak argument over CHOOSE: values lift, a reference is refused
Rule: an argument that lifts over arrays and refuses a multi-cell range (IM*, EDATE, ISEVEN, COMPLEX) treats a CHOOSE by
what it evaluates to. CHOOSE with an array index lifts to an array of values, even over range arms, so the call spills:
`IMABS(CHOOSE({1,2},H1:H2,I1:I2))` is the 2x2 3, 6; 4, 8 and `IMSECH(CHOOSE({1,2},{1,2,3},{4;5}))` a 2x3 with `#N/A`
in its third column (CHOOSE's own broadcast, arrays.md). A scalar index passes on the arm it picks: an array arm lifts
(`IMABS(CHOOSE(1,{1,2,3}))` is 1, 2, 3; `EDATE(CHOOSE(1,{4;5}),1)` is 35, 36; `ISEVEN(CHOOSE(1,{4;5}))` TRUE, FALSE), a
multi-cell range arm is one `#VALUE!` (`IMABS(CHOOSE(1,H1:H3))`, `IMABS(CHOOSE(2,{3;4},H1:H3))`) and a one-cell arm is
read (`IMABS(CHOOSE(1,H1))` is 3). IMSUM folds the whole array (`IMSUM(CHOOSE({1,2},{1,2,3},{4;5}))` is `#N/A`), and
`SUBTOTAL(9,TAKE(CHOOSE({1,2},H1:H2,I1:I2),2))` is a 2x2 of `#VALUE!`: TAKE picks values there, not cells.

Evidence: probes/lift-arrays-choose-arms (excel-graph, 2026-10-18, T-1874). H1:H3 = 3, 4, 5; I1:I2 = 6, 8.

Ours: matches for the lifted functions (T-1874). Static analysis used to take such a call for a scalar, since CHOOSE
gives its array arms a range's shape; the fuzz reproducer `IMSECH(CHOOSE({1,2},{1,2,3},{4;5}))` hit a debug assertion.
The SUBTOTAL over TAKE cell matches since T-1882: SUBTOTAL reads TAKE's first argument as cells only when its static
shape is not an array, analysed once so nesting stays linear.

Status: VERIFIED (excel-graph)
Tasks: T-1874, T-1882

## IMSUM and IMPRODUCT take their arguments whole
Rule: IMSUM and IMPRODUCT (1 to 255 arguments) fold every value their arguments hold, in argument order and
row-major order inside a range or array, and the first bad item is the result.
- A range, a reference a function returns (`IMSUM(IF(TRUE,M1:M2))` is "4+2i") and an array give each of their
  cells; nothing lifts, so the call is one value. A union is `#VALUE!` (`IMSUM((M1,M6))`).
- A blank cell is 0, also one past the used area: `IMPRODUCT(M1:M2,Q1:Q3)` is "0" and `IMSUM(M1:M2,Q1:Q3)` "4+2i".
  An empty argument is skipped, not 0: `IMPRODUCT(M1,)` is "1+2i".
- Empty text is `#NUM!`, typed (`IMSUM(M1,"")`), from a formula (M9 `=""`) and in an array
  (`IMPRODUCT({"1+2i",""})`), unlike the one-argument IM* functions that read it as 0.
- The first bad item wins, whatever its kind: `IMSUM(M5,M4)` ("x", TRUE) is `#NUM!` and `IMSUM(M4,M5)` `#VALUE!`;
  `IMSUM(M5,M7)` is `#NUM!` and `IMSUM(M7,M5)` `#DIV/0!` (M7 `=1/0`); `IMSUM({"x",TRUE})` is `#NUM!` and
  `IMSUM("x",NA())` `#NUM!`. A blank does not stop the walk: `IMPRODUCT(M1:M3,M5)` is `#NUM!`.
- Different suffixes are `#VALUE!`: `IMSUM("1+2i","3+4j")` (where IMSUB and IMDIV give `#NUM!`, T-1625 probes).
- A number is a real part (`IMSUM({1,2;3,4})` is "10"), and one cell is its value (`IMPRODUCT(M2)` is "3").
Evidence: probes/eng-argument-modes A60-A83, B/C/E42, B/C/E50 (excel-graph, 2026-10-11, T-1085);
probes/eng-imsum-improduct-whole B1-B28 (excel-graph, 2026-10-17, T-1384). The data is M1 "1+2i", M2 3, M3 blank,
M4 TRUE, M5 "x", M6 "2-i", M7 `=1/0`, M8 "3+4j", M9 `=""`.
Unprobed: a 3-D reference (ours `#VALUE!`, as a union), a real number next to a `j` number (ours `#VALUE!`: a real
reads as an `i` number), a call whose arguments hold no number at all (ours "0") (T-1828).
Status: VERIFIED (excel-graph)
Implemented in: `fold_complex` in functions/engineering/complex.rs (T-1384)
Tasks: T-1384, T-1828

## IMSUM and IMPRODUCT read cap
Rule: RustSheet stops an IMSUM or IMPRODUCT call with `#ERROR!` once the cells its ranges walk and the elements of its
arrays add up to more than 2^27 (`MAX_LIFTED_CELL_VISITS`), before it walks the range that passes the cap, to bound
per-cell work on untrusted input (CLAUDE.md directive 5). A whole column or row counts only up to the sheet's used
area, so `IMSUM(A:A)` reads only the data; `IMSUM(XA1:XFD1048576)` (2^28 cells) is `#ERROR!` on any sheet, where
Excel walks it.
Evidence: none needed for the cap, a resource limit; Excel has none.
Status: DEVIATION (decision 0014)
Implemented in: `fold_complex` in functions/engineering/complex.rs (T-1384)
Tasks: T-1384

## OCT2BIN of a blank cell is 0
Rule: `OCT2BIN(E5,2)` with E5 blank is "00": a blank cell is the number 0. The other X2Y conversions of a blank cell,
and of typed empty text, are unprobed.
Evidence: gen-m9/d244411dd90f7fbd Calc!A15 (`VLOOKUP(..)&UPPER(OCT2BIN(Data!E5,2))` = "-0.000100", excel-graph, 2026-10-12, T-1289).
Ours: `#NUM!` for OCT2BIN, OCT2DEC and HEX2BIN of a blank cell; BIN2DEC gives 0 and DEC2BIN "0".
Status: VERIFIED (web) for OCT2BIN; the rest UNVERIFIED
Tasks: T-1413

## Engineering edge cases seen in a generated sample (unprobed)
Excel gave these values in a seeded engineering sample, which was not committed. They were read from
nested generated formulas, so each one needs a one-question probe before it is implemented (T-1385).
Seen directly as a whole cell value:
- `HEX2OCT("FFFFFFFFFF",4)` is "7777777777". For a negative input `places` is ignored, where we give
  `#NUM!`.
- `IMPOWER("2i",-1)` is "3.06287113727155E-17-0.5i" and `IMPOWER("-i",4)` is "1+2.45029690981724E-16i".
  A tiny part is written in E notation with 15 significant digits, and its value is not what
  cos/sin of the exact angle gives (ours: "0.0000000000000000306161699786838-0.5i").
Inferred from inside a larger formula:
- CONVERT reads its number before its units. A text number with an unknown unit is `#VALUE!`, not
  `#N/A`.
- DELTA and GESTEP report the second argument's error before the first's. `DELTA(<#NUM!>,<#N/A>)` is
  `#N/A`.
- A boolean cell in BITAND, DELTA or GESTEP is `#VALUE!`, where we give `#NUM!`.
- `IMSQRT("")` is `#NUM!`, where we give "0".
- A negative fractional Bessel order is `#NUM!`: `BESSELK(5,-0.0001)`. We truncate the order to 0.
Evidence: `rustsheet-gen --milestone M9 --count 20 --seed 17` run from a scratch catalog holding only
the engineering signatures (excel-graph, 2026-10-11, T-1085; not committed).
Status: UNVERIFIED (probe pending)
Tasks: T-1385

## T-1625 probes: IMPOWER, IMSUB, IMDIV
- Mixed suffixes (`"3+4i"` with `"1+2j"`) are `#NUM!` in IMSUB and IMDIV.
- An empty-string argument is `#NUM!` in IMSUB, IMDIV and IMPOWER; a boolean second argument is `#VALUE!`.
- `IMPOWER("0",0)` is `#NUM!`; `IMDIV` with a tiny divisor gives a large result (`IMDIV("1","1E-200")` = `1E+200`), not #NUM!.
- IMPOWER does not use the naive polar form: `IMPOWER("i",2)` = `-1+1.22514845490862E-16i` (the naive form gives 1.22464679914735E-16), `IMPOWER("-j",3)` real part -1.83772268236293E-16, `IMPOWER("1+i","2")` real part 1.22514845490862E-16, `IMPOWER("1+i",100)` = `-1125899906842640-0.551513671875006i` (naive: `-1125899906842630-0.552925707312197i`). The values differ, not only the printing; tiny parts also print in E-notation.
- A blank cell argument is 0 in IMSUB (`IMSUB(A7,B7)` with B7 blank), unlike empty text (ironcalc-calc/ENGINEERING/COMPLEXs.xlsx Sheet2 G7:G10).
- Implemented (T-1766): the error rules above, IMDIV by Smith's algorithm, and IMPOWER as `|z|^n` by Excel's `^` (square-and-multiply for an integer `n`) times Excel's SIN/COS of the double `atan2(y,x)·n` (argument reduction by the x87 64-bit pi, numbers.md "Trig of a huge angle"). That reproduces all four values bit for bit: the tiny part of `IMPOWER("i",2)` is `π₆₄ − π_double`, the real part of `(1+i)^100` is `sqrt(2)^100` by square-and-multiply (2^50 + 12.5; libm `pow` gives 2^50 + 7.75) and its imaginary part is that times `sin(double(100·π/4))` reduced by `π₆₄`. Each part prints as number-to-text does (E notation past 20 plain characters).
- UNVERIFIED: a blank first argument in IMDIV and IMPOWER is read as 0 (only IMSUB is evidenced), and Smith's algorithm is our guess at IMDIV's arithmetic (an x87 plain formula could differ in the last ulp).
- UNVERIFIED: an angle `atan2·n` of 2^27 or more (SIN's `#NUM!` cut) falls back to libm sine and cosine; no probe covers it.
Evidence: probes/eng-impower, eng-imsub, eng-imdiv (excel-graph, 2026-10-17, T-1625).
Status: VERIFIED (excel-graph)
Tasks: T-1766

## Base conversions read a blank cell and empty text as 0
Rule: every BIN2*/OCT2*/HEX2* function (with or without `places`) treats a blank cell, `""` and a ref to `=""` as 0: `BIN2DEC("")`, `OCT2DEC(blank)`, `HEX2DEC("")` are 0; the others give "0", or "0000" with `places` 4. None of these is an error.
Evidence: probes/eng-base-conv-blank (excel-graph, 2026-10-18, T-1413); gen-m9/d244411dd90f7fbd A15.
Status: VERIFIED (excel-graph)
Tasks: T-1413
