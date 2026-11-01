# 0228 — Gate A classifies the M5, M6 and M7 functions with no table

- **Status:** accepted (2026-11-01, T-3551). Amends `eval/accuracy/exempt.tsv`, a protected path (decision 0199).
- **Context:**
  - Gate A (decision 0188 R1(a), SPEC-0029 "Gate A holds") needs every in-scope numeric catalog function to have a
    reference table or an `exempt.tsv` line.
  - On main (after #2667), `rustsheet-eval accuracy` counted 92 functions of M5, M6 and M7 with neither: 74 in M5, 16
    in M6 and 2 in M7. No open task covered them. The recount matched T-3551's list exactly.
  - `exact` means no floating-point kernel. `composite` means a table of literals cannot drive the function, and the
    reason names the eval suites that do.
  - The volatile functions fit neither by nature. Decision 0225 (T-3631, an owner ruling) already reads RAND,
    RANDARRAY and RANDBETWEEN for Gate B on an oracle-filled probe of their invariant cells.
- **Decision:** each of the 92 functions is classified below.
  - **`exact`, 16 functions.** Each has an integer result and no rounding question that a semantics entry leaves
    open:
    - the whole-day date functions: DATEDIF, DAYS, DAYS360, EDATE, EOMONTH, NETWORKDAYS, NETWORKDAYS.INTL, WORKDAY,
      WORKDAY.INTL. They read truncated serials (dates.md: `EDATE(0.5,1)` = 31, `EOMONTH(0.5,1)` = 59,
      `DATEDIF(0.5,60,"D")` = 60; gen-m6's fractional `DAYS` cells match Excel). Their results are whole serials or
      counts;
    - DATEVALUE (a whole serial from text) and ARABIC (a whole number from a numeral);
    - GCD and LCM (integer arithmetic up to 2^53, numbers.md "GCD / LCM argument bound");
    - MUNIT (0s and 1s);
    - N (its argument, or 1, 0);
    - PI (a constant).
  - **`composite`, 13 functions:**
    - AGGREGATE and SUBTOTAL: they take references only (numbers.md "AGGREGATE", references.md "SUBTOTAL and AREAS
      take references only");
    - VALUE, NUMBERVALUE, TIMEVALUE and DECIMAL: they parse text, which numeric literals cannot drive;
    - MDETERM, MINVERSE and MMULT: their 2-D matrix arguments cannot be written as the generator's one-row array
      literal, and two of them return arrays;
    - SEQUENCE: its result is an array;
    - RAND, RANDARRAY and RANDBETWEEN: the volatile ruling, below.

    Each reason names the eval suites that drive the function. gen-m5 to gen-m9 and probes hold 5 to 206 cases each.
  - **The volatile ruling:** RAND, RANDARRAY and RANDBETWEEN are `composite`, driven by decision 0225's invariant-cell
    probe (T-3631). A draw has no reproducible value to tabulate, and Gate B already judges these functions on that
    probe. So no third class, no SPEC-0029 change and no change to the `exempt.tsv` reader are needed.
  - **Needs a table, 63 functions,** filed as eleven table tasks of four to seven functions each, by family:
    - T-3668: SIN, COS, TAN, COT, SEC, CSC;
    - T-3669: ASIN, ACOS, ATAN, ATAN2, ACOT, DEGREES, RADIANS;
    - T-3670: SINH, COSH, TANH, COTH, SECH, CSCH;
    - T-3671: ASINH, ACOSH, ATANH, ACOTH;
    - T-3672: EXP, LN, LOG, LOG10, POWER, SQRT, SQRTPI;
    - T-3673: CEILING, CEILING.MATH, CEILING.PRECISE, FLOOR, FLOOR.MATH, FLOOR.PRECISE (after T-3555, reusing its
      decimal-math rule);
    - T-3674: ECMA.CEILING, ISO.CEILING, MROUND, EVEN, ODD, INT, TRUNC (after T-3555);
    - T-3675: COMBIN, COMBINA, FACT, FACTDOUBLE, MULTINOMIAL, QUOTIENT, MOD;
    - T-3676: PRODUCT, SUMSQ, SERIESSUM, PERCENTOF (after T-3549's variadic definitions);
    - T-3677: SUMPRODUCT, SUMX2MY2, SUMX2PY2, SUMXMY2 (array literals, T-3586);
    - T-3678: TIME, HOUR, MINUTE, SECOND, YEARFRAC.

    Some functions here have integer results but a floating-point input stage that Excel rounds in its own way, so
    each needs a table that reads Excel's rule:
    - EVEN, ODD, INT and TRUNC (numbers.md "INT cuts to 15 significant digits first", "ODD above 2^53 adds one in
      binary");
    - FACT, COMBIN and QUOTIENT (their products and quotients round);
    - HOUR, MINUTE and SECOND (dates.md "HOUR, MINUTE and SECOND round to the nearest second").
- **Consequences:**
  - `exempt.tsv` gains 29 lines (16 `exact`, 13 `composite`). `missing tables` falls by 29: from 305 to 276 on this
    branch. No exemption is stale.
  - The eleven table tasks bring M5, M6 and M7 to zero missing tables once they land.
  - M9's other functions (outside decision 0188's 18) are still unclassified. As T-3551 says, they need their own
    classification task once the M9 table slices settle the solver and distribution patterns.
  - If an `exact` function later shows a rounding question, its line comes out and a table task replaces it. That
    takes a new decision record, because the path is protected. The same goes for a `composite` function that the
    generator learns to drive, for example 2-D array literals for the matrix functions.
