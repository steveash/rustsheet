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
- For `|x| >= 8` Excel is the NR asymptotic branch (`sqrt(0.636619772/x) * (P cos(xx) - z Q sin(xx))`
  for J, `(P sin(xx) + z Q cos(xx))` for Y; `z = 8/x`, `xx = x - 0.785398164` or `x - 2.356194491`),
  but three of its coefficients differ from the book (T-0908, fitted by least squares on the
  m9-bessel sweep, then bit-exact on every one of its 354 integer-order cells):
  - J0's `P` has `-0.1098628267e-2` where NR has `-0.1098628627e-2` (two digits swapped). J0 is
    ~3e-10 off true; Y0 keeps the book's value.
  - Y0's `Q` has `-0.934945152e-7` where NR has `-0.934935152e-7`; J0 keeps the book's value.
  - J1's `P` repeats `-0.3516396496e-4`: `1 + y(0.183105e-2 + y(-0.3516396496e-4 + y(-0.3516396496e-4 +
    y(0.2457520174e-5 + y(-0.240337019e-6)))))`. That is why `BESSELJ(10,1)` = 0.04347224965225415
    against true 0.04347274617 (and up to 3.4e-5 off at x = 8). Y1 uses the book's `P`.
  - Higher orders run the same recurrences as for small x: J's upward recurrence (n < x) is bit-exact
    at moderate n (n = 1000 passes; not at n >= 2000, see the T-2881 note below) only as `b * (2j * (1/x)) - a`, like Y's (11 cells are 1 ulp off with `(2j/x) * b`), and Miller's
    for n >= x. In the asymptotic form `(z * Q) * sin` is bit-exact; `z * sin * Q` is 1 ulp off at
    `BESSELY(8.5,1)`.
  - `BESSELJ(0,n)` is 0 for n >= 2; `BESSELY` of x <= 0 is `#NUM!`.
  Implemented in `functions/engineering/transcendental/bessel_excel.rs` (`J0_LARGE` and friends);
  the fdlibm port is gone.
  The asymptotic branch for 8 <= x <= 100 is gated by probe case `bessel-asymptotic` (T-2140).
  The asymptotic form (x >= 8) takes Excel's own sine and cosine of the phase (reduced by the x87 64-bit pi, as
  SIN and COS do; `excel_sin_cos`): the 8..100 sweep stays bit-exact, and so is every number cell of `bessel-large-x`
  (x = 1e3, 1e6, 2^27, where libm's sine missed by 1 ulp to ~1e-13 relative) and of `t2547-bessel-phase-cut` (T-2547).
  Past the phase 134217727 the sine and cosine do not answer and BESSELJ and BESSELY are `#NUM!`, the same cut as
  IMPOWER's angle ("IMPOWER past the angle cut") and below SIN's own 2^27 (SIN(134217727.99) is a number, A67):
  `BESSELJ(134217727.785398,0)` and `BESSELY(..,0)` (phase 134217726.9999998) answer and x + 0.001 is `#NUM!`; the
  order-1 phase `x - 2.356194491` cuts at the same 134217727 (A27:A30). So `BESSELJ(2^27,0)` is `#NUM!` (phase
  134217727.21) while `BESSELJ(2^27,1)` answers (134217725.64), and every order from 2 up, which recurs from J0 and
  J1 (or Y0 and Y1), is `#NUM!` once order 0 is (`BESSELJ(134217728.5,1)` answers, `(..,2)` and `(..,4)` are
  `#NUM!`). A negative x cuts on |x| (`BESSELJ(-134217728,0)` `#NUM!`, `(..,1)` 6.863275229685527E-05), and every x
  further out is `#NUM!` for all orders (1E+14, 2E+14, 1E+15, 1E+10). Whether the phase 134217727 itself answers is
  unprobed (ours `#NUM!`, as IMPOWER).
  Evidence: probes/t2547-bessel-phase-cut A1:A74, bessel-large-x (excel-graph, 2026-10-24, T-2547).
  BESSELJ's and BESSELY's recurrences round twice (T-2548): every product and sum in Miller's downward recurrence
  (`j*tox*bj - bjp`, the 1E-10 rescale, the even-order sum) and in the upward recurrence (`b*(2j*(1/x)) - a`) is x87
  code stored to a double after each operation, so it rounds first to a 64-bit significand and then to a double
  (`extended_product`, `extended_sum`). `tox = 2/x`, `1/x` and the final `ans/(2*sum - bj)` are plain doubles as far
  as the evidence tells. Plain double steps drift from Excel by 1E-15 to 1.6E-13 relative from n = 2000 up (the
  result is most sensitive near the turning point n = x); the twice-rounded steps match all 76 cells of the
  turning-point sweep (n = 1000..100000, x = n, n±1/10/100, n+0.5, n-2/3/5) bit for bit, e.g. `BESSELJ(1E+5,100000)`
  0.009636944011336267 and `BESSELJ(100100,100000)` -0.006729033731473101. Rejected: x87 registers kept across
  steps (64-bit state; 0-2 of 43 Miller cells), FMA steps, other associations of the step, NR 3rd edition's ACC 160
  and power-of-two rescale, a 1E10 division rescale, and every other start order.
  Evidence: probes/t2881-besselj-turning-point (excel-graph, 2026-10-27, T-2881).
  BESSELY shares the upward step by inference only: its oracle cells (BESSELY(1,1000) among them) pass either way, and
  no cell yet tells plain from twice-rounded steps for it.
  Web vs desktop: Mac Excel 16.0300's cached `BESSELJ(100000,5)` 0.001846551246313725 and `BESSELY(100000,5)`
  0.001719431947654535 (ironcalc-calc/ENGINEERING/BESSEL.xlsx All!D18, F18) differ from the web's 0.0018465512463137278
  and 0.0017194319476545319 (t2547-bessel-phase-cut A85, A86), which the engine gives; a Mac has no x87 sine. The engine
  follows the web (decision 0109; Windows desktop is T-1075 item 15).

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
- The fdlibm fallback used before T-0908 was bit-exact on only 14 of the 364 cells; the asymptotic model above
  is bit-exact on all of them except the order cap (`BESSELJ(10,1000000)`, below).
- A fractional order truncates: `BESSELJ(8,9.99)` equals `BESSELJ(8,9)`, `BESSELY(30,1.99)` equals `BESSELY(30,1)`.
- Large orders: `BESSELJ(x,n)` is `0` (not an error) for n = 1000, 10000, 100000 and 1000000 (x = 10, 50), and
  `BESSELY(x,n)` is `#NUM!` for those n. The engine matches this up to n = 100000 but gives `#NUM!` for
  `BESSELJ(10,1000000)` (see "BESSEL order cap"). J underflows to 0 at these x, so no cap shows for J here (it shows above 10,000,000, below); Y is `#NUM!` from n = 1000 (the first `#NUM!` order is below).
- BESSELY's first `#NUM!` order (probes/m9-bessel-order-limits, T-2139): x = 1: n = 151 is -5.1999E+307, n = 152 is `#NUM!`;
  x = 10: n = 246 is -1.3738E+308, n = 247 is `#NUM!`; x = 50: n = 400 is finite, n = 401 is `#NUM!`.
  So the `#NUM!` is the overflow of the upward recurrence past the largest double, not an explicit order cap.
- BESSELJ has an explicit order cap: `BESSELJ(10,n)` is 0 for n up to and including 10,000,000 and `#NUM!` from 10,000,001
  (also `BESSELJ(10,2^31-1)`, `(0,2^31-1)`, `(1E5,2^31-1)`). It is not an underflow: `BESSELJ(1E5,100001)` is 0.0094462635180178,
  `BESSELK(2.5,100001)` and `(2.5,2^31-1)` are `#NUM!` (K overflows).
- `BESSELJ(1E-300,100001)` is `#NUM!` although `BESSELJ(10,100001)` is 0 and the order is far below the 10,000,000 cap: the
  Miller recurrence overflows at a tiny x ("BESSEL order cap", T-2546). The engine matches.
Status: VERIFIED for these cells; the algorithm is fitted (T-0908, above).

Provenance: the coefficients are the published Numerical Recipes (2nd ed., §6.5) rational
approximations, reproduced only because Excel's output matches them bit-for-bit.

## BESSEL order cap
Rule: RustSheet runs no Bessel recurrence for an order above 100,000 (`MAX_ORDER`), to bound per-cell work on untrusted input
(CLAUDE.md directive 5). Above it the engine decides in O(1): BESSELI, BESSELJ and BESSELK are Excel's 0 where a bound proves
Excel's recurrence ends in exactly 0, and `#NUM!` otherwise; BESSELY is always `#NUM!`.
Excel past 100,000 (probes/t2546-bessel-past-cap, 83 cells, excel-graph, 2026-10-25; probes/m9-bessel-order-limits, T-2139):
- Excel's own cap for J, I and K is 10,000,000: `BESSELJ(0,1E+7)`, `BESSELI(100,1E+7)`, `BESSELK(800,1E+7)` are 0, and
  `BESSELJ(0,10000001)`, `BESSELI(100,10000001)`, `(100,2^31-1)`, `(2.5,2E+9)`, `BESSELK(800,10000001)` are `#NUM!`. Ours matches.
- BESSELJ is 0 where J_n(x) underflows, also close to the turning point (`BESSELJ(9E+4,100001)`, `(1E+5,1E+6)`, `(10,5E+6)`,
  `(-10,100001)` are 0), but has its value near and past x = n (`BESSELJ(1E+5,100001)` 0.0094462635180178, `(2E+5,100001)`
  -0.0019102131173835007, `(1E+6,1000001)` 0.004431989058609136). A tiny x is `#NUM!`: Miller's downward recurrence
  (Numerical Recipes `bessj`, rescaled by 1E-10 when a term passes 1E+10) overflows when `j * 2/x` passes 1E+10 over many
  steps: `BESSELJ(1E-5,100001)`, `(1E-300,100001)`, `(3E-5,1E+6)` are `#NUM!`, `(2E-5,100001)` and `(3E-4,1E+6)` are 0.
  That also explains the `BESSELJ(1E-300,100001)` quirk above.
- BESSELI's I0(x) and I1(x) (I0 is also the normalizer of its Miller recurrence) at |x| >= 3.75 are `e^x / sqrt(x) * poly(3.75/x)` with Excel's own
  x87 EXP (`excel_exp`, numbers.md "EXP is an x87 2^(a*log2 e) evaluation"), not the correctly rounded exp:
  `BESSELI(100.21..,5)` is 1.177055823943151E+42, which the platform exp misses by 2 ulps (gen-m9/59a16b5ab566ee5c
  Calc!A299, excel-graph; T-2548). Whether BESSELI's Miller steps round twice is undetermined: that cell is the same either way (ours keep plain doubles).
- BESSELI is 0 where I_n(x) underflows (`BESSELI(100,100001)`, `(700,100001)`, `(1E-3,100001)`, `(8,1E+6)`, `(-2.5,100001)`),
  `#NUM!` above x = 709.7 (T-2613, "BESSELI overflow point"), even though I_n(x) underflows (`BESSELI(713,100001)`,
  `(1E+4,100001)`, `(2E+5,100001)`), and `#NUM!` for a tiny x, as J (`BESSELI(2E-5,100001)`; `(5E-5,100001)` is 0: BESSELI
  starts its recurrence at 2(n + sqrt(40n)), twice J's order, so it overflows from a larger x).
- BESSELK is 0 at every order from 2 on from x = 705.5 (inferred: 705 is `#NUM!` and 706 is 0 at order 100001), where K0 and
  K1 are below the normal range, so Excel's upward recurrence starts from 0 (orders 0 and 1 are T-2421's) (`BESSELK(706,100001)`, `(720,1000)`, `(720,100000)`, `(25019,1E+6)`, `(1E+6,1E+7)`), although K_n(x) itself
  overflows there; below 705.5 it overflows to `#NUM!` (`BESSELK(705,100001)`, `(700,100001)`, `(2.5,100001)`).
- BESSELY has its value when x is large (`BESSELY(1E+6,100001)` 0.00011963510780476231, `(2E+5,100001)`,
  `(1E+7,1E+6)`); `(800,100001)` is `#NUM!` (overflow).
Ours: the engine's 0 comes from Kapteyn's bound `|J_n(nz)| <= (z e^w / (1 + w))^n`, `w = sqrt(1 - z^2)`, for J and
`I_n(x) <= (x/2)^n / n! * e^(x^2 / (4 (n + 1)))` for I, each below e^-850, with the recurrence's compound growth past
1E+10 (from `ln Gamma`) small enough that it provably stays finite; the K rule is the flushed K0 and K1 above. A unit test
checks against the uncapped recurrences that the rules never give 0 where the recurrence does not. The engine matches every
J, I and K cell of the probe above except BESSELJ near and past x = n (above), which stays `#NUM!`, and BESSELY past 100,000
when x is large; those are the deviation. Between a provable 0 and a provable overflow lies a narrow band of x (tiny x at the
overflow edge) where the engine is `#NUM!` and Excel may be 0.
Status: DEVIATION (decision 0014), for BESSELJ with x near or above n, BESSELY with a large x, and that band.
BESSELI and BESSELK took the cap in T-1383 (before it, `BESSELI(2.5,2E9)` looped about 4·10^9 times); T-2546 added the O(1) 0.
A lifted Bessel call is also charged against the lifting visit cap by each element's order before any element runs
(SPEC-0011 `ValueWork::Loop`, T-1819): `1 + 65535·n/100000` units for an order n up to the cap, so 2^16 at the
cap and 1 for n = 0; an order past the cap is decided in O(1) without a loop and costs 1. Only a call whose orders sum past that budget is `#ERROR!` where Excel gives values: at most 2,048 elements at
the cap (`ROWS(BESSELI(2.5,SEQUENCE(2100,1,100000,0)))` is `#ERROR!`), while small orders lift freely
(`ROWS(BESSELJ(2.5,SEQUENCE(2100)))` = 2100 and `ROWS(BESSELJ(SEQUENCE(5000)/100,0))` = 5000, as in Excel). A text order
is charged as at the cap. At the order cap a BESSELI element runs about 2·10^5
recurrence steps; 2,048 such elements take about 2 s in a release build (8,192 took 8 s, T-1383).
Evidence: probes-numeric/m9-bessel (T-1004), probes/m9-bessel-order-limits (T-2139) and probes/t2546-bessel-past-cap (T-2546)
show Excel's behavior for huge orders; the cap itself is a resource limit, not an Excel observation.
Status: DEVIATION (decision 0014)
Tasks: T-1004, T-1383; T-1819 (the lifted cap is an order-based work budget); T-2546 (the O(1) 0 past the cap).
Implemented in: functions/engineering/transcendental/bessel_excel.rs (`MAX_ORDER`), bessel_i.rs, bessel_k.rs;
functions/lift/kinds/engineering.rs (`BESSEL`, `BESSEL_WEIGHT`), functions/lift/driver.rs (`kernel_units`)

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
Open gap: `CONVERT(1,"m","ft")` is 3.2808398950131235 in Excel and one ulp low in ours (probes/scalar-lift-open-a,
probes/eng-convert-boolean-unit A7; T-2046, T-2047).
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
Rule: every temperature pair has its own last-digit form; there is no shared pivot unit. The forms
fit 9 oracle points per pair (7 in probes/convert-temperature-sweep: 0, 1, 0.1, 100, 12345.678,
1E-5, 1E10; plus 68 and -40.5 in probes/convert-targets; excel-graph, T-1297, T-1860).
From C: F `v*1.8+32`, K `v+273.15`, Rank `(v+273.15)*1.8`, Reau `v*0.8`. From F: C `(v-32)/1.8`,
K `(v-32)/1.8+273.15`, Rank `v+459.67`, Reau `(v-32)*0.8/1.8`. From K: C `v-273.15`,
F `(v-273.15)*1.8+32`, Rank `v*1.8`, Reau `(v-273.15)*0.8`. From Rank: C `v/1.8-273.15`,
F `v-459.67`, K `v/1.8`, Reau `(v/1.8-273.15)*0.8`. From Reau: C `v*1.25`, F `v*2.25+32`,
K `v*1.25+273.15`, Rank `(v*1.25+273.15)*1.8`. The Rankine/Kelvin ratio is a multiply or divide
by `1.8`, never `9/5` (`CONVERT(0.1,"K","Rank")` is 0.18000000000000002).
Prefixes: only `K` and `kel` take a prefix (`CONVERT(300,"mK","K")` is 0.3, `"kK"` 300000, `"yK"`
to K 3E-22); `mC`, `kF`, `mRank`, `mReau` and their long names are `#N/A`
(probes/convert-temperature-prefix-a/-b, all 11 prefixes). Prefix overflow:
`CONVERT(1E290,"Ym","Ym")` is 1E+290 and `CONVERT(1E300,"Ym","Ym")` 1E+300 (not a same-unit identity: `CONVERT(1E300,"Ym","Zm")` is 1E+303, so the from- and to-prefix
combine into one ratio when `value * from-prefix` would overflow), while `CONVERT(1E300,"Ym","m")`,
`CONVERT(1E290,"Ym","km")` and `CONVERT(1E300,"YK","K")` are `#NUM!` (the combined ratio itself
overflows the result; probes/convert-temperature-ov00..ov03, ov10-ov12, ov20-ov22). Unknown: a truly overflowing temperature result
(`CONVERT(1E308,"C","F")`): excel-graph rejects any workbook holding a 1E308 literal
("unsupportedWorkbook"), so it needs desktop Excel (M14, decision 0039).
Backend: excel-graph.
Status: VERIFIED (nine values per pair; overflowing results unprobed)
Implemented in: `fn_convert` in functions/engineering/convert.rs (T-0922)
Tasks: T-0922, T-1297, T-1860

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
  The same holds, typed and as a TRUE cell, for IMDIV's first argument, IMAGINARY, IMARGUMENT, IMCOSH, IMCOT,
  IMCSC, IMCSCH, IMEXP, IMLN, IMLOG10, IMLOG2, IMSEC, IMSECH, IMSIN, IMSINH, IMTAN, and COMPLEX's `i_num`
  (probes/eng-imsum-improduct-3d-j-empty G1-G34, T-1828).
  The 255-argument limit of IMSUM and IMPRODUCT is Excel's documented one, not probed.
- A number is read as its digits: `IMABS(3)` is 3, and `HEX2DEC(255)` is 597 (0x255).
- COMPLEX's suffix is case-sensitive: `COMPLEX(3,4,"I")` is `#VALUE!`.
- COMPLEX reads its suffix before its numbers: an invalid suffix (a number, TRUE, "J", other text) is `#VALUE!` and an error suffix is that error, whatever errors real_num or i_num hold (17 gen-m9 cells, T-2532). With a valid suffix, real_num's error still comes before i_num's.
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
  (`IMPRODUCT({"1+2i",""})`), as in every one-argument IM* function ("Engineering edge cases from a generated sample (T-1385)").
- The first bad item wins, whatever its kind: `IMSUM(M5,M4)` ("x", TRUE) is `#NUM!` and `IMSUM(M4,M5)` `#VALUE!`;
  `IMSUM(M5,M7)` is `#NUM!` and `IMSUM(M7,M5)` `#DIV/0!` (M7 `=1/0`); `IMSUM({"x",TRUE})` is `#NUM!` and
  `IMSUM("x",NA())` `#NUM!`. A blank does not stop the walk: `IMPRODUCT(M1:M3,M5)` is `#NUM!`.
- Different suffixes are `#VALUE!`: `IMSUM("1+2i","3+4j")` (where IMSUB and IMDIV give `#NUM!`, T-1625 probes).
- A number is a real part (`IMSUM({1,2;3,4})` is "10"), and one cell is its value (`IMPRODUCT(M2)` is "3").
Evidence: probes/eng-argument-modes A60-A83, B/C/E42, B/C/E50 (excel-graph, 2026-10-11, T-1085);
probes/eng-imsum-improduct-whole B1-B28 (excel-graph, 2026-10-17, T-1384). The data is M1 "1+2i", M2 3, M3 blank,
M4 TRUE, M5 "x", M6 "2-i", M7 `=1/0`, M8 "3+4j", M9 `=""`.
Probed in T-1828 (probes/eng-imsum-improduct-3d-j-empty, excel-graph, 2026-10-20):
- A 3-D reference is `#VALUE!`, as a union is: `IMSUM(Sheet2:Sheet3!A1:A2)`, `IMPRODUCT(...)`, a one-cell
  `IMSUM(Sheet2:Sheet3!A1)`, and the same next to a valid argument (D1-D4).
- A real number (no suffix) takes the suffix of the number it meets, not `i`: `IMSUM("3+4j","3")`, `IMSUM(M8,M2)`
  (M8 "3+4j", M2 3), `IMSUM("3","3+4j")` are "6+4j", `IMPRODUCT("3+4j","3")` is "9+12j", `IMSUB("3+4j","3")` is "4j"
  and `IMDIV("3+4j","3")` is "1+1.33333333333333j" (E1-E10). Only a suffixless number is neutral; `i` and `j` numbers
  still clash.
- A call whose arguments hold no number is "0" when they are blank cells (`IMSUM(Q1:Q3)`, `IMPRODUCT(Q1)`,
  `IMSUM(Q1,Q2)`), `#N/A` when they are empty arguments (`IMSUM(,)`, `IMPRODUCT(,)`), and `#NUM!` when they are
  empty text, typed, in a cell or in an array (`IMSUM("")`, `IMSUM(M9)`, `IMSUM({"",""})`) (F1-F12).
Probed in T-2132 (probes/eng-imsub-imdiv-blank-j, excel-graph, 2026-10-20):
- A blank cell takes the suffix of the other operand, as a suffixless number does: `IMSUB("3+4j",Q1)` is "3+4j",
  `IMSUB(Q1,"3+4j")` "-3-4j", `IMDIV(Q1,"3+4j")` "0" and `IMDIV("3+4j",Q1)` `#NUM!` (A1, A2, A5, A6, B1, B2, B5).
- An empty argument of IMSUB or IMDIV, in either place, is `#N/A` (`IMSUB("3+4j",)`, `IMDIV(,"3+4j")`; A3, A4, B3, B4).
- An empty FIRST argument of IMSUM or IMPRODUCT is `#N/A` whatever follows (`IMSUM(,Q1)`, `IMSUM(,"")`,
  `IMSUM(,M9)`, `IMSUM(,"3+4j")`, `IMPRODUCT(,M8)`, `IMSUM(,,M8)`; C1, C2, C5, C6, D1, D2, D5). A later empty argument is
  skipped: `IMSUM(Q1,)` "0", `IMSUM("3+4j",)` "3+4j", `IMSUM(M8,,M8)` "6+8j", `IMPRODUCT(M8,,M8)` "-7+24j"; `IMSUM("",)`
  is `#NUM!` (C3, C4, C7, D3, D4, D6).
Status: VERIFIED (excel-graph)
Implemented in: `fold_complex` in functions/engineering/complex.rs (T-1384)
Tasks: T-1384, T-1828, T-2132

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

## Engineering edge cases from a generated sample (T-1385)
The T-1085 gen-m9 sample suggested seven differences; a one-question probe of each settled them.
- **A negative base-conversion result ignores `places`.** `HEX2OCT("FFFFFFFFFF",4)` is "7777777777"
  and `HEX2BIN("FFFFFFFFFF",4)` is "1111111111" (C1, C6), as DEC2BIN, OCT2BIN, BIN2HEX and the others
  already did. `places` must still be 1 to 10: 0, -1, 11 and 20 are `#NUM!` and "x" is `#VALUE!`
  (C2:C5, C11).
- **Any negative Bessel order is `#NUM!`, checked before truncation.** `BESSELK(5,-0.0001)`,
  `BESSELK(5,-1E-300)`, `BESSELJ(5,-0.9999)`, `BESSELY(5,-0.0001)` and `BESSELI(5,-0.0001)` are `#NUM!`
  (E1:E9); a positive fraction still truncates (`BESSELK(5,0.9999)` = `BESSELK(5,0)`, E10).
- **Empty text is `#NUM!` in every one-argument IM\* function; a blank cell is 0.** `IMSQRT("")`,
  `IMSQRT(""&"")`, `IMSQRT(<cell holding ="">)`, `IMSQRT(INDEX({"";"4"},1))`, and `""` in IMABS,
  IMAGINARY, IMARGUMENT, IMCONJUGATE, IMCOS, IMCOSH, IMCOT, IMCSC, IMCSCH, IMEXP, IMLN, IMLOG10, IMLOG2,
  IMREAL, IMSEC, IMSECH, IMSIN, IMSINH and IMTAN are all `#NUM!` (D1:D25). A blank cell is 0 in all of
  them: `IMSQRT(blank)` is "0", `IMCOS(blank)` and `IMCOSH(blank)` are "1" (D3, D26:D32). This is the
  rule IMSUB, IMDIV and IMPOWER already followed (T-1625 below).
- **CONVERT refuses a boolean before it reads the units.** `CONVERT(TRUE,"m","ft")`, a TRUE cell,
  `CONVERT(FALSE,"m","m")`, a TRUE array element and `CONVERT(<TRUE cell>,"xyz","m")` are `#VALUE!`
  (A8, A11:A14). The sample's other CONVERT hypothesis was wrong: a numeric text with an unknown unit
  is `#N/A` (`CONVERT("2.5","ft","xyz")`, A1), non-numeric text is `#VALUE!` whatever the units (A2,
  A4, A6, A7); a blank cell is 0 (A15) and `""` is `#VALUE!` (A16).
- **Not differences** (the sample's formulas misled): DELTA's and GESTEP's second-argument error
  wins, as T-1816 already implements (B1, B2); a boolean cell in DELTA and GESTEP is `#VALUE!` (B3,
  B4, B6, B7), which ours already gave; and BITAND reads a boolean, typed or in a cell, as its number
  (`BITAND(TRUE,139)` and `BITAND(<TRUE cell>,139)` are 1, B5, B8, B11). The IMPOWER E-notation cells
  (`IMPOWER("2i",-1)`, `IMPOWER("-i",4)`, `IMPOWER("3+4i",1E-15)`) already matched since T-1766 (F1:F4).
Evidence: probes/eng-gen-sample-edges (excel-graph, 2026-10-19, T-1385).
Status: VERIFIED (excel-graph); implemented (T-1385)
Tasks: T-1385

## CONVERT: a boolean unit is #VALUE! (T-2676)
Rule: a from_unit or to_unit that is a reference to a TRUE or FALSE cell is `#VALUE!`, not the unknown-unit `#N/A`, with
a valid number in the first argument: `CONVERT(2^31-1,<FALSE cell>,"HP")`, `CONVERT(1-2^-53,<FALSE cell>,"mm")` and
`CONVERT(1E+21,"C",<TRUE cell>)` are `#VALUE!`. It is the units' side of "CONVERT refuses a boolean before it reads the
units" (T-1385 above).
A typed boolean and a boolean array element are the same `#VALUE!`: `CONVERT(1,TRUE,"m")`, `CONVERT(1,"ft",FALSE)`,
`CONVERT(1,{TRUE},"m")` (A1:A6). An array of units lifts element by element, so `CONVERT(1,{"m",TRUE},"ft")` spills
{3.2808398950131235, `#VALUE!`} (A7:B7, also A8:B8 and the vertical A28:A29). The text "TRUE" is only an unknown unit,
`#N/A` (A32).
Order: the units are checked in turn and from_unit's error or boolean wins: `CONVERT(1,NA(),TRUE)` is `#N/A` and
`CONVERT(1,TRUE,NA())` `#VALUE!` (A9:A12); with D3 `=1/0`, `CONVERT(1,D3,TRUE)` is `#DIV/0!`, `CONVERT(1,TRUE,D3)` and
`CONVERT(1,FALSE cell,D3)` `#VALUE!` (A13, A14, A24, A25). A boolean in either unit beats an unknown or empty unit in
the other, so both units are type-checked before either is looked up: `CONVERT(1,"xyz",TRUE)`, `CONVERT(1,TRUE,"xyz")`,
`CONVERT(1,"",TRUE)` and `CONVERT(1,TRUE,"")` are all `#VALUE!` (A15:A20). The number argument comes first:
`CONVERT(NA(),TRUE,"m")` is `#N/A` (A27). Cells that cannot tell the order apart: an unknown unit against an error is `#N/A` either
way (A21:A23), `CONVERT("x",TRUE,"m")` is `#VALUE!` (A26) and a class mismatch is `#N/A` (A30).
Evidence: gen-m9/290180acf017709c Calc!A148, A158, gen-m9/f81bd3b6dc373e5c Calc!A291 (excel-graph, T-2676 sweep,
2026-10-26; the units are Data!C6 = FALSE, Data!E10 = FALSE and Data!A7 = TRUE); probes/eng-convert-boolean-unit
A1:A32 (excel-graph, 2026-10-26, T-2810; D1 TRUE, D2 FALSE, D3 `=1/0`).
Ours: since T-2793 `#VALUE!` for a boolean unit, read in order (from_unit's error or boolean first), before the unit
lookup; typed and array-element booleans take the same path, so every error cell of the T-2810 probe matches. Its
three numeric cells are `CONVERT(1,"m","ft")`, one ulp low in ours (3.280839895013123; T-2046, T-2047).
Status: VERIFIED (excel-graph)
Tasks: T-2793, T-2810

## T-1625 probes: IMPOWER, IMSUB, IMDIV
- The empty value IFERROR or IFNA hand on for a blank cell is 0 in IMSUB and IMDIV, as the blank cell is (`IMSUB("1",IFERROR(Q1,"x"))` "1", `IMSUB(IFERROR(Q1,"x"),"1")` "-1", `IMDIV(IFERROR(Q1,"x"),"1")` "0", `IMDIV("1",IFERROR(Q1,"x"))` `#NUM!`); only an argument left out of the formula is `#N/A` (`IMSUB("1",)`). A variable bound to an omitted LAMBDA parameter or to an empty LET value hands the omission on and is `#N/A` too: `LAMBDA(a,b,IMSUB(a,b))("1",)`, `(,"1")`, `LAMBDA(a,b,IMDIV(a,b))("1",)`, `(,"1")` and `LET(e,,IMSUB("1",e))` are `#N/A` (probes/eng-imsub-imdiv-lambda-let-empty A1:A4, A12; T-2770), while `LET(e,IFERROR(Q1,"x"),…)` and `LET(e,IF(TRUE,Q1),…)` bind a real empty value that is 0, and `LAMBDA(a,b,IMSUB(a,b))("1")` (a missing argument) is `#VALUE!`. The first argument's own error still wins (`IMSUB("a*abc",IFERROR(Q1,"x"))` `#NUM!`). Probes/eng-imsub-imdiv-iferror-empty; matched since T-2543.
- Mixed suffixes (`"3+4i"` with `"1+2j"`) are `#NUM!` in IMSUB and IMDIV.
- An empty-string argument is `#NUM!` in IMSUB, IMDIV and IMPOWER; a boolean second argument is `#VALUE!`.
- `IMPOWER("0",0)` is `#NUM!`; `IMDIV` with a tiny divisor gives a large result (`IMDIV("1","1E-200")` = `1E+200`), not #NUM!.
- IMPOWER does not use the naive polar form: `IMPOWER("i",2)` = `-1+1.22514845490862E-16i` (the naive form gives 1.22464679914735E-16), `IMPOWER("-j",3)` real part -1.83772268236293E-16, `IMPOWER("1+i","2")` real part 1.22514845490862E-16, `IMPOWER("1+i",100)` = `-1125899906842640-0.551513671875006i` (naive: `-1125899906842630-0.552925707312197i`). The values differ, not only the printing; tiny parts also print in E-notation.
- A blank cell argument is 0 in IMSUB (`IMSUB(A7,B7)` with B7 blank), unlike empty text (ironcalc-calc/ENGINEERING/COMPLEXs.xlsx Sheet2 G7:G10).
- Implemented (T-1766): the error rules above, IMDIV by Smith's algorithm, and IMPOWER as `|z|^n` by Excel's `^` (square-and-multiply for an integer `n`) times Excel's SIN/COS of the double `atan2(y,x)·n` (the angle by Excel's ATAN2 kernel `atan(y/x)` since T-2610, "1i" below) (argument reduction by the x87 64-bit pi, numbers.md "Trig of a huge angle"). That reproduces all four values bit for bit: the tiny part of `IMPOWER("i",2)` is `π₆₄ − π_double`, the real part of `(1+i)^100` is `sqrt(2)^100` by square-and-multiply (2^50 + 12.5; libm `pow` gives 2^50 + 7.75) and its imaginary part is that times `sin(double(100·π/4))` reduced by `π₆₄`. Each part prints as number-to-text does (E notation past 20 plain characters).
- A blank first argument is 0 in IMDIV and IMPOWER (`IMDIV(blank,"1+i")` and `IMPOWER(blank,2)` are "0"; `IMPOWER(blank,0)` and `IMDIV(blank,blank)` are `#NUM!`). IMDIV by Smith's algorithm matches Excel to the last digit on a sweep of 13 non-trivial divisors, including `1E10+1E-10i` over `1E-5+1E5i` and `1+i` over `1E-200+1E-200i` (`1E+200`).
- IMPOWER changes rule once the angle `atan2(y,x)·n` is near 2^27, but not at 2^27 (a radius-1 base stops answering; others: "IMPOWER past the angle cut"): `IMPOWER("i",85445658)` (angle 134217725.7) answers, `IMPOWER("i",85445659)` (134217727.3) is `#NUM!`. We cut at 134217727 (a bracket, not a derived value). A radius that underflows to 0 is "0" whatever the angle (`IMPOWER("0.5i",134217728)`, `IMPOWER("0.5",134217728)`); one that overflows is `#NUM!` (`IMPOWER("1E-200+1E-200i",-2)`, `IMPOWER("1E200",2)`; `IMPOWER("1+i",170891317)` is this overflow, its angle is under the cut).
- Excel's modulus matches `hypot`, and `sqrt(x²+y²)` is ruled out: the 16-digit literal reads as the 15-digit
  0.707106781186547 (`IMREAL(..)-0.707106781186547` is 0, A17), and `IMABS("0.7071067811865476+0.7071067811865476i")`
  is 0.9999999999999993 (1-6·2^-53; `sqrt` gives 1-7·2^-53; a scaled `|x|·sqrt(1+(y/x)²)` also gives it, so one cell
  does not single out `hypot`), and IMPOWER's `|z|^n` of that base matches Excel from it
  (`IMPOWER(..,170891300)` = `-0.999999886163532+1.2252504385364E-08i`; probes/eng-impower-angle-cut A9-A17,
  eng-impower-past-cut A15; T-1950; excel-graph). Mac desktop's cached IMABS digits in COMPLEXs.xlsx match neither
  (below, "neither `sqrt(x²+y²)` nor `hypot`"), unexplained. See "IMPOWER past the angle cut" for bases past the cut.
- IMLN, IMLOG10 and IMLOG2 use the same modulus: `IMLN("0.7071067811865476+0.7071067811865476i")` is
  `-6.66133814775094E-16+0.785398163397448i` (ln of 0.9999999999999993; `sqrt` would give -7.77E-16), IMLOG10 `-2.89298239965986E-16+..`,
  IMLOG2 `-9.61027951144476E-16+..` (probes/eng-imln-modulus A1, A6, A11; T-2117). IMSQRT keeps `sqrt(x²+y²)`: its first cell cannot tell (15 digits agree) and Excel's "0" for `3E-200+4E-200i` and `1E-170+1E-170i` (x² underflows; A18, A20) matches only that form.
- IMLN, IMLOG10 and IMLOG2 are `#NUM!` when both parts are nonzero and `x²+y²` is infinite or subnormal: `1E153+1E153i` works,
  `1E154+1E154i` (sum 2E308) and `1E-154+1E-154i` (sum 2E-308 < 2.2250738585072014E-308) are `#NUM!`, `1.5E-154+1.5E-154i`
  (2.25E-308), `1E-160+1i` and `1.1E154+0.1E154i` work, and a zero part exempts the number (`IMLN("1E-160+0i")`,
  `IMLN("1E160i")`, `IMLN("1E154+0i")` answer). Implemented (T-2117; probes/eng-imln-modulus-range A1-A42, A71-A78, A81-A84).
- IMSQRT and the logs flush a subnormal square to 0 (T-2572): IMSQRT is the polar root of x² + y² with each square 0
  when it is subnormal, so `IMSQRT("1E-154+1E-154i")` (each square 1E-308 < 2.2250738585072014E-308),
  `IMSQRT("1E-155+1E-155i")`, `IMSQRT("1E-160+0i")`, `IMSQRT("1E-160i")` and `IMSQRT("1E-155+0i")` are "0", while
  `IMSQRT("1E-153+1E-153i")` answers; `IMSQRT("1E-154+1E-153i")` is 2.34467878292635E-77+2.12190513569646E-77i, the root
  of 1E-153 alone at half of atan2(1E-153, 1E-154). With both parts nonzero the logs drop a part whose square flushes
  from the modulus and keep it in the argument: `IMLN("1E-154+1E-153i")` is -352.295519228089+1.47112767430373i
  (ln 1E-153; `hypot` gives -352.290544062662). A zero part still exempts the number (`IMLN("1E-155+0i")` is
  -356.900689414077, though 1E-310 is subnormal), and IMABS does not flush (`IMABS("1E-154+1E-154i")` is
  1.4142135623730951E-154). The overflow side is unchanged: a square past DBL_MAX is `#NUM!` in IMSQRT
  (`1E155+1E155i`, `1E160+0i`, `3E200+4E200i`), a sum past it is "0" (`1E154+1E154i`). Implemented in
  `flushed_square` (functions/engineering/complex/mod.rs; probes/eng-imln-modulus-range A43-A56, A79, A80,
  eng-imln-modulus A17; excel-graph). Extrapolated, not probed: parts of about 1.06E-154 to 1.49E-154, where each
  square is subnormal but their sum is normal (`IMLN("1.2E-154+1.2E-154i")`; ours `#NUM!`, IMSQRT "0"; T-2894).
Evidence: probes/eng-impower, eng-imsub, eng-imdiv (excel-graph, 2026-10-17, T-1625); probes/eng-impower-imdiv-edges, eng-impower-angle-cut (excel-graph, 2026-10-19, T-1892); probes/eng-impower-past-cut (excel-graph, 2026-10-20, T-1950); probes/eng-imln-modulus, eng-imln-modulus-range (excel-graph, 2026-10-21, T-2117).
Status: VERIFIED (excel-graph)
Tasks: T-1766, T-1892, T-1950, T-2117, T-2572, T-2894

## IMPOWER past the angle cut
Rule: once IMPOWER's angle `atan2(y,x)·n` passes the cut (about 134217727), Excel's sine and cosine both read as the
largest double, 1.7976931348623157E+308, in every quadrant: each part is `|z|^n · DBL_MAX`, both positive and equal,
and the result is `#NUM!` when that is not below DBL_MAX: evidenced for a radius of exactly 1 and above 1; a radius
just below 1 whose product rounds to DBL_MAX is unprobed (ours `#NUM!`). A pure imaginary or real
base of radius 1 is therefore `#NUM!` (`IMPOWER("i",85445659)`, `IMPOWER("-1",42722830)`), and so is
`IMPOWER("1.0000000001i",85445659)` and a negative power of a radius below 1 (`IMPOWER(..,-170891318)`), while
`IMPOWER("0.99999973i",85445659)` is `1.71948380972175E+298+1.71948380972175E+298i` (|z|^n ≈ 9.565E-11, the radius of
the 85445658th power, A19, times DBL_MAX; the 85445660th and 85445661th powers and the power 85445659.5 follow the
radius), `IMPOWER("-0.9999999",42722830)` is `2.50776326229151E+306+2.50776326229151E+306i`, and the four quadrants of
`0.7071067811865476±0.7071067811865476i` past the cut all give two equal positive parts
(`…,170891318` = `1.79769293021926E+308+1.79769293021926E+308i`). A radius that underflows to 0 is still "0".
Evidence: probes/eng-impower-past-cut A1-A14, A19 (excel-graph, 2026-10-20, T-1950); probes/eng-impower-angle-cut A18,
A19 (excel-graph, 2026-10-19, T-1892).
Status: VERIFIED (excel-graph)
Implemented in: `fn_impower` in functions/engineering/complex.rs (T-1950)
Tasks: T-1950

## IM* last-digit sweep: IMSEC and IMSECH use different kernels (T-2043)
Evidence: probes-numeric case `im-last-digit-sweep` (excel-graph, no pending cells): 78 inputs (the
23 usable COMPLEXs.xlsx D-column ones plus A20's 3.2E-19 (D20 is a COMPLEX formula; sweep row 20 is not COMPLEXs row 20's input) plus quadrant, 1E-5..1E3, pure real/imaginary rows) x IMABS,
IMARGUMENT, IMCSC, IMSEC, IMSECH, IMLOG2, IMCOT, with IMREAL/IMAGINARY of each text result as
number cells (columns B-R, row = input row). It exists to let T-2044 fit the kernels.
Rule: although sech(3+4i) and sec(4+3i) are mathematically the same real part, Excel gives
`IMSEC("4+3i")` real part -0.0652940278579471 (row 23, G23) and `IMSECH("3+4i")` real part
-0.065294027857947 (row 58, J58/K58): the two functions are not the same formula up to
rotation, so one kernel cannot serve both. Control pair (these agree, not a second split): `IMSEC("3+4i")` and `IMSECH("4+3i")` both
give real part -0.0362534969158689 but IMAGINARY +0.00516434460775318 vs -0.00516434460775318
(sign per the rotation).
Ours before T-2044: IMSEC G23/H23 gave -0.065294027857947 and IMSECH K58 -0.0652940278579471, i.e. swapped; the kernels in "IM* reciprocal kernels" below give Excel's digits.
Status: VERIFIED (excel-graph, 2026-10-21, `im-last-digit-sweep`); the kernels are T-2044's.

Also found by the sweep (same case; Ours is what RustSheet gives):
- Tiny arguments lose about 8 digits in IMCSC and IMCOT: `IMCSC("1E-5+1E-5i")` is 49999.9979339908-49999.9979306575i (E70, F70; exact 50000.0000016667), `IMCSC("0.00001i")` imaginary part -99999.9917276303 (F65), IMCOT real 49999.9958596485 (Q70), imaginary -49999.9958663151 (R70) and -99999.9917326303 (R65). Reproduced by the T-2044 kernels below (before them ours gave the exact values, E70 50000.0000016667, F65 -99999.9999983333). Status: VERIFIED (excel-graph); implemented (T-2044).
- Excel keeps tiny results where the denominator is huge: `IMCSC("-724.227-562.884i")` is -6.94796703439417E-245-6.26263832046024E-246i (E55, F55; the IMSEC of it is -6.26263832046024E-246+6.94796703439417E-245i, G55:I55) and `IMSECH("-524.561+5.3113i")` is 1.73046857535442E-228-2.53533899849465E-228i (K42, L42). Rule: past the point where the denominator overflows (the hyperbolic argument v above about 355), cosh v and |sinh v| are both M = e^|v|/2, and the result is the trigonometric parts over M, i.e. times 2e^-|v| (IMSEC `(cos x, sgn(y)·sin x)`, IMCSC `(sin x, -sgn(y)·cos x)`, IMSECH `(cos y, -sgn(x)·sin y)`); this reproduces all 15 digits of the 6 cells. Where 2e^-|v| is subnormal the parts print 0 (`IMSEC("1.5+711i")`, `IMSECH("711+1.5i")`). Status: VERIFIED; implemented (T-2335; ours was 0 before). IMCSCH of a large non-real argument (`IMCSCH("-524.561+5.3113i")`) is unprobed and still answers 0.
- `IMCSC("0")`, `IMLOG2("0")` and `IMCOT("0")` are `#NUM!` (D21, M21, P21), `IMARGUMENT("0")` is #DIV/0!. Implemented (T-2334); before it we answered the text `NaNNaNi`, `-inf`, `inf`. Status: VERIFIED.
- Text quirks (not kernels): `IMCOT("-57.9375+18.7089i")` prints "-4.00011374673746E-17-1i" (P43): the
  imaginary part -0.9999999999999999 keeps its "1", while an exact -1 is "-i" (P44); ours "-i" for both.
  `IMCSC("1E-20")` and `IMCOT("1E-20")` print "100000000000000000000" (D18, P18); ours "1E+20". Owned by T-2554 (after
  the IM* text-formatting tasks T-2001, T-2056). Status: VERIFIED (excel-graph).
Errors in the sweep (row 21, input "0"): IMARGUMENT #DIV/0! (C21); IMCSC, IMLOG2 and IMCOT #NUM! (D21:F21, M21:R21).

## IM* reciprocal kernels, IMABS and IMARGUMENT (T-2044)
Fitted on probes-numeric `im-last-digit-sweep` (excel-graph, T-2043), scoring each candidate on whether it
prints Excel's 15 digits (IMABS and IMARGUMENT: the double, bit for bit). SIN and COS of the real (IMSEC, IMCSC,
IMCOT) or imaginary (IMSECH) part are Excel's own, reduced by the x87 64-bit π (numbers.md "Trig of a huge
angle"); cosh and sinh are libm's, except IMSECH's and IMCSCH's (IMSINH's `1/u` form) and IMSEC's
(IMCOS's `1/u` form inside the band, whose ends are unprobed, T-2797; T-2884). Rule:
- IMCSC: `t = sin²x + (cosh 2y − 1)/2`, result `(sin x·(cosh y/t), −cos x·(sinh y/t))`: 69 of the 69 finite cells
  (the old `sin²x·cosh²y + cos²x·sinh²y` 58). Writing sinh²y as `(cosh 2y − 1)/2` is what loses the tiny-argument
  digits above; `sin²x + (cosh²y − 1)` reproduces those two rows but misses `0.00829243+6.03779e-05i` (row 31).
- IMSEC (T-2743): `t = cos²x + ((cosh 2y + 1)/2 − 1)`, i.e. sinh²y as cosh²y − 1 with cosh²y from the
  half-angle identity, result `(cos x·cosh y/t, sin x·sinh y/t)`: all 70 finite cells. Inside the `1/u` band of
  IMSEC's parts (0.01 < |y| <= 20) sinh²y is `(cosh 2y − 1)/2` instead (T-2920, below). Near a pole the
  cancellation in cosh²y − 1 is visible: `IMSEC("956.615+0.00387493i")` is -2.46276584105671+258.045027895372i
  (G33; exact -2.462765841048564+258.045027894412i, off in the 10th digit) and `IMSEC("23.7944+0.00977722i")`
  4.33329584706818-0.178960935839407i (G29; exact real part 4.3332958470681874). IMCSC's `(cosh 2y − 1)/2`
  (T-2044) missed G29 and G33; `cos²x·cosh²y + sin²x·(cosh²y − 1)` with `cosh²y = cosh y·cosh y` gets them but
  misses rows 45 and 79 (`-9.51393+8.31084i`, `1+0.5i`); `cos²x + (cosh y·cosh y − 1)` misses rows 20, 36, 45.
- IMSECH (T-2750): `t = fma(sinh x, sinh x, cos²y)`, i.e. `cos²y + sinh²x` (sinh squared itself, unlike IMSEC) with the
  product sinh²x fused into the sum (one rounding), result `(cosh x·cos y/t, −sinh x·sin y/t)`: all 62 finite cells.
  The unfused sum (T-2044) misses only J34/L34, `IMSECH("1.34384-77.6742i")`, Excel -0.367433538630135+0.376675382306336i
  (exact imaginary part 0.37667538230633553, unfused ...306335); fusing cos²y into the sum instead (`fma(cos y, cos y,
  sinh²x)`) also misses J34. The old `cosh²x cos²y + sinh²x sin²y` kernel missed rows 28, 49, 79 and the IMSECH("3+4i")
  digit; this is why IMSEC("4+3i") and IMSECH("3+4i") differ. Other forms scored on the 61 sweep rows with a nonzero
  imaginary part (row 42 excluded, the overflow path answers it): `1/IMCOSH(z)` as `(a, −b)/(a² + b²)` 58 (misses rows
  7, 49, 54), Smith's division 56, `cosh²x − sin²y` 52, the half-angle `(cosh 2x + cos 2y)/2` 55, IMSEC's T-2743 form
  turned a quarter 55, `(e^x ± e^-x)/2` for cosh and sinh 51. A fused form fitting where the plain one misses suggests
  Excel's build contracts `a·a + b·b` to `fma(b, b, a·a)`. That is refuted for IMCSC and IMCOT and undetermined for IMSEC and IMCSCH (T-2757, scored on
  every oracle cell of IMSEC, IMCSC, IMCSCH and IMCOT in gen-m9, ironcalc-calc, probes and probes-numeric `im-*`: 634,
  642, 546 and 648 cells; each kernel misses one cell there today, none of them a kernel digit): fusing IMCSC's `sin²x`
  (`fma(sin x, sin x, (cosh 2y − 1)/2)`) misses D33 (`IMCSC("956.615+0.00387493i")` real part ...189521 for Excel's
  ...189522), and fusing IMCOT's `2cos²x` (`fma(−2cos x, cos x, cosh 2y + 1)`) misses P31 and P70 (real part
  120.582756444086 for 120.582756444066, and 49999.9958602735 for ...8596485). Fusing IMSEC's `cos²x`
  (`fma(cos x, cos x, (cosh 2y + 1)/2 − 1)`) and either product of IMCSCH's sum ties the current kernels: no cell
  there tells them apart. T-2817 probed inputs chosen (with the engine's kernels) so the forms print different text,
  probes-numeric `im-sec-csch-fused` (excel-graph, 2026-10-27; 34 IMSEC and 44 IMCSCH inputs, the text scored):
  - IMSEC: the plain T-2743 kernel 23 of 34, fusing `cos²x` 10; `cos²x + sinh²y` 22, fused either way 18 and 22,
    `cos²x + (cosh y·cosh y − 1)` 19 (fused 14), `fma(cos²x·cosh y, cosh y, sin²x·sinh²y)` and the other product fused 21.
    The kernel is unchanged (it also keeps all 70 sweep cells); `IMSEC("-1.44771+0.0419247i")`, Excel
    7.30033916775268-2.47256566938037i, is fitted by none.
  - IMCSCH: `t = sinh²x + sin²y` (the identity for the four-product sum) 33 of 44, fusing either square of it 32, the
    four-product `sinh²x·cos²y + cosh²x·sin²y` 25, it with `sinh²x·cos y·cos y` fused 30 and with `cosh²x·sin y·sin y`
    fused 18; squares of the products (`(sinh x·cos y)² + (cosh x·sin y)²`) 26, fused 26 and 24. The kernel is now
    `sinh²x + sin²y` (T-2817); every older cell ties. No form fits all 44 (`IMCSCH("0.0853196+0.00680423i")`, Excel
    11.6323554979869-0.929943499720481i, is fitted by none), so the remaining misses are likely in the cosh, sinh, sin or
    cos parts, not the grouping (superseded: T-2841 below fits all 44 with IMSINH's parts).
  - T-2841 scored the parts on every IMSEC and IMCSCH oracle cell (gen-m9, ironcalc-calc, probes, probes-numeric
    `im-*`). IMCSCH with IMSINH's `1/u` sinh x and cosh x (`inverse_exp_sinh_cosh`) fits all 44 fused texts and keeps
    every older cell. The `(EXP(x) ∓ EXP(-x))/2` pair fits 39 and IMSIN's `trig_sinh_cosh` 38, and each loses 2 older
    cells. Excel's reduced SIN and COS of y tie libm's. IMSEC with each numerator divided first, `cos x·(cosh y/t)`
    and `sin x·(sinh y/t)`, fits 27 of 34 (23 before) and keeps every older cell. For IMSEC's cosh y and sinh y,
    IMSINH's `1/u` form fits 28 but loses 1 older cell, the EXP pair 29 (10 lost) and IMCOS's `trig_sinh_cosh` 31
    (9 lost). cosh 2y as the EXP pair ties; as the `1/u` form it fits 26. These two are now the kernels
    (functions/engineering/complex/trig.rs). Seven IMSEC texts stayed one unit of the 15th digit off (B3, B9, B19, B20,
    B21, B31, B33), among them `IMSEC("-1.44771+0.0419247i")`.
  - T-2884 scored IMSEC's cosh y, sinh y and cosh 2y parts the same way, on every IMSEC oracle cell (gen-m9,
    ironcalc-calc, probes, probes-numeric): fused texts fitted of 34 and older cells lost. Using IMCOS's
    `trig_sinh_cosh` (`1/u` form inside (0.01, 20]) inside the band and libm's cosh and sinh outside it fits 31 and
    keeps every older cell. The full `trig_sinh_cosh`, with the EXP pair outside the band, fits 31 but loses 9, so
    outside the band Excel's IMSEC keeps libm's digits. Other candidates (fitted, lost): IMSINH's `1/u` form at |y| with
    the sign of y 29, 0; at y 28, 1; EXP pair 29, 10; `e = EXP(y)` then `(e ∓ 1/e)/2` 31, 9; libm sinh with any other
    cosh 27 or 28, 0. cosh 2y taken the same way (`1/u` inside the band, libm outside) fits 32 and keeps every older cell.
    `trig_sinh_cosh(2y)` everywhere (the EXP pair outside the band) ties it, so no cell decides cosh 2y outside the band; it fits B20 and B33 (`IMSEC("-1.44771+0.0419247i")`). The EXP pair
    fits 31. IMSINH's `1/u` form fits 30. `cos²x + sinh²y` fits 28 and loses 4, `cos²x + (cosh²y − 1)` 22 and loses 5,
    and fusing cos²x fits 15. `(cosh 2y − 1)/2` for sinh²y fits 32 but loses 2. Dividing the product `cos x·cosh y`
    by t fits 28. These are now the kernels. B3 and B31 stay one unit of the 15th digit low in the real part
    (`IMSEC("2.11217+0.627617i")` Excel -0.86864314680255, ours ...549).
  - T-2920 fitted B3 and B31 (`IMSEC("2.11217+0.627617i")` -0.86864314680255+0.803795556920423i,
    `IMSEC("-3.75483+1.33847i")` -0.436112042213995+0.267414799382832i; both real parts lie near a 15-digit
    tie, and the old kernel was a few ulps low). Each candidate was scored on every IMSEC oracle cell of gen-m9,
    ironcalc-calc, probes and probes-numeric, counting texts fixed and older cells lost (IMREAL and IMAGINARY of
    them included). The rule: inside the `1/u` band of `sec_sinh_cosh` (0.01 < |y| <= 20) sinh²y is
    `(cosh 2y − 1)/2`, and outside it `(cosh 2y + 1)/2 − 1` as before. This fixes all 4 cells and loses none, so
    all 34 fused texts fit. Switching on the band of 2y instead ties. Excel's split is VERIFIED only as far as the
    cells go: rows 29 and 33 of im-last-digit-sweep (|y| = 0.00977722 and 0.00387493, below the band) need
    `(cosh 2y + 1)/2 − 1`, and every fused text (|y| inside the band) is fitted by `(cosh 2y − 1)/2`. Where the
    switch lies between 0.00977722 and the smallest fused |y| is the band's own unprobed end (T-2797).
    Losing candidates (fixed, lost):
    - `(cosh 2y − 1)/2` everywhere: 4, 3 (G33).
    - The same with the EXP pair for cosh 2y outside the band: 4, 3.
    - With IMSINH's `1/u` cosh 2y: 4, 11.
    - The whole kernel in x87 extended registers, stored once: 4, 25.
    - It with `t` stored to a double first: 4, 35.
    - It with cosh y, sinh y and cosh 2y also unstored `1/u` registers: 4, 50.
    - `cos²x` unrounded (extended) in the sum: 4, 36. `fma(cos x, cos x, ·)`: 4, 36.
    - Half-angle `t = (cos 2x + cosh 2y)/2`: 4, 19. `(1 + cos 2x)/2` for cos²x: 4, 11.
    - Fit neither cell: extended numerators over the double `t` (either division order, Excel's unrounded COS, or
      the unstored `1/u` parts), `(cos²x + cosh²y) − 1`, `(cos²x − 1) + cosh²y`, multiplying by `1/t`, and libm's
      cos x.
- IMCOT: `(sin 2x, −sinh 2y) / ((cosh 2y + 1) − 2cos²x)`: all 63 cells with a nonzero imaginary part, including the
  tiny-argument rows (`cosh 2y − cos 2x` misses row 31 by one unit of the 12th digit), and gen-m9's
  `IMCOT("-1.5+0.5i")` -0.0557109884865415-0.463942860183828i (45 cells, T-2537). The grouping matters only
  there: T-2044's `cosh 2y − (2cos²x − 1)` also fits the 63 but prints ...5416 for it, and so do
  `cosh 2y − cos 2x`, `cosh 2y − (1 − 2sin²x)` and `(cosh 2y − 2cos²x) + 1`. Exp-based cosh fits the same
  (scratch fit over the sweep, T-2537). Past `cosh 2y`'s range the
  value is `−i` for y > 0 and `i` for y < 0 (P27, P44, P49, P68, P69, P71; P52, P55). A real number keeps `1/tan x`.
  T-2787 probed 24 more inputs where the two groupings print different text (probes/t2787-imcot-denominator-grouping,
  excel-graph, 2026-10-28, libm trig): `(cosh 2y + 1) - 2cos²x` matches 21, T-2044's `cosh 2y - (2cos²x - 1)` 2 (A2, A9,
  the cells ours misses in the last digit; A17 fits neither). The grouping was not the cause: the three cells need
  IMSEC's sinh 2y and cosh 2y (`sec_sinh_cosh`: IMSINH's `1/u` kernel for 0.01 < |2y| <= 20, libm's outside) in
  place of libm's. A2 and A17 need the kernel's sinh 2y (A17's imaginary part fits no other sinh and no cosh or cos²x
  within 3 ulp), A9 its cosh 2y, one ulp above libm's. With `(cosh 2y + 1) - 2cos²x` that fits all 24 cells, the 54
  sweep cells with a finite imaginary part (the tiny-y rows sit below the band, where libm's sinh is the kernel's) and
  gen-m9's `-1.5+0.5i`, 79 of 79 (scratch fit with the engine's own primitives). Evaluating the denominator in x87
  extended precision fits 61 at best, so the arithmetic is plain doubles. Status: VERIFIED (excel-graph); implemented
  (T-3032).
- IMABS: `a·sqrt(1 + (b/a)²)` with a the larger of |x|, |y|: 77 of 78 cells bit for bit (`hypot` 59,
  `sqrt(x²+y²)` 55, scaling by |x| or by |y| always 66); the miss is B33, `956.615+0.00387493i` (Excel
  956.6150000078482, ours 2 ulps lower). IMPOWER's modulus is still `hypot` (T-1950 evidence). A part near DBL_MAX overflows the scaled
  form to infinity as `hypot` does (`IMABS("1E308+1E308i")`); that edge is UNVERIFIED.
- A real argument skips the kernels: IMCSC is `1/sin x`, IMSEC `1/cos x`, IMSECH `1/cosh x` (gen-m9/2f0d384b8ed0d392
  A109 and a7fda34502ed3030 A146, `IMCSC(0.2)` 5.03348954767234, where the IMCSC kernel gives ...235; gen-m9/01810fc89c5ec2c0
  A318, `IMSECH("1.5")` 0.42509603494228; every real row of the sweep agrees).
- IMARGUMENT: ATAN2's kernel, `atan(y/x)` plus or minus π (numbers.md "ATAN2 is ATAN(y/x) plus or minus π"): 77 of 77
  (libm `atan2` 67). The overflow and subnormal edges of that kernel are UNVERIFIED for IMARGUMENT.
- IMCSC and IMCOT of a tiny imaginary part (probes-numeric/im-csc-cot-tiny-imag, T-2531, excel-graph, 56 of 56
  cells agree with ours): `cosh 2y` rounding to 1 is Excel's behavior too. `IMCSC("1E-9i")`, `IMCSC("1E-20i")`,
  `IMCOT("1E-8i")`, `IMCOT("1.1E-8i")`, `IMCOT("1E-9i")`, `IMCOT("1E-9+1E-9i")` and `IMCOT("1E-20i")` are `#NUM!`;
  `IMCSC("1E-8i")` is `-90071992.5474099i`, `IMCSC("1.1E-8i")` `-99079191.8021509i`, `IMCSC("1E-9+1E-9i")`
  `1000000000-1000000000i`; `IMCOT("1E-6i")` is `-999911.107320937i` (only a few digits survive between 1E-8 and 1E-6).
- UNVERIFIED: for |x| ≥ 2^27 (SIN's own `#NUM!` cut) the reciprocal kernels take libm's sine and cosine.
- IMLOG2 (T-2231, superseding the T-2044 time-box): `ln` of IMABS's scaled modulus and the argument (`atan2`),
  both times K = 1.4426950408889636, the double one ulp above log2 e (= `1/ln 2` with the double one ulp below ln 2).
  It fits all 103 distinct real and 103 imaginary parts of IMLOG2 cells in the eval suites (sweep N/O, COMPLEXs AH19,
  eng-im-overflow-edges A104-A109, gen-m9, eng-im-overflow-edges-2 A35-A44). The earlier candidate, dividing
  `ln |z|` by the double below ln 2, fails on A108 `IMLOG2("712+3i")` because the division and the multiplication
  round differently; `hypot` as the modulus misses only `-0.66925-0.233532i` (sweep row 53). `log2`, `ln/LN_2` and
  `ln·log2 e` fit 92-94 of the 103 real parts.
- IMLOG10's imaginary part is `atan2 / LN_10`: `atan2·log10 e` misses `IMLOG10("711+1.5i")` (0.000916231724847909,
  eng-im-overflow-edges A95) and `atan2·(1/LN_10)` misses `IMLOG10("1.5+710.4i")` (0.68127117135401); the division fits
  all 36 IMLOG10 cells (T-2231). Its real part stays `log10 |z|` (`ln/LN_10` fits the same cells).
Evidence: probes-numeric `im-last-digit-sweep` (excel-graph, 2026-10-21); COMPLEXs.xlsx G7, G8, G14, G23, H14, H23,
T14, V4, V15, W4, W7, W21, Z25, AB7; probes/eng-argument-modes A45; probes/eng-im-overflow-edges A47.
Status: VERIFIED (excel-graph)
Implemented in: `imcsc`, `imsec`, `imsech`, `imcot` in functions/engineering/complex/trig.rs, `imabs`, `imargument` in functions/engineering/complex/mod.rs (T-2044); `imlog2_parts`, `imlog10_parts` (T-2231)
Tasks: T-2044, T-2231

## IMTAN and IMCOT use the double-angle form (T-0907)
Rule: IMTAN computes tan(x+iy) as `(sin 2x + i·sinh 2y) / (cos 2x + cosh 2y)`, and IMCOT is the
reciprocal of that value, `conj(t)/|t|²`; the tan/tanh product form loses the last 1-10 digits
where the real part is small (`IMTAN("1.23+9.5121i")` real part 6.89106871662165E-09, the product
form 6.89106877738345E-09; `IMCOT("7-12.6i")` real part 2.25273756035673E-11). A fit over every
row of the workbook: the double-angle IMTAN matches all of them, the reciprocal IMCOT all but one
(T14, last digit; Smith's division misses T7 too), while `sin z / cos z` and `cos z / sin z` miss
most. IMCOT is now the T-2044 form above (no longer the reciprocal of IMTAN). IMCOT of a real number keeps `1/tan x` (`IMCOT("1E-20")` = 1E+20, where the double-angle
denominator is 0).
Remaining after T-0907 (19 cells of COMPLEXs.xlsx, all 1-2 ulps or the 15th digit; T-2044 fixed all but IMLOG2 AH19 and the desktop-only H2, H3, H10, K25): IMABS (G7,
G8, G14, G23; neither `sqrt(x²+y²)` nor `hypot`), IMARGUMENT (H2, H3, H10, H14, H23; neither
`atan2` nor `atan(y/x)` ± π for all of them), IMCSC (V4, W4, W7, V15, W21), IMSEC (Z25), IMSECH
(AB7), IMCOS (K25), IMLOG2 (AH19) and IMCOT T14. The workbook's values come from Macintosh Excel,
and the oracle calibration found IMARGUMENT differing by ulps between Excel for the web and desktop
(docs/architecture/oracle.md), so these may be platform libm digits.
- IMSEC and IMSECH do not share a kernel: `sech(3+4i)` and `sec(4+3i)` have the same real part mathematically, yet
  `IMREAL(IMSEC("4+3i"))` is -0.0652940278579471 (COMPLEXs.xlsx Z25, Mac cache, which the web agrees with) and
  `IMSECH("3+4i")` prints -0.065294027857947 (probes/eng-argument-modes Calc!A45 and gen-m9/a2e51f48717af2a1
  Calc!A24, excel-graph). Ours gives the other digit in both. The remaining cells are now T-2044 (sweep T-2043);
  IMARGUMENT H2, H3, H10 and IMCOS K25 are desktop/web differences (docs/research/oracle-calibration.md), not fitting
  targets until the desktop oracle exists.
- Where `cosh 2y` overflows (|y| above about 355) IMTAN gives `i` (`-i` for a negative imaginary part) and
  IMCOT `-i`, never `#NUM!`: IMTAN("1+400i") = "i", IMTAN("1-400i") = "-i", IMCOT("1+400i") = "-i",
  IMTAN/IMCOT("1+355i") and ("1+356i") the same; at y = 354 the real part is still the tiny
  `6.01509887059592E-308` (IMTAN("1+354i") = "6.01509887059592E-308+i"). Excel-verified
  (probes/eng-imtan-imcot-overflow A1-A9, excel-graph), and RustSheet matches all nine.
- For a real argument at or near the double nearest π/2 Excel gives a finite value, not `#NUM!`:
  IMTAN("1.5707963267948966") = "148752068975966", IMTAN("1.5707963267949") = "-286413440886091",
  IMCOT("1.5707963267948966") = "6.72259557049637E-15", IMCOT("1.5707963267949") = "-3.49145625605507E-15"
  (probes/eng-imtan-imcot-overflow A10-A13, excel-graph). The text "1.5707963267948966" reads as 1.57079632679489
  (15 digits), 6.7E-15 below π/2. These are TAN's x87 kernel, reduced by the 64-bit π (numbers.md "SIN / COS at multiples of 90°"): the remainder is
  `(FRAC_PI_2 - x) + 6.1257422745431E-17`, π₆₄/2's tail, and both pairs of values are 1 over the tangent of it to every
  printed digit, while the true π (libm's `tan`) gives 148752623989354 and -286411383293069 from the sixth or seventh
  digit. Here IMTAN's double-angle denominator `cos 2x + cosh 2y` rounds to 0 (x within about 5.2E-9 of π/2), and IMTAN
  falls back to TAN's kernel (only these two points are probed; the window's edges and other odd multiples of π/2 are
  not); IMCOT of a real number is 1 over it anyway (T-2534). Since T-2760 all four match.
Evidence: ironcalc-calc/ENGINEERING/COMPLEXs.xlsx (Excel-cached, Microsoft Macintosh Excel) N, O,
T, U columns; probes/eng-argument-modes Calc!A33 `IMCOT("3+4i")` = "-0.000187587737983659-1.00064439247156i"
and A51 `IMTAN("3+4i")` = "-0.000187346204629478+0.999355987381473i" (excel-graph), which the
double-angle form matches too, so the digits are not Mac-only.
Status: VERIFIED (desktop cached values and excel-graph)
Tasks: T-0907, T-1966, T-2760, T-2043, T-2044

## Base conversions read a blank cell and empty text as 0
Rule: every BIN2*/OCT2*/HEX2* function (with or without `places`) treats a blank cell, `""` and a ref to `=""` as 0: `BIN2DEC("")`, `OCT2DEC(blank)`, `HEX2DEC("")` are 0; the others give "0", or "0000" with `places` 4. None of these is an error.
Evidence: probes/eng-base-conv-blank (excel-graph, 2026-10-18, T-1413); gen-m9/d244411dd90f7fbd A15.
Status: VERIFIED (excel-graph)
Tasks: T-1413

## The second argument's error wins in the two-argument base conversions, DELTA and GESTEP
Rule: DEC2BIN, DEC2HEX, DEC2OCT, BIN2OCT, BIN2HEX, HEX2BIN, HEX2OCT, OCT2BIN, OCT2HEX, DELTA and GESTEP read the second argument (`places`, `number2`, `step`) before the first, scalar or inside one array element. `DEC2BIN("x",NA())` is `#N/A` and `DEC2BIN(NA(),"x")` is `#VALUE!`; the same for all eleven. BESSELJ, BITLSHIFT, CONVERT, COMPLEX, IMPOWER and ERF instead report the first argument's error. In the nine base conversions an invalid places value wins too, not only its error: `DEC2BIN(NA(),0)` is `#NUM!` (gen-m9 sweep, "Base conversions" below; T-2545). Whether DELTA's and GESTEP's second argument also wins by value is unprobed.
Evidence: probes/eng-first-vs-second-error A1:D11 (excel-graph, 2026-10-19); probes/eng-later-argument-lift M2, M8.
Status: VERIFIED (excel-graph)
Tasks: T-1816, T-2545

## IM* results that overflow (T-1827)

Rule: for IMSUM, IMPRODUCT, IMEXP, IMPOWER and IMDIV (the functions probed), a result that
overflows or is NaN is an error, never text. IMSUM and IMPRODUCT give `#VALUE!`; IMEXP of a pure
real number, IMPOWER and IMDIV give `#NUM!`. The other IM* functions are UNVERIFIED.
Status: VERIFIED (excel-graph); implemented for the five (IMEXP with a nonzero imaginary part: T-2002, T-2060).
Tasks: T-1827, T-1996, and the follow-ups T-2001 to T-2003.
Evidence: probes/eng-im-overflow (excel-graph).

- Examples: `IMPRODUCT("1E200","1E200")`, `IMSUM("9E307","9E307")` and
  `IMPRODUCT("1E200","1E200","0")` are `#VALUE!` (A5-A7, A25, A30, A36-A38); `IMEXP("710")`,
  `IMPOWER("1E155",2)` and `IMDIV("1E300","1E-300")` (A21) are `#NUM!`.
- The limit is the double range: `IMPRODUCT("1E154","1E154")` is `1E+308`, `IMEXP("709.7")` is
  `1.65498402768026E+308`, `IMPOWER("1E154",2)` is `1E+308`.
- IMEXP with a nonzero imaginary part does not overflow to an error: `IMEXP("1000+1i")` is
  `9.71297746009434E+307+1.51270661257499E+308i` (A12). The modulus is capped at DBL_MAX (the
  ratio of the parts is tan(1) and re / cos(1) is 1.7977E+308), not the mechanism of the values
  near 1.797E+308 of IMPOWER, which are `|z|^n·DBL_MAX` past its angle cut ("IMPOWER past the angle cut", T-1950). Implemented (T-2002, T-2060). Probes
  (probes/eng-imexp-overflow-imag, excel-graph): the cap holds for a negative imaginary part
  (`IMEXP("1000-1i")`), for `709.9+1i` (exp overflows only past 709.78) and for `1000+3.14159i`
  (`-1.79769313485599E+308+4.770340154251E+302i`), down to `1000+1.5E-8i`; but `IMEXP("710+1E-300i")`,
  `"1000+1E-300i"`, `"1000-1E-300i"` and every imaginary part up to `1.4E-8` are `#NUM!`. The
  boundary is exactly 2^-26 = sqrt(DBL_EPSILON) = 1.4901161193847656E-8: `1000+1.4901161193847E-8i` is
  `#NUM!` and `1000+1.4901161193848E-8i` is capped (probes/eng-imexp-overflow-cutoff A1, A2; `1.49E-8`
  and `1.4901E-8` error, `1.4902E-8` caps). The window is independent of x (`800`, `709.9`:
  eng-imexp-overflow-cutoff-pi A21-A24). Exactly 2^-26 is `#NUM!` too, so the cap needs y strictly past it (probes/eng-imexp-cap-window A35),
  and so does a negative y (A36, A37). The same kind of window applies around every other multiple
  of pi/2, but is smaller: for pi/2, pi and 2pi, below and above, the boundary lies in
  (1.053E-8, 1.054E-8] (A1-A30), and for 100pi in (1.05E-8, 1.06E-8] (A31-A34): `1.053E-8` from the multiple is `#NUM!`, `1.054E-8` is capped. That is
  2^-26.5 = 2^-26/sqrt(2) = 1.0537E-8, where cos(d) = 1 - d^2/2 stops being distinguishable from 1;
  implemented as that. The exact value is within the bracket but not pinned to the bit.
- IMEXP's sine and cosine of the imaginary part reduce the argument as SIN and COS do (the x87
  64-bit pi, see the IMPOWER entry above): `IMEXP("1000+3.14159i")` is
  `-1.79769313485599E+308+4.770340154251E+302i` (probes/eng-imexp-overflow-imag A3) and
  `IMEXP("710i")` is `0.999999998182636+0.0000602887066915739i` (probes/eng-im-overflow A42); libm's
  sin gives other last digits. Implemented with `excel_sin_cos` (T-2060). `IMEXP("1000+134217727i")` is
  `-5.27422318923931E+307-1.7185826441088E+308i` but every |y| >= 134217728 (`1000+134217728i`, `1000-134217728i`, `2E8`,
  `1E9`, `1E15`) is `#NUM!` (`1000-134217727i` answers, A39-A40), even when x is small (`IMEXP("1+134217728i")`;
  probes/eng-imexp-overflow-cutoff A17-A24); implemented as the SIN limit (T-2087).
- A part near DBL_MAX is written as `1.79769313486232E+308`, a decimal above DBL_MAX: the 15-digit
  rounding of 1.7976931348623157E+308 is not clamped (probes/eng-imexp-overflow-imag A22, A24, A27,
  A30). Ours keeps the unrounded part when the rounding overflows (T-2060). Numeric DBL_MAX (`=2^1023*(2-2^-52)&""`) is `1.7976931348623E+308` (14 digits, not the
  IM* text's 15) and `TEXT(..., "0.00E+00")` is `1.80E+308` (probes/dbl-max-text-a, -b; ours differs for the TEXT form, T-2106). A DBL_MAX
  literal in a formula makes excel-graph reject the workbook; whether the formula `=DBL_MAX literal`
  equals the computed value is unprobed.
- IM* text keeps 15 significant digits for an exponent past 98, unlike `&""` (numbers.md): A11,
  A17 (`IMPOWER("1+i",2000)`), A40 and A41 print `8.21840746155497E+307`, `1.35498631931463E+308` and
  `1.65498402768026E+308`, which ours now matches (T-2001: `complex_part_to_text` keeps 15 digits). This is a
  formatting difference, not T-0907 precision.
- IM* text rounds each part's double to 15 significant digits once, correctly, as `to_precision` does. It does not
  round a decimal string again: rounding half-up from the shortest-repr string would fix probes/eng-im-overflow-edges
  A17 but `rustsheet-eval check` reports 21 regressions in probes and 19 in ironcalc-calc, and rounding from the
  17-digit string regresses 1 in each. Status: VERIFIED against the existing oracle cells (2026-10-26, T-2629 notes).
- IM* text writes a small part in full past 20 characters: `IMEXP("710i")` is
  `0.999999998182636+0.0000602887066915739i` (A42). Ours now writes a part with decimal exponent -5 or above in plain notation whatever its
  length (exponent -5 is plain by A42 and -16 is E notation by A9; -6 to -15 are all E notation, `IMSUM("1.23456789012345E-6","0")` is `1.23456789012345E-06`, while -1 to -5 are plain up to `0.0000123456789012345`, an imaginary part likewise: probes/eng-im-small-part A1-A16, B1-B12, excel-graph, T-2056; the 15-digit rule at negative exponents of -99 or below is also unprobed); A42 still
  fails: the digits differ since Excel reduces a large sine or cosine argument differently (ours
  `...915853`).
- An input part of `1E308` or more is `#NUM!` in every IM* function (`IMREAL("1E308")`,
  `IMSUM("1E308","0")`); ours already matches. `IMSQRT("1E307")` and `IMSQRT("4E307")` are `#NUM!`
  too (A33, A34). The threshold per part lies between 1.34E154 and 1.35E154, consistent with sqrt(DBL_MAX) = 1.34078E154 (exact boundary unprobed): IMSQRT squares each
  part, and a part past it is `#NUM!` (`"1.34E154"` is `1.15758369027902E+77`, `"1.35E154"`,
  `"1E160i"`, `"-1E160"`, `"1E154+1.4E154i"` are `#NUM!`). When each square is finite but their
  sum overflows, the result is `0`, not an error: `IMSQRT("1E154+1E154i")`, `"-1E154-1E154i"`,
  `"1.34E154+1.34E154i"` (probes/eng-imsqrt-large, eng-imsqrt-large-b, excel-graph, T-1996).
  Implemented in `imsqrt`.
- `IMDIV("1","1E-308")` is `#NUM!` (A20) although results near 1E+308 are text (A46); the cause
  (subnormal divisor?) is unprobed, ours matches.
- IMSIN, IMCOS, IMSINH and IMCOSH: a result part that overflows or is NaN is `#NUM!` (`IMCOSH("1000")`,
  `IMSINH("1000")`, `IMSIN("1+1000i")`, `IMCOS("1+1000i")`), as IMEXP of a real number (T-2003).
- IMSUB: a part that overflows is 0 and the other part stays, no error: `IMSUB("9E307","-9E307")` and the `j` form are "0",
  `IMSUB("9E307+1i","-9E307")` is "i", `IMSUB("1+9E307i","1-9E307i")` is "0"; `IMSUB("5E307","-5E307")` is `1E+308`
  and an input part of `1E308` is `#NUM!` as everywhere (T-2003).
- IMSUM and IMPRODUCT: the overflow is `#VALUE!` the moment it happens, so it beats a later bad argument
  (`IMSUM("9E307","9E307",1,"x")`, `IMPRODUCT("1E200","1E200","x")`), a later cancelling term
  (`IMSUM("9E307","9E307","-9E307")`) and a later blank cell (`IMPRODUCT("1E200","1E200",<blank>)`, not "0") (T-2003).
Evidence: probes/eng-complex-overflow A1-A21 (excel-graph, 2026-10-19). Implemented in complex.rs
(`finite_complex_result`, `fn_imsub`, `fold_complex`).

Edges probed by T-2077 (probes/eng-im-overflow-edges, excel-graph):

- A blank cell argument met earlier in IMPRODUCT (a blank inside a range is UNVERIFIED, T-2231) makes the running product 0, so a later overflow cannot
  happen: `IMPRODUCT(<blank>,"1E200","1E200")` is `0`; the blank last is `#VALUE!` (the overflow
  came first). Implemented.
- IMSIN and IMCOS are `#NUM!` when `exp(|imaginary part|)` overflows (|y| above about 709.78 by analogy with SINH/COSH in numbers.md; probed only at 710.4 and 711, T-2231),
  whatever the real part (`IMSIN("1.5+710.4i")`, `IMCOS("710.4i")`), although `cosh y` is still
  finite there. IMCOSH and IMSINH are `#NUM!` for a zero imaginary part when `exp(|x|)` overflows
  (`IMCOSH("710.4+0i")`) but answer text with a nonzero one (`IMCOSH("710.4+1.5i")` is
  `1.17873946358278E+307+1.6621900184285E+308i`); past cosh's own range it is `#NUM!`
  (`IMCOSH("711+1.5i")`). A tiny nonzero imaginary part with a large real one is UNVERIFIED. Implemented.
- IMSEC, IMCSC, IMSECH and IMCSCH answer `0` (both parts) where the denominator overflows and the true result is
  subnormal (`IMSECH("711+1.5i")`, `IMCSCH("710.4+0i")`, `IMSEC("1.5+711i")`); a normal tiny result is kept (IM* last-digit sweep, T-2335); a zero denominator stays
  `#NUM!` (COMPLEXs.xlsx Sheet1 V22:Y22). Implemented (IMCSC of 0 since T-2334, IMCSCH of 0 and `IMCSCH("1E-200+1E-200i")` since T-2540: Excel-verified in gen-m9; a tiny real-only or imaginary-only IMCSCH such as `IMCSCH("1E-200")` also gives `#NUM!` here: UNVERIFIED). A denominator that underflows to 0 for IMCSC (`IMCSC("1E-200")`) and IMCOT (`IMCOT("1E-310")`) is also `#NUM!` here: UNVERIFIED.
- IMTAN and IMCOT near the edge answer `i` / `-i` (`IMTAN("1.5+710.4i")`); IMLN, IMLOG10, IMLOG2
  and IMSQRT do not overflow there. Already matched.
- Last-digit differences remain as known failures (a kernel question, not a rule; T-2231): IMCOSH/IMSINH
  `710.4+1.5i` (several units in the 15th digit), IMCOT `711+1.5i`, IMLOG2 and IMLOG10 at 710-800. IMCOSH/IMSINH
  `1.5+710.4i` and IMSECH `1.5+711i` are fixed by the `1/EXP(-x)` kernel ("IMSINH, IMCOSH and IMSECH take sinh and
  cosh as (1/u ∓ u)/2 with u = EXP(-x)", T-2629).

## Engineering edge-pool smoke check (T-2249)
Rule: observed values at the edges of the M9 engineering generator pools (tools/src/rustsheet_tools/gen/pools.py):
- COMPLEX's suffix: empty text is the default `i` (`COMPLEX(3,4,"")` is "3+4i"); "I", "J", "k", "ij", " i", 1 and TRUE
  are `#VALUE!`.
- An inumber may start with `+`: `IMABS("+i")` is 1 and `IMSUB("+i","1")` is "-1+i" (ours `#NUM!`, T-2420).
- OCT2DEC of 11 octal digits given as a number (`OCT2DEC(77777777777)`) is `#NUM!` (ours 7516192767, T-2420); OCT2DEC,
  OCT2BIN, HEX2DEC, HEX2BIN and HEX2OCT of TRUE are `#VALUE!` (ours `#NUM!`, T-2420).
- HEX2BIN's negative range ends at FFFFFFFE00 ("1000000000"); FFFFFFFDFF and FFE0000000 are `#NUM!`.
- `places`: a fraction truncates (`DEC2BIN(9,2.9)` is `#NUM!`, below the length; `DEC2BIN(9,10.5)` "0000001001"), "4"
  is read, 0, -1 and 11 are `#NUM!`, "x" and TRUE `#VALUE!`; a negative result ignores places 1 to 10 and 2.9 alike.
- `BESSELI(713,0)` and `(714,0)` are `#NUM!`; `BESSELK(713,1)` and `(714,1)` are 0 (ours a subnormal, T-2421).
  BESSELI past our order cap is 0 (`BESSELI(2.5,100001)`, `(2.5,1E+6)`), as BESSELJ is ("BESSEL order cap"; ours matches since T-2546).
  BESSELJ and BESSELY at x = 1000 and 1E+5 miss by a few ulps (x above 100 is T-2140's).
- `ERF(5.9)` is 0.9999999999999999 and `ERF(6)` 1; `ERFC(26.5)` is 2.2109076642637346E-307 and `ERFC(27)` 0.
- BIT*: 2^48-1 is the largest number (2^48 is `#NUM!`), a fraction or a negative number is `#NUM!`, "13" is read
  and TRUE is 1. BITLSHIFT past 2^48-1 is `#NUM!` (`BITLSHIFT(2^47,1)`); a fractional shift truncates
  (`BITLSHIFT(1,1.5)` 2, `(1,-1.5)` 0). `BITLSHIFT(1,48)` and `(1,54)` are `#NUM!`, but `(1,53)` and `(1,-53)` are 0
  (ours `#NUM!`, T-2421), and `(1,-54)` is `#NUM!`.
Evidence: a scratch suite of 790 cells (one to four formulas per entry of the pools complex_text, complex_suffix,
bin_digits, oct_digits, hex_digits, dec_number, places, bessel_x, bessel_order, convert_unit, erf_x, bit_number,
bit_shift, db_table, db_field and db_criteria), filled through excel-graph on 2026-10-22 and deleted afterwards, as
the task asked. Every cell had a value (none pending). Ours: 760 of 790 match. The rest are listed here, in
criteria.md "Database edge-pool smoke check (T-2249)", and the known DEVIATION and last-digit families named above.
Status: UNVERIFIED (probe pending: T-2420 and T-2421 commit probe cases for the mismatches and the BESSELI
cap cells; the cells that match are regenerated by the gen-m9 sweep, T-2258)
Tasks: T-2249, T-2420, T-2421, T-2258

## Engineering functions in the gen-m9 engineering sweep (T-2258)
Rule: what the 601 new failing cells of the T-2258 sweep (seeds 2258, 22580, 22581; cases listed in the owning tasks)
show about Excel. Each item names the task that matches it. "Ours" is what RustSheet gave on 2026-10-23.
- **COMPLEX reads its suffix first.** A suffix other than "i", "j" or empty text (a number, TRUE, "J", "a", "ü", other
  text) is `#VALUE!`, and an error suffix is that error, even when real_num or i_num is an error:
  `COMPLEX(<#NUM!>,0,"J")` (060b18586fcff6e6 A144), `COMPLEX(59,KURT(..)<#DIV/0!>,"J")` (6d16afd40600302b A112),
  `COMPLEX(<#NUM!>,-1,TRUE)` (2b0324e2de2e7b74 A62), `COMPLEX(-0.878..,<#N/A>,<100 cell>)` (7dc52cfcbc8b4448 A59) are
  `#VALUE!`; `COMPLEX(<#NUM!>,-2.5,<#N/A>)` (408aa11aa923a4d9 A119) is `#N/A`, `COMPLEX(61,<#NUM!>,<#CALC!>)`
  (e882060cf4613302 A52) `#CALC!`, `COMPLEX(61,<#VALUE!>,<#REF!>)` (b5aef3645cec1b95 A346) `#REF!`. Ours reports
  real_num's or i_num's error. The order between real_num and i_num is unchanged (the first argument's, "The second
  argument's error wins ..." above). T-2532.
- **"1i" when a part is not exactly 1.** IM* text omits the coefficient only for an imaginary part of exactly ±1; a part
  that rounds to 1 at 15 digits keeps it: `COMPLEX(0.1,1+2^-52,"i")` is "0.1+1i", `COMPLEX(1/3,1-2^-53)`
  "0.333333333333333+1i", `COMPLEX(1+2^-52,1-2^-53,"j")` "1+1j", `IMDIV("5","-1.5+0.5i")` "-3-1i",
  `IMPOWER("1000+1i",1)` "1000+1i" (493bb194c8e0b577 A331, 62c886e21a1042af A243, 7dc52cfcbc8b4448 A247, ...). Implemented
  for the text of COMPLEX and IMDIV (T-2533): the coefficient is compared unrounded.
  IMPOWER's angle is Excel's ATAN2 kernel `atan(y/x)` (IMARGUMENT's), not libm `atan2`: for `IMPOWER("1000+1i",1)`
  `atan(0.001)` is 0.0009999996666668668, one ulp above `atan2(1,1000)`, so the imaginary part
  `hypot(1000,1)·sin(angle)` is 1.0000000000000002 and prints "1i" (libm's angle gives exactly 1, "1000+i"; gen-m9
  74779cff2810ccf4 A379). Every other IMPOWER cell in the suites is unchanged by the switch. Implemented (T-2610);
  unprobed (T-2736): where `atan(y/x)` errs (x zero or subnormal, y/x overflowing) ours keeps libm `atan2`; the
  kernel's flush of a subnormal angle to 0 now makes `IMPOWER("1E20+1E-300i",1)` "1E+20" (libm's angle keeps the 1E-300 part),
  and x < 0 with a -0 imaginary part takes +π where `atan2` took -π.
- **IM* trigonometry of a real number is SIN, COS and TAN.** With a zero imaginary part, IMSIN, IMCOS, IMTAN, IMSEC,
  IMCSC and IMCOT give exactly the worksheet functions' values (x87 reduction, numbers.md "Trig of a huge angle"), as the
  engine's own SIN/COS/TAN reproduce: `IMSIN(123456789.123)` 0.99984293955201 (= SIN; libm 0.999842939552045),
  `IMCOS(..)` 0.0177227601686489, `IMTAN(..)` 56.4157574800738, `IMSEC(..)` 56.424619556099 (= 1/COS), `IMCOT(..)`
  0.0177255441505541 (= 1/TAN), `IMSIN("710")` 0.0000602887066915739, `IMCOT("710")` 16586.8543722202, `IMCSC("710")`
  16586.8544023646, `IMSIN(3.14159)` 2.65358979335278E-06, `IMTAN(3.14159)` -2.65358979336212E-06 and `IMTAN("5")`
  -3.38051500624659 (35 cells; COMPLEXs.xlsx N6, a Mac cache, has -3.38051500624658, see "IMTAN and IMCOT use the
  double-angle form"). Past TAN's limit they are `#NUM!`: `IMSIN("1E+200")`, `IMTAN("1E+200")`, `IMSEC("1E+200")`,
  `IMCSC("1E+200")`, `IMSIN("7777777777")` (`IMSIN(HEX2OCT("FFFFFFFFFF"))`, 1068c44afcd56d53 A57), IMSEC of 1E+15 and
  `IMTAN("1.05966105763914E+57")`; but `IMCOT("1E+200")` is "0" (060b18586fcff6e6 A119). probes/t2534-im-trig-real-limit
  (excel-graph, 2026-10-26) confirms the limit for all six at 2^27: at 134217727 they answer, and at ±134217728, 1E+10,
  1E+15 and ±1E+200 five are `#NUM!` and IMCOT is "0". probes/t3054-imtan-real-tan-kernel (excel-graph, 2026-10-28)
  settles IMTAN's kernel on the web: nine real arguments where TAN's kernel and the double-angle form
  `sin 2x / (cos 2x + 1)` print different text are all TAN's (`IMTAN("3.14159")` -2.65358979336212E-06, double-angle
  ...207E-06; `IMTAN("-17.2737")` -197.642601323436, ...305; `IMTAN("-17.3011")` 44.7544983375159, ...163; row 3,
  `IMTAN("4")`, prints 1.15782128234958 either way), and so is row 1, COMPLEXs.xlsx row 6's own
  `IMREAL(IMTAN(COMPLEX(5,0,"i")))`, -3.38051500624659. IMTAN of a real is still CONFLICTING (web vs desktop):
  COMPLEXs.xlsx N6 (Mac 16.0300) caches that formula as -3.38051500624658, the libm double-angle value (T-1075 item 17). IMTAN is one of numbers.md's "Excel for the web and desktop Excel
  differ in the last digits" functions, so decision 0030 rule 4 (kept by decision 0039) makes the desktop N6 the fitting
  target and the web cells, which the double-angle form misses by far less than 1e-9 relative, no target for further
  fitting until T-1075 item 17 is answered (T-3054, dropped into T-2774).
  Ours (T-2534): IMSIN, IMCOS, IMSEC, IMCSC and IMCOT of a real take SIN, COS and TAN's kernels and the limit
  (`real_trig`); IMTAN takes the limit but keeps the double-angle form below it, so the desktop N6 still passes and the 35
  web `IMTAN("5")` cells and the other IMTAN-of-a-real cells stay known failures (T-2774).
- **sinh and cosh inside IMSIN and IMCOS are (EXP(t) ∓ EXP(-t))/2, also for small t; IMSINH's are not.**
  `IMCOS("1.5E+2-2E-3i")` has imaginary part -0.0014297538124271 (= -SIN(150)·(EXP(-0.002)-EXP(0.002))/2; with libm
  sinh ...242709), and `IMSIN("1E-200+1E-200i")` is "1E-200": the imaginary part cos(x)·sinh(1E-200) is 0 because the
  two EXPs are equal. Implemented (T-2535: `exp_sinh_cosh`, 4 gen-m9 cells). IMSINH keeps a tiny real part
  (`IMSINH("1E-200+1E-200i")` is "1E-200+1E-200i", gen-m9/40fc68fccfaf9b76 A155), so it is not the EXP form, and
  `IMSINH("4")` is 27.2899171971278 (33 gen-m9 cells; `IMSEC(IMSINH("4"))` -1.80717748625735 follows from that
  text) while `IMSINH("-4")` is -27.2899171971277. The complex text is not the cause:
  `COMPLEX((EXP(4)-EXP(-4))/2,0)` prints 27.2899171971277 and `IMREAL(IMSINH(4))` is 27.2899171971278, so IMSINH's
  double at 4 is 27.289917197127753, one ulp above (EXP(4)-EXP(-4))/2 = libm sinh = 27.28991719712775
  (probes/t2535-complex-text-tie). Over 43 real arguments (probes/t2535-imsinh-imcosh-kernel-sweep, excel-graph
  2026-10-26) libm sinh misses only 4; the EXP form misses 4 and 0.01; cosh − EXP(-x) misses 12 and 0.01; a correctly
  rounded sinh misses -4, ±12 (and `"12-3i"`), where the web agrees with the Mac desktop cache of COMPLEXs.xlsx P12 and
  P21. No formula symmetric in EXP(x) and EXP(-x) gives both ±4. IMCOSH matches libm cosh on all 62 inputs. The
  kernel is `1/EXP(-x)` (next entry, T-2629). Status: VERIFIED (excel-graph).
- **IMSINH, IMCOSH and IMSECH take sinh and cosh as (1/u ∓ u)/2 with u = EXP(-x) (T-2629).** The real part x
  enters through `u = EXP(-x)` and its reciprocal: sinh x = `(1/u - u)/2`, cosh x = `(1/u + u)/2`, and the imaginary
  part y through SIN and COS's own kernels (x87 reduction). The reciprocal makes the kernel lopsided in x: at 1.5
  `1/EXP(-1.5)` is 4.481689070338065, one ulp above EXP(1.5), so sinh(1.5) and cosh(1.5) come out 2.1292794550948178 and
  2.3524096152432477 (libm + 1 ulp each), while at -1.5 both are libm's. This is what makes `IMSINH("4")`
  27.289917197127753 but `IMSINH("-4")` libm's -27.28991719712775 (above). Evidence: probes/t2629-imcosh-imsinh-y-sweep
  (800 IMCOSH and IMSINH of ±1.5+yi for 200 y in [0.103, 2.84], excel-graph 2026-10-26): libm sinh/cosh with libm sin/cos
  fits 1,562 of the 1,600 parts, this kernel all 1,600 (`IMSINH("-1.5+2.46454679i")`'s real part needs Excel's
  COS(2.46454679) = -0.7794268635101594, libm ...596); probes/t2778-imsinh-sign-sweep (300 real arguments at both signs,
  0.0105 <= |x| <= 18.2, excel-graph, T-2778): 300 of 300, where `(EXP(x) - EXP(-x))/2` fits 224 and libm 97. With
  these values the plain reciprocal of IMSECH also gives `IMSECH("1.5+711i")` -0.371302652691887i
  (probes/eng-im-overflow-edges A9, A17 and A58). Outside the probed band Excel's digits are libm's: `IMSINH("0.01")`
  0.0100001666675 (the form cancels to ...6674999), `IMSINH("0.00012+1.23456789i")` and `IMCOSH` of it
  (probes/t2535-imsinh-imcosh-kernel-sweep A55, A6, B6; COMPLEXs.xlsx P8, S8) and `IMCOSH("100+1.5i")`
  9.50749421835286E+41+.. (eng-im-overflow-edges-2 A60), where `1/EXP(-100)` is one ulp below EXP(100). So Excel switches
  forms below 0.0105 and above 18.2; neither cut is probed. `IMSIN("-4i")` is -27.2899171971278i, IMSINH(4)'s
  digits (T-2778's probe C2), though outside the band IMSIN's own sinh is the EXP pair (`IMSIN("1E-200+1E-200i")`
  "1E-200", `IMCOS("150-0.002i")`): since T-2796 IMSIN and IMCOS take this kernel at -y inside the band (sin z =
  -i·sinh(i·z)), which fixes C2 and keeps every one of the 1,023 IMSIN/IMCOS cells of the suites (excel-graph and desktop
  caches); a mixed argument inside the band is fitted only by those cells, not by a sweep. Ours: `inverse_exp_sinh_cosh` in functions/engineering/complex/trig.rs
  over 0.01 < |x| <= 20, libm outside. Status: VERIFIED (excel-graph) inside the band; the cuts are UNVERIFIED.
- **IMSQRT of a negative real is polar through COS and ATAN2 on the web (the engine keeps desktop's libm value).** `IMSQRT("-1")` (`IMSQRT(TEXT(-1,"0"))`) is
  "6.1257422745431E-17+i" = COS(ATAN2(-1,0)/2) with Excel's COS (libm cos(π/2) 6.12323399573677E-17), `IMSQRT(-2.5)`
  9.68564897336834E-17+1.58113883008419i, `IMSQRT(-0.0001)` 6.1257422745431E-19+0.01i (516488da78a6b08c A52,
  59a16b5ab566ee5c A6, 7dc52cfcbc8b4448 A254). Status: CONFLICTING (web vs desktop). Mac desktop 16.0300 caches libm's
  value for the same formula: ironcalc-calc/ENGINEERING/COMPLEXs.xlsx Sheet1!AL26 `IMREAL(IMSQRT(COMPLEX(-1,0)))` is
  6.12323399573677E-17 and AL12 (of -12) 2.12115047744981E-16, which the engine matches with libm `atan2`/`cos`/`sin`.
  T-2536 found that Excel's ATAN2 and SIN/COS give the four web cells but turn AL12 and AL26 into failures. The gap is a
  near-zero residue (about 2.5E-20 on a 6E-17 part), so under decision 0030 rule 4 and decision 0039 the engine keeps the
  desktop value and the 5 gen-m9 cells stay known failures. The desktop check is T-1075 item 16.
- **IM* kernels still to fit (T-2044, T-2537, T-2538).** New web cells: `IMSECH("3+4i")` (and `" 3+4i"`, `"3 + 4i"`,
  `"3+4j"`) "-0.065294027857947+0.0752249603027732i" (57 cells), `IMSECH("0.1")` 0.995020748953227, `IMCOT("-1.5+0.5i")`
  "-0.0557109884865415-0.463942860183828i", `IMCSC("-1.5+0.5i")` "-0.888092201609111-0.029103639571751i",
  `IMCSC("1000+1i")` "0.617941078424196-0.320079291513745i", IMLOG2 of 100, "420.31", 0.02 and -0.0001
  (6.64385618977473, 8.71530997122009, -5.64385618977473, -13.2877123795495+4.53236014182719i) and `IMLOG2("1000+1i")`
  "9.96578500600925+0.00144269455999091i"; `IMABS("0.25-3i")` 3.0103986446980735 (both `sqrt(x²+y²)` and `hypot` give
  3.010398644698074, so on the web too IMABS is neither), `IMABS("-724.227-562.884i")` 917.24759197558,
  `IMARGUMENT("1.5E+2-2E-3i")` -1.3333333332543209E-05 (= Excel's ATAN2; f64::atan2 ...4321E-05). Candidates the triage
  checked (UNVERIFIED until T-2044's sweep confirms them): IMLOG2's real part is ln|z| / 0.693147180559945 (ln 2 to 15
  digits: 5 of 5 real parts here and COMPLEXs.xlsx AH19; the exact ln 2 gives 1 of 5) and its imaginary part ATAN2 / LN(2)
  (2 of 2); IMCSC is conj(s)/|s|² of sin z built from SIN, COS and the exp-based sinh/cosh (both IMCSC inputs; a Smith
  reciprocal gives ...112). IMCOT("-1.5+0.5i") is not the double-angle form with Excel's kernels.
- **IM* log and power modulus.** `IMLN("1E+200")` is 460.517018598809, `IMLOG10("1E+200")` "200", `IMLOG2("1E+200")`
  664.385618977473 (ours `#NUM!` for all three since T-2389; the overflow is still wrong), while `IMLOG10("1E-200+1E-200i")` and `IMLOG10("1E+300+1E+300i")` are
  `#NUM!`: T-2117's rule (on its branch) of a `hypot` modulus that is `#NUM!` when both parts are nonzero and x²+y² overflows
  or is subnormal. `IMPOWER("1E+300+1E+300i",-0.5)` is `#NUM!` too although the result would be tiny (ours
  7.77E-151-3.22E-151i; 1068c44afcd56d53 A136). T-2539.
- **IMCSCH of 0 is `#NUM!`**, as IMCSC of 0: `IMCSCH("0")`, a 0 cell, an empty cell (`IMCSCH(Data!Z1)`), and a denominator
  that underflows, `IMCSCH("1E-200+1E-200i")` (ours `NaNNaNi`, `inf-infi`). This settles the two IMCSCH UNVERIFIED notes in
  "IM* results that overflow (T-1827)". T-2540.
- **IMLN and IMLOG10 of 0 are `#NUM!`** ("0", "0i", an empty or 0 cell; ours was "-inf" until T-2389, which also made the four `1E-200+1E-200i` / `1E+300+1E+300i` IMLOG10 cells match).
- **Tiny results.** `IMSEC`/`IMCSC("-724.227-562.884i")` keep their 1E-245 parts (as the T-2043 sweep found), while
  `IMEXP("-724.227-562.884i")` is "0": e^-724.227 is subnormal and both parts flush (ours "0+0i" before T-2335). RustSheet flushes every subnormal IM* result part to 0 in the complex number-to-text: VERIFIED for IMEXP and the IMSEC/IMSECH overflow edges, inferred (UNVERIFIED) for the other IM* functions from numbers.md "Subnormal results become 0".
- **How an inumber is read.** Text that VALUE reads is a real number: `IMPOWER("$4.00",1)` "4", `IMSEC("$3.14")`
  -1.00000126827407, `IMCONJUGATE("2:24:00")` "0.1", `IMSIN("0:00")` "0" (ours `#NUM!`). A leading space is read
  (`IMSECH(" 3+4i")`), but a leading tab (`IMCOT(CHAR(9)&"123456789.123")`) or ideographic space U+3000 is `#NUM!` (ours
  reads them). Text of 604 characters is `#VALUE!` (`IMSECH("TRUE"&REPT("AB",300))`; ours `#NUM!`), as DECIMAL's 255
  limit. A number argument keeps its full precision: `IMPOWER(1.6564497455696935E-10,3)` is 4.54500948137352E-30, which
  no 15-digit reading of the base gives (ours ...349, the 15-digit read). Cells: 65ff9a692665eba9 A28, b5aef3645cec1b95
  A101, 12e4aa13a20cc1f8 A391, 3cf48801617a72ad A91, 12e4aa13a20cc1f8 A224, 8fa8b572ebc1f001 A377, c2d778463fe008ed
  A232, 756a9f7bc3a06bcc A345. Implemented in T-2541 (`complex_of`); probes/eng-inumber-reading (excel-graph, 2026-10-25) adds:
  `IMABS("$1,234.50")` 1234.5, `IMABS("50%")` 0.5, `IMABS("1E3")` 1000, trailing space read, text of 255 characters read
  and 256 `#VALUE!` (`REPT("0",254)&"3"` 3, `REPT("0",255)&"3"` and `REPT("0",255)&"3+4i"` `#VALUE!`), `IMREAL("1E-310")`
  `#NUM!`, and a number keeps full precision into IMREAL(1/3) 0.3333333333333333 while IMPOWER, IMSUM and IMSUB of it
  print 15 digits.
  Inner minus (probes/eng-inumber-date-boundary, excel-graph, 2026-10-25): text with a minus sign that is neither its
  first character (past leading spaces) nor an exponent's is not read even where VALUE reads it: `IMABS` of
  "2024-01-01", "1-2-2020", "1-Jan-2020", "2020-1-2", "1-1-1", "$-5", "2-" and "$1,000-" is `#NUM!`. Slash and
  month-name dates and clock text are read as VALUE reads them ("1/2/2020" 43832, "2024/01/01" 45292, "1/2/2020 3:00"
  43832.125, "Jan 1, 2020" 43831, "1/2/20" 43832, "12:00 PM" 0.5), and so are " -5", "- 5", "(5)", "1E-3%" 1E-05,
  "1,000E-2" 10 and "3 1/2" 3.5. Ours matches all of these since T-2541 (`has_inner_minus`) except "3 1/2", which
  VALUE does not read yet (text.md, T-2646).
  A currency sign, `$` or `€`, that starts the text or a part (after a `+` or `-` that is not an exponent's, spaces
  allowed on both sides) and stands before a digit or a `.` (past spaces) is dropped (probes/eng-inumber-reading A5,
  A20:A26, excel-graph, 2026-10-28; probes/eng-inumber-currency-shapes A1:A12, A27:A40, excel-graph, 2026-10-27 and
  2026-10-28): `IMABS` of "$1+2i", "-$1+2i", "+$1+2i", "- $1+2i", "$1-2i", "$1+2j", "$ 1+2i", "1+$2i", "1-$2i",
  "1+ $2i", "1+$ 2i", "$1+$2i" and "€1+2i" is 2.23606797749979, "$3i" 3, "$.5" 0.5 and "$.5+2i" 2.06155281280883.
  Not read (`#NUM!`): "$i" and "-$i" (no number after the sign), "$$1", "$-5", "$ -5", a sign inside a number ("1$2",
  "1.$5", "1E-$3", "$1e+$2") and the other currency signs "£1+2i", "¥1+2i". Implemented in T-2641
  (`strip_currency_symbols`).
  End whitespace (probes/eng-inumber-currency-shapes A13:A26): a tab, NBSP, LF, CR or U+3000 at either end, past
  leading spaces, is `#NUM!` for a complex number and a plain real alike, where a space is read ("$1+2i " 2.23606797749979):
  `IMABS` of NBSP&"3+4i", "3+4i"&tab, "3+4i"&LF, "3+4i"&CR, LF&"3+4i", "5"&tab, "5"&NBSP, CR&"5", "5"&U+3000,
  " 5"&tab and " "&tab&"5" are `#NUM!`. Implemented in T-2641 (`complex_of`).
  Unprobed (ours in brackets): a "+" inside non-complex text; "(-5)" and "June-2020" [`#NUM!`, the inner-minus rule];
  "1E$3" and "12$3i" [`#NUM!`]; leading spaces before NBSP, LF, CR or U+3000 (only tab is probed) [`#NUM!`]; one of
  the five followed by trailing spaces ("5"&tab&" ") [read]; these five inside the text; a leading tab on text over
  255 characters [`#VALUE!`]. T-3059 holds them.
- **Large parts in plain notation.** An IM* part is written in plain notation up to a decimal exponent of 20, zero-padded
  past 15 significant digits (`IMPOWER("1-2i",59)` is "-331147245179666000000-252614032879411000000i";
  `IMSUM("123456789012345678901","0")` is "123456789012345000000"), and in E notation from exponent 21
  (`1.23456789012345E+21`, `1.5E+21`) (probes/eng-im-large-part, exponents 15 to 25, real and imaginary parts, IMSUM and
  IMPRODUCT; matched since T-2542). Plain notation is not capped at 20 characters here, unlike coercion to text.
- **Base conversions.** BIN2DEC, BIN2HEX and BIN2OCT of non-binary text are `#NUM!` ("ABC", "a", " x ", "TRUE", "a*";
  matched since T-2544; OCT2*/HEX2* of such text already give `#NUM!`). `places` is checked before the number: an invalid
  places (0, including an empty cell and fractions that truncate to 0 such as 0.5 or 0.01005, a negative one, 11, 42) is
  `#NUM!` even when the number is an error or unreadable text (`DEC2HEX(<#VALUE!>,11)`, `HEX2BIN(<#N/A>,0)`,
  `OCT2HEX(<#DIV/0!>,-0.5)`, `DEC2OCT(" x ",-2.5)`), and over a boolean number (`DEC2BIN(TRUE,-1)`, `HEX2BIN(TRUE,0)`;
  `DEC2BIN(TRUE,4)` stays `#VALUE!`; probes/t2545-base-places-boolean). Ours matches since T-2545 (`number_and_places`). A boolean is `#VALUE!` in every
  OCT2*/HEX2* conversion, also OCT2HEX, as a FALSE or TRUE cell, an `IF(..)` FALSE or an `INDEX(..)` TRUE (ours `#NUM!`),
  T-2420.
- **BESSEL.** `BESSELJ(1E+15,0)`, `BESSELJ(1E+15,5)` and `BESSELY(214285740740749.06,2)` are `#NUM!`,
  while x = 123456789.123 (below 2^27) computes: `BESSELJ(..,1)` 4.9869177786549744E-05 and `BESSELY(..,3)`
  5.1668995876750464E-05 (matched since T-2547: the phase cut and Excel's sine, "BESSELJ / BESSELY" above);
  `BESSELJ(1E+5,100000)` is 0.009636944011336267 (matched since T-2548: twice-rounded recurrence steps, "BESSELJ / BESSELY" above). `BESSELI(1E+15,n)` is `#NUM!` (ours 0), and BESSELK at 713 and 714 is 0 for orders 0, 1
  and 4, T-2421. Past the order cap Excel gives 0: `BESSELI(100,100001)`, `BESSELI(8,1E+6)`, `BESSELJ(0,1E+6)`,
  `BESSELK(25019,100001)` ("BESSEL order cap"; ours matches since T-2546). `BESSELK(700,3)` is 4.69979514014056E-306 (matched
  since T-2549: the subnormal recurrence term flush, "BESSELK near the bottom of the double range").
- **BIT shifts.** `BITRSHIFT(n,-53)` is 0 for n = 41, 434, 633, 699, 944, `BITLSHIFT(218,53)` 0, and a number 0 shifts
  to 0 by 54 either way (`BITRSHIFT(0,54)`, `BITLSHIFT(<empty>,-54)`) although `BITLSHIFT(1,-54)` is `#NUM!` (T-2249). T-2421.
- **Inner functions.** 70 cells differ only because an inner call does (QUARTILE/PERCENTILE of a lone text or boolean
  cell, LINEST/LOGEST of one point, XIRR/XNPV type checks, F.TEST of scalars, Z.TEST, PROB, T.DIST of 1E+307, CHISQ.INV of
  1E-300, a boolean read by F.INV.RT, FDIST, BETA.DIST, HYPGEOMDIST and STANDARDIZE, TBILLYIELD, ODDLPRICE, ODDFYIELD,
  FREQUENCY, paired functions over an empty cell, FORECAST.ETS); they are owned by those functions' tasks (T-2258's
  Outcome lists them). Also: `DB(59341,-1,28,6,12)` is a NaN number inside a formula in ours (ISNUMBER TRUE), so
  `DCOUNTA(Db!A1:D9,DB(..),Db!F1:F2)` is `#VALUE!` where Excel gives DB's `#NUM!` (T-2551).
Evidence: gen-m9 cases of seeds 2258, 22580 and 22581 (excel-graph, 2026-10-22/23, T-2258); every value above is an
Excel cell of eval/suites/gen-m9 with its case id in the owning task. The engine-side formulas (SIN, (EXP(4)-EXP(-4))/2,
COS(ATAN2(-1,0)/2), ln|z|/0.693147180559945) were evaluated in RustSheet's own worksheet functions, which match Excel.
Status: VERIFIED (excel-graph) for the observed values. The mechanisms (IM trig of a real as SIN/COS/TAN, exp-based sinh/cosh, IMSQRT through COS and ATAN2) are fitted against RustSheet's own kernels and reproduce every cited cell; T-2534, T-2535 and T-2536 re-confirm them on the cells. The IMLOG2 and IMCSC formulations are UNVERIFIED candidates.
Tasks: T-2258, T-2532 to T-2551, T-2420, T-2421, T-2335, T-2389, T-2044, T-2117, T-1966

## Leading plus in IM*, 11-digit OCT/HEX text and booleans in OCT2*/HEX2* (T-2420)
Rule: IM* read one leading `+` like a leading `-`: `IMABS("+i")` 1, `IMABS("+3+4i")` 5, `IMABS("+4i")` 4, `IMSUB("+i","1")` "-1+i", `IMDIV("2i","+i")` "2"; a second sign (`"++i"`, `"+-i"`, `"-+i"`, `"1++i"`) is `#NUM!` (probes/eng-leading-plus-oct-bool A1:A26).
Rule: a boolean is `#VALUE!` in every OCT2* and HEX2* argument, typed, from a cell, from `IF`/`INDEX`, in an array literal or a MAP result, with or without places; the same as BIN2DEC (A35:A62). A boolean inside arithmetic is a number (`OCT2DEC(7+B1:B2)` is `#NUM!`, A62).
Rule: OCT2DEC, OCT2BIN and OCT2HEX of 11 digits, number or text, are `#NUM!` (`OCT2DEC(7777777777)` is -1; A27:A33).
Rule: ODDFPRICE, ODDFYIELD, ODDLPRICE and ODDLYIELD read the date arguments first and a negative date (-0.5 too) is `#NUM!` at once, ahead of an error in a later argument (`ODDFPRICE(39000,-1,..,NA(),..)` is `#NUM!`); a date of 0 is not, and an out-of-order date pair still lets the later error win (`#N/A`) (probes/fin-odd-date-before-error-arg). The full order, with a date past 9999-12-31 also `#NUM!` as it is read, is in financial.md "ODDFPRICE, ODDFYIELD, ODDLPRICE and ODDLYIELD check order" (T-2483).

## BESSELK underflow, BESSELI overflow and BIT shifts of 49 to 53 (T-2421)
Rule: BESSELK below the smallest normal double is 0: `BESSELK(705,0)` is 3.135297088468052E-308 but `BESSELK(705.5,n)` and every x past it (708, 713, 714, 1000, 1E5) are 0 for n = 0, 1 and 4 (probes/eng-besselk-underflow-bit-shift).
Rule: BESSELI is `#NUM!` for every order from x = 710 on, however large (`BESSELI(1E+15,2)`, `(-1000000000000002,3)`); `BESSELI(705,0)` is 2.26205E+304 and 700 is finite.
Rule: BITLSHIFT/BITRSHIFT truncate the shift amount first and limit it after (`BITLSHIFT(1,53.5)` and `(1,-53.5)` are 0; 54 is `#NUM!`, 54.x not probed). A number 0 shifts to 0 by any amount (`BITLSHIFT(0,1E+10)`). For a nonzero number an effective left shift of 48 is `#NUM!` (`BITLSHIFT(1,48)`, `BITRSHIFT(1,-48)`), of 49 to 53 places is 0 (not the overflow `#NUM!`), 54 and more is `#NUM!`; a right shift of 48 to 53 places is 0, 54 is `#NUM!`.
Not fixed here: `BESSELI(2.5,100001)` and `(2.5,1E+6)` are 0 in Excel and `#NUM!` here (BESSEL order cap DEVIATION, pinned in probes/deviations); `BESSELK(2.5,171)` onward overflows to `#NUM!` (170 is 7.09E+287, 200 and 1000 are `#NUM!`).

## BESSELK near the bottom of the double range, BESSELI overflow point (T-2550)
Evidence: probes-numeric/m9-besselk-underflow (147 cells, excel-graph): BESSELK(x,n) for n = 0 to 5 and BESSELI(x,0) at x = 500, 600, 650, 680, 690, 695, 700, 702, 705, 707, 708, 709, 709.5, 709.78, 710 to 713 in steps of 0.5 from 710 (21 x values x 7 cells = 147). Ours matches 134 of 147 cells.
Rule: ours is bit-exact for BESSELK at every n probed up to x = 695, and at x = 700, 702 and 705 for n = 0 and 1; BESSELK is 0 for n = 0 to 5 from x = 707 on (T-2421 puts the flush between 705 and 705.5), as ours.
Rule: in the upward recurrence `K(n+1) = K(n-1) + 2n/x * K(n)`, Excel sets the product `2n/x * K(n)` to 0 when it is subnormal (|product| < 2.2250738585072014E-308; inferred: the probe only brackets the threshold in (1.34E-308, 2.67E-308], from the x = 700 products of n = 1 (flushed) and n = 2 (kept)), as the other subnormal flushes here do (BESSELK below the smallest normal, T-2421). Replaying the recurrence from the oracle's K0 and K1 with that flush reproduces all 12 differing cells bit-exactly; without it, it reproduces our 12 values bit-exactly. At x = 700 only the n = 1 product (about 1.3E-308) is subnormal, so only K2 = K0 (4.669776528722504E-306) while K3 (4.69979514014056E-306), K4 (4.710060487066566E-306) and K5 (4.753624402849892E-306) still get their terms; at 702 and 705 every product is subnormal, so K2 = K4 = K0 and K3 = K5 = K1 (`BESSELK(702,4)` 6.310849433265845E-307, `(702,5)` 6.315342484400538E-307, `(705,4)` 3.135297088468052E-308). Ours adds the subnormal term and misses these 12 cells (n = 2 to 5 at x = 700, 702 and 705) by 1.6E-5 (`BESSELK(700,3)`) to 1.7% relative (`BESSELK(702,5)`, ours 6.4241464536256334E-307).
Superseded by "BESSELI overflow point (T-2613)" below: the cut is at 709.7. Rule: BESSELI(x,0) is finite up to x = 709.5 (2.02976276636543E+306) and `#NUM!` at 709.78 and from there to 713; ours is finite at 709.78 (2.685109662266049E+306), so the overflow cut lies in (709.5, 709.78], below the true overflow of I0 near 713.98 (T-2421 has 710 on).
Ours flushes that term since T-2549 and matches all 12 cells (146 of the probe's 147).

## BESSELI overflow point (T-2613)
Evidence: probes/besseli-overflow-cut-sweep (x = 709.50 to 709.78 step 0.01, orders 0 and 1), -fine (709.700 to 709.710 step 0.001, orders 0, 1 and 5), -finest (709.7001 to 709.7009, 709.70001, 709.70005, 709.70009, order 0), excel-graph.
Rule: BESSELI(x,n) is finite up to x = 709.7 (`BESSELI(709.7,0)` is 2.4788083687166675E+306, `(709.7,1)` 2.4770621409647947E+306, `(709.7,5)` 1.9147648105012915E+306) and `#NUM!` at every probed x above it, to 709.78 and on, at orders 0, 1 and 5. The cut is in (709.7, 709.70001], far below the true overflow of I0 near 713.98 and of `exp` at 709.78, so it is a constant in Excel's code, taken as 709.7. Negative x is unprobed (ours takes |x|).
Status: VERIFIED (excel-graph) for positive x
Implemented in: functions/engineering/transcendental/bessel_i.rs (`BESSELI_OVERFLOW_X`)
Tasks: T-2613

## BESSELJ and BIN2* cells of the T-2436 sub-expression probe
Rule (excel-graph, 2026-10-24, probes/t2436): `BESSELJ(1E+15,100)` is `#NUM!` (caa1eed48b1883fe A399; matched since T-2547, the phase cut), and `BIN2DEC("Hello World")` and `BIN2HEX("#N/A",1)` (text cells) are `#NUM!` (matched since T-2544); an `IMSUM` of
an empty array is `#VALUE!` (arrays.md "An empty array as a function argument").
Status: VERIFIED (excel-graph), one cell each
Tasks: T-2547, T-2544, T-2580

## IMPOWER of a base whose squared modulus overflows (T-2539)
Evidence: probes/eng-impower-overflow (12 cells, excel-graph, 2026-10-24).
Rule: IMPOWER is `#NUM!` when both parts are nonzero and `x²+y²` overflows, whatever the power and even when the result is tiny: `IMPOWER("1E+300+1E+300i",-0.5)`, `(…,-1)`, `(…,0.5)`, `(…,1)`, `(…,0)`, `IMPOWER("1E+154+1E+154i",-0.5)` and `(…,2)`, `IMPOWER("1E+155+1E+155i",1)`. A pure real base is not refused: `IMPOWER("1E+300",-0.5)` is `1E-150`, `IMPOWER("1E+200",-0.5)` is `1E-100`. An underflowing square gives "0" (`IMPOWER("1E-200+1E-200i",2)`, `("1E-170+1E-170i",2)`). A pure imaginary base is probed in T-2626 (below). `IMLN("1E+200")`, `IMLOG10("1E+200")` and `IMLOG2("1E+200")` already match after T-2117.
Status: VERIFIED (excel-graph)
Tasks: T-2539

## IMPOWER of a pure imaginary or lopsided base (T-2626)
Evidence: probes/eng-impower-overflow-shapes (14 cells, excel-graph, 2026-10-28).
Rule: a pure imaginary base is not subject to the both-parts-nonzero `x²+y²` overflow refusal (the modulus is `|y|`): `IMPOWER("1E+300i",-0.5)` is `7.07106781186548E-151-7.07106781186547E-151i`, `("1E+300i",-1)` is `-1E-300i`, `("-1E+300i",-0.5)` is `7.07106781186548E-151+7.07106781186547E-151i`, `("1E+200i",-0.5)` is `7.07106781186548E-101-7.07106781186548E-101i`; the result is `#NUM!` only when the radius overflows (`("1E+300i",2)`). `IMPOWER("1E+300i",1)` is `6.1257422745431E+283+1E+300i` (the x87-reduced cosine of the angle, not 0). A lopsided base with both parts nonzero is `#NUM!` as soon as `x²+y²` overflows, whatever the power and however small `y` is: `IMPOWER("1E+200+1E-300i",-0.5)`, `(…,-1)`, `(…,2)`, `(…,1)`, `IMPOWER("1E+300+1E-300i",-0.5)`, `IMPOWER("1E-300+1E+200i",-0.5)`, `IMPOWER("1E+200+1E+10i",-0.5)` and `(…,2)`. The existing guard in `fn_impower` already matches all 14 cells.
Status: VERIFIED (excel-graph)
Tasks: T-2626

## IMCOSH and IMSINH past |x| = 709 work in log space (T-2231)
Evidence: probes/eng-im-overflow-edges A6-A21, probes/eng-im-overflow-edges-2 A1-A24, A58-A114 (excel-graph, 2026-10-24).
Rule: for |x| > 709 and a nonzero imaginary part, IMCOSH and IMSINH compute `exp(|x| + ln(cos y / 2))` and
`exp(|x| + ln(sin y / 2))`, then give sinh's factor the sign of x (IMCOSH `(a, sign(x)·b)`, IMSINH `(sign(x)·a, b)`).
- Consequences: `#NUM!` whenever cos y or sin y is negative, even far below the overflow: `IMCOSH("709.05+2i")`,
  `"709.0000000001+2i"`, `"709.3-0.5i"`, `"-709.3-0.5i"`, `"-710.4-2i"`, `"709.5+3.14159265358979i"`, `IMSINH("709.4+2i")`,
  while `IMCOSH("709+2i")` is `-1.71003213329077E+307+3.73648837869846E+307i` (product form) and `"-710.4+1.5i"` answers
  `1.17873946358278E+307-1.6621900184285E+308i`. A result past DBL_MAX is `#NUM!` (`IMCOSH("711+1.5i")`); `"710.48+1i"` answers.
- The digits differ from `cosh x·cos y`: `IMCOSH("710.4+1.5i")` is `1.17873946358278E+307+1.6621900184285E+308i`
  (exact ...281), `IMSINH("709.5+1E-200i")` has real part `6.77493159657354E+307` against `IMSINH("709.5")`'s
  `6.77493159657316E+307`; the candidates `exp(x - ln 2 + ln t)`, `exp(x + ln t)/2` and `exp(x/2)·exp(x/2)·t/2` all miss.
- A zero imaginary part keeps the real path (`cosh x`, `sinh x`; `#NUM!` once `exp |x|` overflows): `IMCOSH("709.5")`,
  `"709.5+0i"`, `IMCOSH(709.5)` are `6.77493159657316E+307`, `IMSINH("-709.5")` its negative.
- The cut lies in (709, 709.0000000001]; implemented as `|x| > 709`. UNVERIFIED: a large y with |x| > 709 (ours takes
  libm sin and cos; SIN's own reduction could matter), and whether IMSIN/IMCOS and IMSECH/IMCSCH share the
  negative-factor `#NUM!` (T-2631).
- IMSIN and IMCOS with 709.78 < |y| < 710.4 are `#NUM!` as for larger |y| (`IMSIN("1.5+709.9i")`, `"0+709.9i"`,
  `IMCOS("-2-709.9i")`; A25-A34), and `IMSIN("1.5+709.7i")` answers: `exp |y|` overflowing is the cut, as implemented.
- IMPRODUCT of a range holding a blank, and of a blank alone, is "0" (`IMPRODUCT(C1:C3)` with C2 blank, `IMPRODUCT(C2:C3)`,
  `IMPRODUCT(C2)`; A55-A57).
Status: VERIFIED (excel-graph)
Implemented in: `hyperbolic_log_form` and `HYPERBOLIC_LOG_CUT` in functions/engineering/complex/trig.rs
Tasks: T-2231
