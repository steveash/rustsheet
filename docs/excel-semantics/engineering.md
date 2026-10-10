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
Rule: RustSheet returns `#NUM!` for BESSELJ and BESSELY orders above 100,000 to bound per-cell work on
untrusted input (CLAUDE.md directive 5). Excel gives `BESSELJ(10,1000000)` = `0` (probes-numeric/m9-bessel, T-1004), so the engine differs above 100,000 for J.
Evidence: probes-numeric/m9-bessel (T-1004) shows Excel's behavior for huge orders; the cap itself is a resource limit, not an Excel observation.
Status: DEVIATION (decision 0014)
Implemented in: functions/engineering/transcendental/bessel_excel.rs (`MAX_ORDER`)

## DECIMAL of unreadable text
Rule: DECIMAL reads its first argument as text (a number or TRUE goes through its text form, so `DECIMAL(0.5,13)`, `DECIMAL(1E+20,10)` and `DECIMAL(TRUE,10)` read "0.5", "1E+20", "TRUE"). Leading spaces and tabs are skipped; empty or all-blank text is 0. Any other unreadable character is `#NUM!` (not `#VALUE!`): a sign (`"-1"`, `"+1"`), a point, trailing space, a non-breaking space, a digit not below the radix. Text over 255 characters is `#VALUE!` (a non-numeric radix too). Radix 16 only also skips leading zeros and one `x`/`X` (`"0x1F"`, `"x1F"`, `"00x1F"` are 31; `"0x0x1"`, `"1x1F"`, `"0x 1F"` are `#NUM!`; in radix 10 or 15 `"0x1F"` is `#NUM!`, in radix 36 the x is a digit). The sum is a float with no 2^53 limit: 255 ones in radix 2 is 5.78960446186581E+76.
Evidence: probes/decimal-unreadable (68 cells, excel-graph, 2026-10-10); gen-m5/98ed0b6354e3394a Calc!A9.
Ours: matches.
Status: VERIFIED.
Not probed: whitespace other than space and tab (newline, form feed) at the start.
Tasks: T-1198
