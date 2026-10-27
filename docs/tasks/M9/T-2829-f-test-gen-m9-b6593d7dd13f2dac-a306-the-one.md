---
id: T-2829
title: "F.TEST gen-m9/b6593d7dd13f2dac A306: the one cell bratio leaves visibly off (T-2681)"
milestone: M9
lane: research
status: in-progress
priority: P2
size: M
tier: opus
depends_on: [T-2681]
campaign: visible
---

## Goal
`F.TEST(Data!B6:E12,Data!A1:D10)` (gen-m9/b6593d7dd13f2dac Calc!A306) has Excel's value
1.23966888990022E-69 to 15 significant digits, without losing a bit-exact F.TEST cell.

## Context
After T-2681 F.TEST's tail for both shapes above 1 is bratio's `bpser` with Excel's pieces
(fisher.rs `f_test_p`, f_test_tail.rs); this cell (a = 6, b = 11.5, x = 6.29E-13) is the only
F.TEST cell on file still visibly off, 25 ulp (docs/research/t2681/README.md "The 8 cells").
Its data hold 123456789.123 next to values near 1. Excel's p is 19 ulp from the exact tail at the
two-pass f; the exact variances move the model's p by 3 ulp only; the z that would give Excel's
p is 48 units (2^-53) from bratio's, about eight ulps of x. Ideas: how Excel forms x from the two
variances when one is 8E11 times the other (x = df_s/(df_b f + df_s) against df_s v_s/(df_b v_b +
df_s v_s), and the variance sums in x87 registers); probe the same shapes with the data scaled.
Use `docs/research/t2681/score_bratio.py` to score.

## Findings so far (2026-10-27, first opus run; resumed 13:55 UTC by a second run; scripts not committed, numbers reproducible as below)
Inputs: a = 6, b = 11.5, two-pass x = 6.291397944737322e-13, Excel p = 1.23966888990022E-69 (the oracle's
shortest repr has 15 digits; A289 below has 17). Ours (`f_test_bratio_p`) 1.2396688899002135E-69: 25 ulp low, and
it displays ...021, so it is visibly off.
- **Not x.** Exact tail at each x against Excel, by mpmath `betainc`: two-pass double x +19 ulp (Excel is lower
  than the exact tail), exact x +27. The other forms tried all give x at or above the two-pass x (+19 to +42):
  one-pass, `n·Σx² - (Σx)²`, Welford, x87-extended accumulators (numpy longdouble), row- or column-major order,
  `1/(1 + db f/ds)`, `ds v1/(db v2 + ds v1)`, f in extended. Excel would need x about 4 ulp below the two-pass x.
  The port is flat in x here: x ± 1 ulp gives the same p, because fl(a·LN x) is stored on a 2.8E-14 grid.
- **Not ln x formed from pieces**: `fl(ln 12) - fl(ln total)`, `-ln(1 + 23f/12)` and the rest land on the same
  stored fl(a·LN x) (-25 ulp), or at +5 to +43 when a·ln x is left unstored.
- **z**: with ln B exact, Excel's p needs z = a·ln x - ln B about 4.0E-15 below the exact z. fl64(a·LN x) is 9.18E-15
  below (that is our -25 ulp), and an unstored a·ln x is about exact (+19).
- **ln B**: TOMS 708 `betaln` (reduce a0 to (1, 2], then `ln w + GAMMALN(a0) + algdiv(a0, b0)`) equals the
  accurate ln B on A306 and on A289 (both 0 ulp apart in p). Excel's GAMMALN sum held in a register,
  `-GL(a) + (-GL(b) + GL(a+b))` (BETA.DIST's bpser form, ln B 13 ulp high in p), puts A306 on Excel's 15 digits.
  But used for every pair of shapes both >= 1 with one above 8, it turns gen-m9/a790a4d4a6a82756 Calc!A289
  (`F.TEST({1,1,2,2,2,3},Data!B1:E11)`, a = 2.5, b = 11, x = 2.9621212121212663E-30, Excel
  4.2838236027000643E-72, bit-exact today) into ...0517, visibly off. So that form is out unless something
  separates the two cells (b half-integer against integer? a0 >= 4?).
- **Probe probes/t2829-ftest-a306-shapes (excel-graph, 2026-10-27)**, ours from `rustsheet-eval show`:
  - A1 = A306's F.TEST with inline arrays: Excel 1.23966888990022E-69, the same double as the gen-m9 cell. A2,
    `2*F.DIST.RT(VAR.S(..)/VAR.S(..),23,12)`, gives the same double, so F.TEST is 2·F.DIST.RT at Excel's own f.
    Ours is 1.239668889900213E-69 on that path (a different kernel from F.TEST's).
  - A3 and A4 are VAR.S of the two samples. Excel's are 765.7947626390592 and 635065683430145.2; ours shows A4
    as ...145.3, 1 ulp apart (it still scores Pass).
  - A7-A12 hold the large sample times 1 ± 1E-9..1E-8, which moves x by up to about 1E5 ulp. Ours (accurate ln B)
    is low against Excel by a constant 5.3E-15 relative on every cell (A1 -65, A7 -67, A8 -68, A9 -67, A10 -67,
    A11 -66, A12 -65, in units of 1E-85). Against the GAMMALN-register ln B it is high by a constant 2.7E-15.
    The residual of a stored fl(a·LN x) changes with x, so a constant offset means Excel stores a·LN x as the
    port does. What remains is a constant of the shapes: ln B, s or EXP.
  - A5 is the b = 11 twin (the large sample less its last value): Excel 1.0045406401767847E-69, ours ...7833,
    1.4E-15 relative low. A6 is its F.DIST.RT.
  - Excel's ln B would have to be accurate - 5.3E-15 at (6, 11.5), accurate - 1.4E-15 at (6, 11), and the
    accurate value at (2.5, 11) (A289). No form tried gives all three: the GAMMALN register sum
    (-8.9E-15 / -1.8E-15 / +2.7E-15), `GL(a) + algdiv(a, b)` (+1.8E-15 / -1.8E-15 / -0.9E-15), and TOMS's
    reduction (0 / 0 / 0). Since -1.4E-15 is below one ulp of a stored ln B, look at the series s and EXP of the
    extended z too.
- **Probe probes/t2829-ftest-tail-over-b (excel-graph, 2026-10-27)**: `2*F.DIST.RT(1E+12,df1,df2)`, with Excel's
  values compared to `f_test_bratio_p(df2/total, df1·f/total, df2/2, df1/2)` (Excel - ours, relative; ulps):
  - a = 2.5: b = 8.5, 9, 10, 11, 12 and 13 are bit-exact; b = 9.5, 10.5, 11.5 and 15 are 3-4 ulp low (about
    -6E-16 to -8E-16); b = 19.5 is +2 ulp. Integer b is exact here, and half-integer b is not.
  - a = 6: b = 8.5 +11, 9 +3, 9.5 -22, 10 -10, 10.5 +1, 11 +9, 11.5 **-143 (-2.3E-14)**, 12 -7, 13 +9, 15 +5,
    19.5 +7. a = 6, b = 11.5 stands out here too, on another x. Check what is special at that pair (TOMS's
    reduction, the L140 or bgrat dispatch at a0 = 6 with b0 = 11.5, or half_integer_beta's product).
- Next ideas: (1) A306's data hold 123456789.123: probe F.TEST on the same shapes with x placed so that
  fl(a·LN x) has a different residual, to tell whether Excel stores a·LN x here at all; (2) probe a = 6 against
  b = 11 and 11.5 with modest data to test a half-integer-b ln B form; (3) check whether Excel orients this point
  through w1 (bratio's 1 - w with the swap) for a0 >= 4.

## Acceptance criteria
- [ ] A306 within half a unit of the 15th significant digit of Excel's value, or the cause
      recorded in docs/excel-semantics/statistics.md with the probe that shows it.
- [ ] No new visibly-off F.TEST cell (`uv run rustsheet-fitlab f-test`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The cells 1 to 3 ulp off (T-2830).
