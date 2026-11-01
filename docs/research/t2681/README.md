# T-2681: F.TEST's tail is TOMS 708 bratio with Excel's pieces for both shapes above 1

Question (T-2681, follow-up of T-2571): what Excel does for the 8 F.TEST cells the remainder model
`exp(a ln x) exp(b ln y) R` left visibly off. Finding: for both shapes above 1 (df 3 or more on
each side), F.TEST's tail is T-2464 r2-bratio's integrated model of Excel's incomplete beta
(`docs/research/t2464/r2-bratio/bratio_excel.py`, `BEST`): bratio's `bpser` (w3's x87 series) and
its L140 branch (`bup` with Excel's beta density as prefactor, plus `bpser`). For a shape of 1/2
the remainder model stays better.

## Scores (291 F.TEST cells of the `f-test` fit, bit-exact / within 4 ulp / visibly off)

`score_bratio.py` on the engine of `main` before T-2681 (`score_bratio.out`):

| bratio branch | cells | bratio `BEST` | engine before |
|---|---|---|---|
| L140 bup+bpser | 52 | 33 / 52 / 0 | 14 / 45 / 5 |
| bpser (a, b > 1) | 61 | 35 / 58 / 1 | 17 / 51 / 2 |
| w_bpser (a shape of 1/2) | 116 | 28 / 88 / 23 | 75 / 114 / 0 |
| w1_bpser, swapped | 8 | 7 / 8 / 0 | 2 / 6 / 0 |
| closed forms, bup+bgrat | 54 | 48 / 53 / 1 | 52 / 54 / 0 |

The engine (fisher.rs `f_test_p`) takes bratio where `f_test_bratio_p` reaches: both shapes above
1, a + b <= 200, bratio's smaller-side shape below 40, and in L140 x0 <= 0.7. One piece differs
from `BEST`: bup's density prefactor is BETA.DIST's density for a shape of 1/2 or 1 as the engine
already has it (`beta_density_small_shape`, T-2513's model M2, which replaced w1's M1 there), not
M1. With M1 the hybrid (`score_bratio.py`, which also leaves the cells that need TOMS `gsumln` to
the model) is 192 / 284 / 1; the engine with M2 needs no `gsumln` (M2 takes -LN(b) at a shape of 1)
and scores 205 / 284 / 1 against 160 / 270 / 7 before (`uv run rustsheet-fitlab f-test`, engine
row). On the 113 cells with both shapes above 1: 31 / 96 / 7 before, 76 / 110 / 1 after.

Ablations over the 113 cells with both shapes above 1 (`Cfg` switches one at a time from `BEST`):

| change | bit-exact / <=4 ulp / visibly off |
|---|---|
| `BEST` | 68 / 110 / 1 |
| plain TOMS bpser (`bpser="toms"`) | 38 / 76 / 24 |
| w3's EXP argument stored to a double (`w3_z="z53"`) | 47 / 80 / 22 |
| TOMS brcmp1 instead of Excel's density times x y (`brcmp1="toms"`) | 51 / 100 / 8 |
| the density correctly rounded instead of w1's M1 | 64 / 107 / 2 |
| M2 (the engine's density) instead of M1, in the engine | 76 / 110 / 1 |
| TOMS betaln for w3's ln B above 8 (`w3_big="toms"`) | 69 / 111 / 1 |
| `bup_arith="double"`, `bup_break=False`, `w3_lg="gl"`, `w3_order="min"`, `elem="libm"` | 67-68 / 110 / 1 |

So what decides Excel's value is w3's series with the unstored EXP argument and bup's density
prefactor (Excel's density for a shape of 1/2 or 1); the other switches move at most one cell.

## The 8 cells

| cell | shapes | branch | before | after |
|---|---|---|---|---|
| gen-m9/58e3253bbd3a7a82 A141 | 7.5, 2.5 | L140 | 12 ulp | bit-exact |
| gen-m9/8f2ac25945af87da A119 | 6, 4.5 | L140 | 10 ulp | bit-exact |
| gen-m9/a7bf0d8d81d33ba6 A81 | 5, 3 | L140 | 5 ulp | bit-exact |
| gen-m9/fec39efaba9284c3 A36 | 8.5, 2.5 | L140 (accurate ln B above 8) | 9 ulp | 1 ulp (bit-exact with M1) |
| gen-m9/caa1eed48b1883fe A380 | 2.5, 6 | bpser | 4 ulp | 3 ulp |
| gen-m9/f34f66fb7f81e673 A4 | 4, 10 | L140 | 11 ulp | 3 ulp |
| gen-m9/bcb24385baafb2f0 A377 | 3.5, 7.5 | bpser (x = 8.7E-28) | 13 ulp | 3 ulp |
| gen-m9/b6593d7dd13f2dac A306 | 6, 11.5 | bpser (x = 6.3E-13) | 26 ulp | 25 ulp, visibly off |

A377 has a nested FILTER, so the sweep leaves it unparsed; `filter_cell.py` forms its arguments.
A306 stays off: its data hold 123456789.123 next to values near 1, Excel's p is 19 ulp from the
exact tail at the two-pass f, the exact variances move the model's p by 3 ulp only, and the z that
would give Excel's p is 48 units (2^-53) from bratio's, about eight ulps of x. Open in T-2829; the cells 1 to 3 ulp off in T-2830.

## Files
- `dump_cells.py`: the sweep with the engine's value, to JSON (run from tools/).
- `score_bratio.py`, `score_bratio.out`: the per-branch table and the M1 hybrid.
- `filter_cell.py`: A377's arguments.
