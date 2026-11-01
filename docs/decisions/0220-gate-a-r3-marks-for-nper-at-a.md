# 0220 — Gate A R3 marks for NPER at a tiny positive rate

- **Status:** accepted (2026-11-01, T-3535). Applies decision 0188 R3, case 2, to committed Gate A points.
- **Context:**
  - Decision 0188 R3 lists NPER at 0 < rate < 1e-14 as an approved `MATH-OVER-EXCEL` case. Rates where 1 + rate is 1
    are excluded: they are `#DIV/0!` in Excel and stay so. Negative tiny rates are not covered. The case is recorded
    in financial.md, "NPER at a tiny positive rate".
  - T-3535's first NPER table (`eval/accuracy/tables/NPER/s1`, excel-graph and 50-digit mpmath) holds 12 points inside
    that region where Excel is more than G = 1e-4 off the math: A121, A130, A133, A135, A142, A143, A151, A152, A153,
    A156, A158 and A162, at rates from 3.0E-16 to 7.6E-15.
  - Excel is off by more than the case's "up to 4%" at two of them:
    - `NPER(3.04353262019695E-16,370.793811118894,-6303.49478902119,0,0)` (A135) is 23.0 in Excel against a true
      17.0, 35% off.
    - A158 (rate 5.1E-16) is 15% off.
    The error grows as the rate nears the 1 + rate = 1 cut, where ln(1+rate) holds only a few bits.
  - `eval/accuracy/math-over-excel.tsv` is a protected path (T-3379), so marking them needs this record.
- **Decision:**
  - The 12 points are R3-marked against the financial.md entry, inside the approved region.
  - The entry's rule now says Excel's error reaches 35% near the cut, citing these points.
  - The region is unchanged: positive rates below 1e-14 where 1 + rate is not 1.
  - Today the engine reproduces Excel there, so each marked point is also a known failure owned by T-3367, which
    moves the engine to the math.
- **Consequences:**
  - The marked points are checked against the math only. Each keeps its evidence on the point: the check rejects a mark
    where Excel is not more than G off the math.
  - T-3367's acceptance includes removing these 12 known-failures lines once the engine returns the math.
  - The other NPER points with Excel more than G off the math are outside the region and are not marked. They are
    ill-conditioned annuities (rates of 0.01 to 4.8 over long horizons) where Excel's double arithmetic loses digits.
    The engine matches Excel to the bit there, so they pass Gate A on Excel's side.
  - No eval cell is marked.
