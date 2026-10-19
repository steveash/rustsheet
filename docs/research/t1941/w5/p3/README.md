# W5 part 3: degree reduction and soft-direction search for the near-1 kernel (T-1941)

Base: c5_f99t with Excel's tie-line bits applied (D3 -> ...b68, N2 +1 ulp; see `../lines/README.md`).
Scores are exact far/near rates on the `../metrics.py` samples.

## 1. Free refits of reduced degree: random level (`degred.py`)
Relative minimax of c5's own function at (9,8), (8,8) and (8,9), with D0 = 2^-13:
- the function errors are 3.9e-28, 2.5e-22 and 3.3e-25;
- the exact scores are far 0.29-0.31, near 0.36-0.39 (random-smooth).

They reproduce the function but not c5's stage coordinates. That is expected, so this alone decides nothing.

## 2. Pinned refits, W4-style (`pinfit.py`)
- Method: drop the top coefficient(s), then solve a weighted least-squares correction of the rest.
  - Function rows: 80 points, tolerance FTOL relative.
  - Stage pins: Taylor coefficients of every visible Horner stage at x = 1, orders 0-2, taken from c5.
    "Visible" means ulp(S_j)/|C(1)| >= 3e-18; D's top stages, about 1e-20 in the output, are excluded.
  - Pin tolerances: order 0 = K0·2 ulp(S_j); order k = 2 ulp(S_j)/TT^k.
- **Near-1 data pin each stage sum only loosely.** A compensated shift of S_j(1) by k ulps stays invisible until
  k·|t| ~ 1 (part 1, `control2.py`). So K0 = 1e3-1e5 is the physical setting, and K0 = 1 over-constrains.

| structure | K0 = 1, TT = 1e-3 | K0 = 1e3, TT = 1e-6 | best (9,8): K0 = 1e5, TT = 1e-8 |
|---|---|---|---|
| (9,9) | 0 (c5 itself) | 0 | - |
| (9,8) | rms 1.3e4 | rms 35, max 173 | rms 3.5, max 14; far 0.256, near 0.316 |
| (8,8) | rms 6e7 | rms 1.3e6 (K0 = 1e3, FTOL 3e-17) | infeasible |
| (8,9) | rms 3e7 | rms 4.8e4 | infeasible |

- **N must have degree 9.** (8,8) and (8,9) cannot keep c5's function and near-1 stage structure by 1e4-1e6
  tolerance units. The worst rows are N.S7's derivative and the mid stage sums.
- **(9,8), with D9 dropped, is feasible within loose pins.** But every feasible refit loses the near-1 basin:
  near 0.25-0.52 against c5's 0.75.

## 3. Exact search along soft directions (`softsearch.py`)
- Directions: SVD of the function-preservation map (relative N/D change per ulp of each coefficient, 80 points),
  scaled by 1/1e-16; the 16 smallest singular directions.
- Steps: ±{0.3 ... 3e5} ulps per direction, greedy on far+near exact (25k + 25k sample), 4 sweeps.

| start | far+near before | after | metrics far / near after |
|---|---|---|---|
| c5 (bits fixed), (9,9) | 0.499 | 0.503 | 0.307 / 0.752 |
| (9,8) pinned refit | 0.292 | 0.450 | 0.310 / 0.624 |

- **No W4-style jump.** G went from 30% to 73% this way. Here (9,9) stays on its plateau, and (9,8) climbs only
  to below c5.
- Degree reduction does not unlock this kernel, and the evidence favours (9,9), or (9,8) with D9 below its
  visibility.
