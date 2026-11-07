# T-4635: GAMMA.DIST at a huge finite x / beta

`gen.py` builds probes/t4635-gamma-dist-huge-ratio. It and these predictions were committed before the excel-graph
fill.

Evidence before the fill (probes/t4625-gamma-dist-ratio-overflow): at x / beta = 1E308, GAMMA.DIST is `#NUM!` at
shapes 1.5 and 2 in both forms, and a number at shapes 0.5, 1, 2.5, 2.9, 3, 10 and 1E5. At shape 2 it is a number at
x / beta 1E200 and below.

Hypothesis H, predicted before the fill:
- `#NUM!` exactly for 1 < a < 2.5 at x / beta 1E308, and a number for a = 1 and a >= 2.5.
- Every shape is a number at x / beta 1E200.
- CHISQ.DIST and CHIDIST at df 2 to 5, x 1E200 and 1E290 (Excel for the web refuses a workbook with x 1E307 or more, first
  fill), show what the shared incomplete gamma does below the cut; a `#NUM!` there would show that the cut
  comes from the shared incomplete gamma.

The ratio cut is unknown. The 1E250 to 1.7E308 columns locate it. Nothing is tuned to the fill: the rule recorded is
what the cells show, and a cut is implemented only where the probe brackets it.

Run: `python3 -I docs/research/t4635/gen.py eval/suites/probes/cases/t4635-gamma-dist-huge-ratio.case.json`

## First fill (excel-graph 2026-11-07) and the second probe

Note: the first probe's columns are x / beta 1E216, 1E258, 1E298, 1E306, 1E307, 1E308, 1.5E308 and 1.7E308 (x times
1E8), not "1E200 and up" as the generator's comment says.

What the first fill shows (cdf and density alike, every cell): shape 1 is a number everywhere; shape 1.01 is `#NUM!`
from 1E306, shape 1.1 from 1E298; shapes 1.25 to 2.1 from 1E308; shapes 2.25 to 2.51 from 1.5E308. Hypothesis H is
refuted (shapes 2.25 to 2.51 fail at 1.5E308; 1.01 and 1.1 fail below 1E308). CHISQ.DIST and CHIDIST at df 2 to 5 are
numbers at 1E200 and 1E290.

Hypothesis H2, committed before the second fill (`gen2.py`, probes/t4635-gamma-dist-huge-ratio-cut):
- Low regime, shapes just above 1: `#NUM!` exactly where y^a overflows a double (y = x / beta), which fits 1.01
  (cut 1E305.2) and 1.1 (cut 1E280.2). It cannot hold at 1.25 (1E258^1.25 overflows, a number), so it stops at a
  shape between 1.1 and 1.25.
- High regime: `#NUM!` where y * (4 - a) overflows, for 1 < a < 4 (fits every first-fill cell from 1.25 to 2.51,
  and 2.9 and 3 at 1E308 from T-4625). Shapes 0.5 and 1 do not follow it (numbers at 1E308).
- The cut is on x / beta, not on x: the Beta sheet repeats the first fill's verdicts.

## Second fill (excel-graph 2026-11-07): H2 half right; rule R frozen before the third probe

The Beta sheet repeats the beta 1E-8 verdicts at beta 1E-100 and 1E-200: the cut is on y = x / beta. The low regime
fits H2 at every cell (1.01 cut 1E305.2, 1.02 1E302.2, 1.05 1E293.6, 1.1 1E280.2, all bracketed) and stops between
shape 1.1 and 1.15 (1.15 to 1.3 are numbers up to 1E306). The high regime refutes y * (4 - a) (shape 1.5 fails at
7E307). Candidates scored on the Hi sheets' brackets of c = DBL_MAX / cut: 4 - a, exp(k (4 - a)) and a / (a - 1) each
miss a bracket; 4 / a sits in every bracket (1.25 and 1.5: 2.57 to 3.59; 2: 2.0 to 2.57; 2.1 to 2.2: 1.80 to 2.0;
2.25: 1.63 to 1.80; 2.5, 2.6: 1.50 to 1.63; 2.75: 1.38 to 1.50; 2.9, 3: 1.28 to 1.38; 4 and 10: below 1).

Rule R (frozen; gen3.py is the fresh sweep that tests it, generated after the freeze):
- Shape at or below 1 (0.5, 0.9, 0.99, 1): no `#NUM!` at a finite y.
- 1 < shape <= 1.1: `#NUM!` where y^shape overflows a double, or where 4 * y / shape does.
- shape > 1.1 (from 1.15 evidenced): `#NUM!` where 4 * y / shape overflows, that is y > shape * DBL_MAX / 4; never from
  shape 4 up. The cut between 1.1 and 1.15 is unknown (Cut sheet narrows it).
Predicted for gen3: Fresh column A (0.97 of the cut) a number, B (1.03) `#NUM!`, the same in C and D; shapes 0.3 and
0.7 numbers.

## Third and fourth fills (excel-graph 2026-11-07): rule R holds

Fresh sweep (gen3): 84 of 84 cells as rule R predicted (0.97 of the cut a number, 1.03 `#NUM!`, both forms, 21
shapes from 1.25 to 3.47); shapes 0.3 and 0.7 numbers at 1.79E308. Cut sheet: shapes 1.11 to 1.14 are numbers at
every y to 1E306, so the low regime ends in [1.1, 1.11). Chi-squared (gen4): every cell as predicted. The engine
implements rule R (gamma.rs `huge_ratio_num_error`); statistics.md "GAMMA.DIST at a huge finite x / beta (T-4635)"
records it.
