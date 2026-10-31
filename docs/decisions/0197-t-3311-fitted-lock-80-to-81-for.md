# 0197 — T-3311: fitted lock 80 to 81 for NEWTON_START_MAX, duplication banked, A107 one ulp off under decision 0083

- **Status:** accepted (2026-10-31, T-3311). It applies decision 0188 R4.3 and decision 0083's standing ulp
  allowance, and adds no rule.
- **Context:** T-3311 makes Excel's lower-tail gamma quantile at a shape above 1 the Newton steps from the
  leading-factor root on T-3060's cdf form, keeping the first iterate whose cdf is nearest p
  (`incomplete_gamma::closed_form::excel_newton_quantile`; docs/excel-semantics/statistics.md, "Deep lower tail at a
  shape above 1"). The steps apply only where the form at the root is within `NEWTON_START_MAX` of p. That cut is a
  region chosen by scoring oracle cells, so it is a fit (decision 0188 R4):
  - it carries a `FITTED(T-3311, ...)` tag and applies only in its measured range;
  - its hold-out, fresh sweeps and Gate A envelope are in T-3311's Outcome, with the 50-digit references and the
    scripts in docs/research/t3311.
- **The cut and A107.** The hold-out picks `NEWTON_START_MAX` = 1E-6 (the wider of the two cuts tied best on its
  train part, and the leader on its test part). The review (T-3311 round 1) asked for that cut over the 1E-8
  tie-break the branch held. At 1E-6 one gated cell loses its bit:
  - gen-m9/c18dbf21f11093f3 A107 `CHISQ.INV(POISSON.DIST(9,100,FALSE),8)`: Excel 7.92099982566666E-8, ours
    7.920999825666661E-8, +1 ulp, within 15 digits. No Newton iterate is Excel's value there, and no density or form
    ordering T-3311 tried changes that.
  - What 1E-6 gains over 1E-8, gated: gen-m9/a5828d3925575713 A338 `GAMMAINV(1E-16,2.5,TRUE)` bit-exact (-5 ulp to
    0) and gen-m9/a612e89321a6df9c A169 `CHISQ.INV(1E-20,6)` within 15 digits (-5 to -4 ulp), both visibly fixed.
    Against main: 6 fixed, 4 closer, 6 visibly fixed, 1 lost (A107), nothing farther or newly visibly off.
  - On fresh sweep 2 (probes-numeric/t3311-fresh-sweep-2, 360 cells, filled after the 1E-6 model was frozen), exact /
    within 4 ulp / within 15 digits: 1E-6 179 / 261 / 285 of 360 and 23 / 28 / 30 of the 30 in-region cells; 1E-8
    165 / 249 / 278 and 11 / 18 / 25; main 148 / 231 / 268 and 1 / 9 / 23. Its one cell farther than main at 1E-6
    is ungated: A346 `CHISQ.INV.RT(1-1.30E-14,4)`, -1 to -3 ulp, within 15 digits.
  - A107 meets all four conditions of decision 0083's standing allowance: it is numeric and within 4 ulp; nothing
    becomes visibly off; the PR fixes 6 visibly-off cells against 1 regression; T-3311's Outcome lists it and T-3403
    names it as its ulp follow-up.
  - **Owner ruling.** On 2026-10-31, in the T-3311 session
    (https://claude.ai/code/session_01WTJ7rq9tsQyspbybqNLk1a), the owner chose "Allow, with records": T-3311 ships
    1E-6 and A107 goes from bit-exact to 1 ulp off under decision 0083's allowance, recorded here.
- **Decision:**
  - `NEWTON_START_MAX` is 1E-6, with its measured range (23 points, shape 1.5..7.5, p 1E-200..1.0000000827E-10).
  - `rustsheet-eval update-baseline --allow-regressions` runs once in the T-3311 PR, for A107 alone.
  - quality/fitted.json rises by one, for this one new fit (`scripts/check_fitted.py --update`).
  - quality/duplication.json banks a drop in duplicated lines (`scripts/check_duplication.py --update`; the count
    only fell).
- **Consequences:** eval/baseline/gen-m9.txt gains exactly the A107 line and loses the fixed cells' lines; reviewers
  confirm the baseline diff adds no other line. T-3403 owns A107's 1-ulp miss. The fitted ratchet stays one-way for
  new fits, each with its own record.
