# 0217 — Gate A exempts ABS, MIN and MAX as exact and SUMIF and SUMIFS as composite

- **Status:** accepted (2026-11-01, T-3550; an `exempt.tsv` change under decision 0199)
- **Context:** Gate A (decision 0188 R2, SPEC-0029 "Gate A holds") holds only when every in-scope catalog
  function with `returns: number` has a reference table or an `exempt.tsv` line. `rustsheet-eval accuracy`
  on main printed `missing tables 323`. Ten of them are M2 functions, and M2 is the current milestone.
  T-3548 gives ROUND, ROUNDUP and ROUNDDOWN tables, and T-3549 does the same for SUM and AVERAGE. That leaves
  five functions:
  - **ABS** (`number`, `mode: value`) returns its argument or the argument's negation. Both are exact in
    binary floating point, so there is no kernel whose rounding a table could measure. The coercion of a text
    argument (numbers.md: `ABS` of a 256-character text is `#VALUE!`) is a semantic rule that the eval suites
    test, not an accuracy question.
  - **MIN, MAX** (`number`, `mode: aggregate`) return one of their numeric arguments unchanged, or 0 when
    there is none. No arithmetic is done on the value. Which arguments count is coercion
    (coercion.md "MIN / MAX coerce value arguments like SUM; ranges and arrays give only their numbers"), and
    the eval suites test it.
  - **SUMIF, SUMIFS**: the catalog marks `range`, `sum_range` and `criteria_range` as `mode: range`, so a table
    of scalar literals cannot drive them. Their numeric part is a plain left-to-right sum that never snaps
    (numbers.md "Summing functions", T-3048, probes/t3048-snap-conditional-sums: 5.551115123125783e-17 over
    0.1, 0.2, -0.3). The report's "Excel evidence per function" table (`make eval`, 2026-10-31) shows Excel
    cells for both: SUMIF in fixtures, gen-m2, gen-m5–m9, ironcalc-calc, probes, probes-numeric and smoke;
    SUMIFS in fixtures, gen-m2, gen-m5–m7, gen-m9, ironcalc-calc, probes and probes-numeric. Gate B counts
    1571 and 1528 attributed cells for them.
- **Decision:** `eval/accuracy/exempt.tsv` lists ABS, MAX and MIN as `exact` and SUMIF and SUMIFS as
  `composite`, naming gen-m2, probes and ironcalc-calc as the suites that drive them.
- **Consequences:** `missing tables` drops by 5, from 323 to 318. After T-3548 and T-3549, no M2 function is
  missing a table. The SUMIF and SUMIFS sums stay measured only by the eval suites, where Gate B counts
  them. If a later probe shows a scalar-literal form that Excel accepts for either function, a table task
  replaces the exemption, with its own decision record. AVERAGEIF, AVERAGEIFS, MINIFS, MAXIFS and the other
  criteria functions outside M2 are classified by T-3551, not here.
