# 0084 — Allow two COTH/CSCH regressions unmasked by T-0917

- **Status:** accepted (2026-10-20, T-0917)
- **Context:** T-0917 makes COTH, CSCH and SECH 1 over Excel's own TANH, SINH and COSH. All 66
  pairs of probes-numeric `t0917-hyperbolic-switchover` (excel-graph) show Excel computes them that
  way, bit for bit, including where its TANH/SINH are 1-2 ulp off the true value
  (docs/excel-semantics/numbers.md "COTH, CSCH and SECH are 1 over Excel's own TANH, SINH and
  COSH"). With ATANH/ACOTH this fixes 39 cells of ironcalc-calc TRIGONOMETRIC.xlsx. Two cells
  passed before only by accident, because libm's `cosh/sinh` and `1/sinh` happened to give Excel's
  value where the engine's own TANH(0.123) and SINH(0.88) do not:
  - `COTH_CSCH_SECH!B10` (COTH(0.123)): the engine's TANH(0.123) misses `SINH_COSH_TANH!D10`
    already (a known failure); COTH inherits it.
  - `COTH_CSCH_SECH!C54` (CSCH(0.88)): the engine's SINH(0.88) misses probes-numeric m9-hyperbolic
    A234; CSCH inherits it.
  The same cause fails one previously passing cell of the ungated suite probes-numeric:
  `m9-hyperbolic` Calc!A126 (CSCH(0.001)). In the new case `t0917-hyperbolic-switchover`, five cells
  fail that libm's old formulas would pass: A81 CSCH(0.001), A97 COTH(0.123), A105 CSCH(0.3), A109
  COTH(0.42), A159 CSCH(0.88). Against these, probes-numeric gains 333 cells.
- **Options considered:**
  - *Keep libm for COTH/CSCH.* Contradicts the 66 probe pairs and gives up the fixed cells.
  - *Special-case the two arguments.* Fitting to cells, not to Excel's formulation.
  - *Fit Excel's SINH/TANH kernel in the same PR.* Tried in T-0917 without success (numbers.md "SINH
    and TANH on 0.4 .. 1: no formula found yet"); it is its own research task.
- **Decision:** run `rustsheet-eval update-baseline --allow-regressions` once in the T-0917 PR, for
  exactly these two cells. They, m9-hyperbolic A126 and the five switchover cells join T-2134's acceptance list (the SINH/TANH kernel fit).
- **Consequences:** `eval/baseline/ironcalc-calc.txt` loses 39 lines and gains these two. Reviewers
  confirm the baseline diff adds no other line. T-2134 must make them pass.
