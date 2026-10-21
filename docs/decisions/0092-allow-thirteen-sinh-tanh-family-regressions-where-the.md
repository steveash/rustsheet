# 0092 — Allow thirteen SINH/TANH-family regressions where the expm1 form replaces the series and sinh/cosh

- **Status:** accepted (2026-10-21, T-2134)
- **Context:**
  - T-2134 fits Excel's SINH and TANH below |x| = 1. With a = expm1(x) and b = expm1(-x), Excel's SINH
    is (a - b)/2 and its TANH is (a - b)/((a + b) + 2): docs/excel-semantics/numbers.md "SINH and TANH
    below 1 are built from expm1". This replaces T-0914's Taylor-series SINH (below 0.5), the exp form
    (0.5 to 1), sinh/cosh TANH (below 0.9) and the e^2x TANH (0.9 to 1). COTH and CSCH are 1 over these
    (decision 0084), so they follow.
  - On probes-numeric `t2178-hyperbolic-unit-sweep` (1035 arguments, excel-graph) the expm1 form
    matches 773 of the 1015 SINH cells below 1 and 781 of the TANH cells, against 612 and 551 for the
    old forms. It wins or ties in every band from 1E-8 to 1. For every argument, some (a, b) within
    3 ulp of libm's expm1 reproduces both cells. The misses are Excel's own expm1, which is not
    libm's and is still unknown.
  - Ungated cells newly bit-exact: t2178 sweep 2236 -> 2627 of 3105, m9-hyperbolic 220 -> 241,
    t0917-hyperbolic-switchover 164 -> 176, power-ulp-sources 13 -> 20.
  - Gated: 14 cells are fixed, among them `TRIGONOMETRIC.xlsx` SINH_COSH_TANH!D10 and D40,
    COTH_CSCH_SECH!B10 and B40 (B10 is decision 0084's), `POWER.xlsx` inside_a_lambda!C3 and C4, and eight
    gen-m5 cells of T-2134's list (`SINH(0.3)`, `TANH(0.2)`, COTH/CSCH of small arguments).
    Thirteen fail that the old forms matched. They are two arguments only, where libm's expm1
    misses Excel's by an ulp:
    - x = PI()/4: `TRIGONOMETRIC.xlsx` SINH_COSH_TANH!D51 and COTH_CSCH_SECH!B51 (B51, SINH, was a
      known failure already); gen-m5
      0362d48f56108faa A10, 4d099bea7898ebf1 A361, 651e713f5ae232ed A239, b0d597ec5af5fb92 A180,
      b0ecb7ca78460b77 A8 and A294, ee1667411bbe2ccc A141 (TANH and COTH of PI()/4);
    - x = +-0.9999999999: SINH_COSH_TANH!B15 and B17 and COTH_CSCH_SECH!C15 and C17.
- **Options considered:**
  - *Keep the old forms above some cut.* To reach PI()/4 the cut must sit below 0.785. Cuts from 0.5
    to 0.78 cost 20 to 28 of the sweep's SINH cells and 45 to 63 of its TANH cells, against the nine
    cells of that argument. Replacing expm1 by exp(x) - 1 above a cut (from 0.2 to 1) never fits
    SINH(PI()/4) on the sweep's argument.
  - *Special-case the two arguments.* That fits individual cells, which decision 0056 forbids.
  - *Wait for Excel's expm1.* It has no worksheet function to probe directly. The shape above is
    the evidence for it, and a later fit of the expm1 kernel alone would bring these cells back.
- **Decision:** run `rustsheet-eval update-baseline --allow-regressions` once in the T-2134 PR, for
  exactly these thirteen cells.
- **Consequences:** `eval/baseline/ironcalc-calc.txt` and `eval/baseline/gen-m5.txt` gain the thirteen
  lines and lose the fourteen fixed ones. Reviewers confirm that the baseline diff adds no other line.
  The follow-up that fits Excel's expm1 owns these cells.
