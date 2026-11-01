# 0224 — Gate A's SUM and AVERAGE include the near-zero snap and the running-total overflow

- **Status:** accepted (2026-11-01, T-3549). Settles T-3549's design question 1 (option (a)) for decision 0188's Gate A.
- **Context:**
  - Gate A compares each point against its 50-digit math (decision 0188, SPEC-0029). For SUM the plain math is the
    exact sum of the doubles of the written literals. AVERAGE divides it by the count.
  - Two VERIFIED Excel rules make Excel differ from that sum by far more than G = 1e-4 relative. Neither is an
    inaccurate kernel:
    - **The near-zero snap.** numbers.md, "Near-zero snapping in addition and subtraction", fitted on 1,296
      probes-numeric `snap-addsub/*` cells and refined by T-3100, T-3145, T-3248 and T-2714. Every SUM call returns
      exactly 0 when the final addition's two addends have opposite signs and the result is below 8 ULP of the
      running total before the last addend. That bound is itself flushed when it is subnormal. AVERAGE snaps its sum
      before dividing. So `SUM(0.1,0.2,-0.3)` is 0, while the exact sum is 2.8E-17: Excel's relative error is 1.
    - **Intermediate overflow.** numbers.md, "Overflow in `+ - * /` is #NUM! where it happens", aggregate
      overflow, T-0192. `SUM(9E+307,9E+307,-9E+307)` and the matching AVERAGE are `#NUM!`, although the exact sum
      is a finite 9E+307. `SUM(9E+307,-9E+307,9E+307)` is 9E+307.
  - An R3 mark (follow the math) would make the engine drop a deliberate, evidenced rule, which directive 1 forbids.
    Listing the points as Excel-over-G points (option (b)) keeps a definition that knowingly disagrees with
    Excel's documented function. Not drawing cancelling sums (option (c)) would leave the snap's 8-ULP edge out of
    Gate A.
- **Decision:**
  - SUM's and AVERAGE's Gate A definitions (`tools/src/rustsheet_tools/accuracy/defs/aggregate.py`) include both
    rules. Both depend only on the doubles of the arguments:
    - The math is 0 where the final addition snaps. Otherwise it is the exact sum, or the exact sum over the count.
    - A point where any left-to-right running total is beyond the double range is a domain error, so Excel's
      `#NUM!` is the target.
  - Both rules read Excel's running total: each addition rounded to a 64-bit significand and then to a double, as
    numbers.md "Division rounds twice" records for `+`. This is the engine's `extended_sum`, and in the tools it is
    `fitlab.exact.double_round`. The snap compares that double total with the bound. The exact sum is used
    everywhere else.
  - The tail stratum draws near-cancelling sums: short decimals, then a last addend that cancels them to within
    a few units of its 15th digit. Some of those sums end in a trailing 0, which is never snapped. So the 8-ULP
    bound is crossed on both sides.
- **Consequences:**
  - The snap points pass Gate A on the math side (math 0, engine 0, Excel 0). They do not need R3 marks or
    known-failures lines.
  - A cancelling sum that does not snap (for example `SUM(0.1,0.2,-0.3,0)`) keeps its exact-sum math, 2.8E-17. In
    Excel and the engine it is 5.551115123125783e-17, one double rounding of 0.1 + 0.2 away. It passes on Excel's
    side of the envelope, like any ill-conditioned point where the engine matches Excel to the bit.
  - The probes-numeric math sidecars of SUM and AVERAGE cells take the same definition, so a SUM cell that Excel
    snaps reads 0 in its sidecar too.
  - A change to either Excel rule (new evidence) changes the definition. The definition cites numbers.md for both.
