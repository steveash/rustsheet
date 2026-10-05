# Numbers and floating point

Background: Excel stores IEEE-754 doubles. It displays at most 15 significant digits and applies
some extra rounding rules. Microsoft documents a few of them
(https://learn.microsoft.com/en-us/troubleshoot/microsoft-365-apps/excel/floating-point-arithmetic-inaccurate-result),
but the exact rules must be measured.

Every rule below that cites oracle-sidecar values was re-checked with exact float parsing on
2026-10-05 and still holds (T-0026, docs/research/numeric-reaudit-2026-10-05.md).

## Near-zero snapping in addition and subtraction
Rule: when the formula's **outermost** operation is `+`/`-`, or the formula is `SUM(...)`, and the
result is within a few ULP of zero relative to the operands, Excel returns exactly 0.
`=0.1+0.2-0.3`, `=(0.1+0.2)-0.3`, `=SUM(0.1,0.2,-0.3)` and `=1-0.9-0.1` all give 0. Snapping does
**not** happen when the whole formula is parenthesized (`=(X-b)`) or another operation follows
(`=X-b+0`, `=(X-b)*1`). For X = b·(1±k·2^-52): k ≤ 3 always snaps, k = 4–7 snaps for some bases only
(15, 9, 9, 9 of 24), k = 8 never; identical for `X-b`, `X+(-b)` and `SUM(X,-b)`. The exact
threshold is left to T-0110. The earlier hypothesis (no snapping with parentheses or in SUM) was wrong.
Evidence: probes/precision-and-rounding.case.json A2–A4, A15; probes-numeric/30bc5f8fab02d0a2
`snap-addsub/*` (1,296 cells). Oracle: excel-graph, 2026-10-04.
Status: VERIFIED

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
Status: VERIFIED. Implementation: `crates/rustsheet-engine/src/functions/math_and_trigonometry/decimal_round.rs` (T-0106).

## MOD is #NUM! for a huge quotient
Rule: `=MOD(999999999999957+0.5,1)` is `#NUM!`, not 0.5. The threshold on number/divisor is not
measured yet.
Evidence: probes/round-family A20. Oracle: excel-graph, 2026-10-05 (T-0106).
Status: UNVERIFIED (threshold unknown)
Tasks: T-0141

## Number → text conversion uses 15 significant digits
Rule: `=CONCAT(0.1+0.2)` → "0.3": numbers become text with 15 significant digits (124 of 136
`CONCAT` probes equal C's `%.15g`; `(a/b*b)&""` round-trips to the integer in 100 of 100). The
12 others: `1e307*100` → `#NUM!`; subnormals print `0` (`1e-307/10`, `2^-1074`, see below); the
exponent is uppercase (`-2.22044604925031E-16`, `5.55111512312578E-17`, `1E+21`); and Excel stays
in fixed notation where `%.15g` would switch: `123456789012345678` → "123456789012345000",
`1e15+0.3` → "1000000000000000", `9999999999999999` → "9999999999999990" and `2^53+1` →
"9007199254740990" (digits past 15 are zeros), `1e-5` → "0.00001", `1e-15` → "0.000000000000001"
(`LEN` 17). (`1E+20&0.1` → "1E+200.1", `1E+20&100` → "1E+20100"); the exact switch is the 20-character rule below.
Under `&`: `0.1+0.2&100` → "0.3100", `1/3&7` → "0.3333333333333337", `=7+Data!$E5&10` with E5 = 1E-15 → "710". Snapping does not apply inside the CONCAT argument:
`CONCAT(0.1+0.2-0.3)` → "5.55111512312578E-17". `TEXT(x,"0.000…")` also pads after 15
significant digits: `TEXT(1/3,"0.000000000000000000")` = `0.333333333333333000`.
Notation rule (T-0107): plain notation whenever the plain string, without the sign, has at most
20 characters, otherwise `d.dddE±XX` with at least two exponent digits: `99999999999999900000`
and `-10000000000000000000` stay plain, `1E+20`/`-1E+20`/`1E+21` do not; `0.000000000000000001`
(1E-18), `0.000000000000000123` and `0.00012345678901234` stay plain, `1E-19`, `1.23E-17`,
`9.99999999999999E-16` and `1.23456789012345E-05` (21+ characters plain) do not. Trailing zeros of
the 15 digits are dropped (`2^0.5` → "1.4142135623731").
Evidence: probes/number-to-text-notation A1–A44 (excel-graph, 2026-10-05, T-0107);
probes/precision-and-rounding.case.json A9, A16; probes-numeric/4f413cc257ce15c4
`number-to-text/*`. Oracle: excel-graph, 2026-10-04. gen-m1/f91603cd8616521f Calc!A50, A186, A27,
A21, A188 (oracle: excel-graph, 2026-10-05, T-0109).
Status: VERIFIED. Implementation: `number_to_text` in `crates/rustsheet-engine/src/number_format.rs` (T-0107).

## Negative zero prints as 0
Rule: `-x` where x is 0, empty or FALSE converts to the text "0" (no sign); the oracle cannot tell
whether that value is a negative zero or +0. `-0&0.5` → "00.5",
`-0&"x y"` → "0x y", `-Data!C5&1` with C5 empty → "01", `0.1&-'Q''1'!B$4` → "0.10".
Evidence: gen-m1/f91603cd8616521f Calc!A20, A249, A13 and 9 more cells (A7 A28 A33 A71 A110 A112 A125 A177 A250; A71 is
`-Data!B$3` with B3 = FALSE → "01000");
gen-m1/57fe331cb7611dbf Calc!A235 (oracle: excel-graph, 2026-10-05, T-0109).
Status: VERIFIED. Implementation: `number_to_text` (T-0107).

## Overflow in `*` is #NUM! where it happens
Rule: a product beyond the double range is the error `#NUM!` at that point, so it propagates
through `&`, CONCAT, TEXT and LEN as an error. It never becomes an "infinite" number that is
formatted later: `CONCAT(1e307*100)`, `(1e307*100)&""`, `TEXT(1e307*100,"0.000000000000000000")`
and `LEN((1e307*100)&"")` are all `#NUM!`. A result of exactly 1e308 is finite:
`=1E+307*10-10^308` is 0, and `=10^308*10` is `#NUM!`.
Evidence: probes-numeric/4f413cc257ce15c4 Calc!A121-A124; probes/precision-and-rounding A12, A13.
Oracle: excel-graph, 2026-10-04.
Status: VERIFIED for `*`. UNVERIFIED for `+`, `-`, `/` (no probe yet).
Tasks: T-0136

## Comparison tolerance
Rule: comparisons treat operands within a few ULP of each other as equal, like the +/- snapping.
Measured with X = b·(1+k·2^-52) against b: `=`, `<>`, `<=` and `>` treat X as equal to b for
k ≤ 2 at every base; for k = 3–8 only some bases (7 of 12 still equal at k = 8). `<` and `>=` are
uninformative here because X ≥ b in every probe. The threshold is left to the implementing task.
Evidence: probes-numeric/99bc3137b5869417 `compare-tolerance/*` (648 cells). Oracle: excel-graph, 2026-10-04.
Status: VERIFIED

## 15-digit literal parsing
Rule: numeric literals, `"text"+0` and `VALUE("text")` keep only 15 significant digits;
digits 16+ are **truncated**, not rounded: `VALUE("9307149050110945.41")` = 9307149050110940,
`=1234567890.12345678` = 1234567890.12345. Truncation matched 450 of 450 probes.
Evidence: probes-numeric/23733e52a2ff91ab `literal-15-digit/*`; probes/precision-and-rounding A11.
Oracle: excel-graph, 2026-10-04.
Status: VERIFIED
Tasks: T-0143

## Subnormal results become 0
Rule: a result below the smallest normal double is returned as 0: `=2^-1074` → 0,
`=CONCAT(1e-307/10)` → "0".
Evidence: probes/precision-and-rounding A14; probes-numeric `number-to-text/*/1e-307/10`.
Oracle: excel-graph, 2026-10-04.
Status: VERIFIED

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

## POWER and `^`: odd roots of negatives, zero and overflow
Rule: `POWER(-8, 1/3)` is -2 (an odd root of a negative number is real); `POWER(0,0)` is #NUM!;
`POWER(0,-1)` is #DIV/0!; text operands give #VALUE!; array operands work element by element.
Evidence: ironcalc-calc/MATH_AND_TRIGONOMETRY/POWER.xlsx (Excel-authored): Sheet1!D2 (#DIV/0!),
D3 and F9 (-1.9999999999999998 — we return -2, a 1-ulp NumericUlp known failure), sheets 2-8 for ranges.
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

## `^` / POWER are not correctly rounded
Rule: Excel's power differs from correctly rounded pow in the last digit: `=2^(-0.5)` =
0.7071067811865475 (pow: …476), `=100^100` = 1.0000000000000005E+200.
Integer exponents are affected too: with x = 1E-15, `x^3` = 1.0000000000000001E-45 (pow: …03E-45);
`3^-0.5` = 0.5773502691896258 (pow: …257).
Evidence: gen-m2/1b6ceed7489e0800 Calc!A62, f0d1bacec5de8146 Calc!A196 (oracle: excel-graph,
2026-10-04, T-0203); gen-m1/6462bf910ef9f28f Calc!A33, A41, A48, A57, A86, A207 (oracle:
excel-graph, 2026-10-05, T-0109).
Status: VERIFIED
Tasks: T-0221

## ROUND family with huge num_digits
Rule (inconsistent; Excel is right): with an enormous num_digits ROUND/ROUNDDOWN return the number
unchanged (`=ROUNDDOWN(Data!E3,123456789.123)` → 0.5; `ROUND(Data!$B3,123456789.123)` keeps 1E-15;
`ROUNDDOWN(0.5, 1000000000000005.8)` → 0.5), yet `ROUND(1000000000000000.0,Data!$D$7)` evaluates to 0
(it is the divisor in caa981b0753f455c A270, which is `#DIV/0!`; dd99be1f2454159f A159 is similar).
Evidence: gen-m2/f7e6d351c8538ef5 A35, 893302d6c5b99e2e A136, 7469c3e920e1ade4 A336 (Calc sheet). Oracle: excel-graph, 2026-10-04 (gen-m2 sidecars, T-0203).
Status: UNVERIFIED (the rule separating the two outcomes needs a probe before implementing; T-0218)
Tasks: T-0218

## GAMMALN / GAMMALN.PRECISE (T-0926)

Excel's cached values in `ironcalc-calc/STATISTICAL/GAMMALN_GAMMALNPRECISE.xlsx` are the correctly
rounded ln|Gamma(x)|: exactly 0 at x = 1 and 2, GAMMALN(4) = ln 6. The Lanczos `statrs::ln_gamma`
is off by a few ulp, so `special::ln_gamma` uses fdlibm's `lgamma` (`libm` crate; chosen over a
vendored port to keep one maintained definition). 12 cells (B/C rows 5, 9, 15, 23, 24, 26) still
differ in the last digit and need oracle probes (T-0004).

## Excel for the web and desktop Excel differ in the last digits of some functions
Rule: for ATAN2, IMARGUMENT, IMREAL∘IMSQRT, IMCOS, IMTAN, ERFC/ERFC.PRECISE, BESSELJ/BESSELY,
MIRR, IRR and SIN, Excel for the web (the excel-graph oracle) and desktop Excel can differ by up
to 4 ULPs (11 cells), within 1e-9 relative (4 cells), or leave different near-zero residues (3 cells) (two-flow IRR: desktop 2.2e-16, web 0;
`SIN(4)`: desktop -0.7568024953079283, web -0.7568024953079282).
Evidence: docs/research/oracle-calibration.md (18 ironcalc-calc cells, cached by desktop Excel,
recalculated through excel-graph).
Status: VERIFIED (observation). Decision 0030 (rule 4): engine fits target desktop values where
they exist; a web golden that is already a known failure and is missed within 1e-9 relative (or by
a residue below 1e-15) is not a fitting target. Sidecar goldens that differ from desktop below
1e-9 are web values until T-0142. Say which Excel a value comes from when working on these functions.

## GEOMEAN last digits differ from a log-sum evaluation

`=GEOMEAN("47")` is exactly 47 in Excel, and `=GEOMEAN(1,2,"5",TRUE)` is 1.778279410038923;
a log-sum evaluation (`exp(mean(ln x))`) gives 46.99999999999999 and 1.7782794100389228. Excel's
formulation is not yet known (T-0941). Evidence: Excel for the web (excel-graph) sidecar
`crates/rustsheet-xlsx/tests/calc_tests/STATISTICAL/GEOMEAN.oracle.json`, cells Sheet1!B27, G27.
Values are from Excel for the web, which can differ from desktop in the last digit (decision 0030).
Status: VERIFIED (these two values, web); general rule and formulation UNVERIFIED.

## GCD / LCM argument bound
Rule: Excel accepts an argument equal to 2^53: `GCD(2^53,2)` = 2, `LCM(2^53,1)` = 9007199254740992
(and 2^53-1). RustSheet rejects arguments `>= 2^53` with `#NUM!`. Excel's real upper bound is unprobed.
Evidence: probes/resource-limits C1–C4 (excel-graph, 2026-10-05; C1 and C3 are known failures).
Status: VERIFIED (the 2^53 cells); bound UNVERIFIED (probe pending, T-0239)
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
