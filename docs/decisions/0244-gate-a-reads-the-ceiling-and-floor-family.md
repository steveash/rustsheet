# 0244 — Gate A reads the CEILING and FLOOR family's math at the written decimals

- **Status:** accepted (2026-11-01, T-3673). Narrows SPEC-0029 "Excel behavior to match" for CEILING, CEILING.MATH,
  CEILING.PRECISE, FLOOR, FLOOR.MATH and FLOOR.PRECISE, as decision 0223 does for the ROUND family.
- **Context:**
  - Excel cuts the ratio `number/significance` to 15 significant digits before taking the step, and breaks a tie at the
    16th digit away from zero (numbers.md, "FLOOR.PRECISE: a 16th-digit tie in the ratio cut goes away from zero",
    VERIFIED: probes/t2652-floor-ceiling-ties, probes/t2687-floor-ceiling-nonunit-ties, all seven functions).
  - A plain ceiling or floor of the exact ratio makes that rule look like an error, by a whole step. At
    `CEILING(100,1.01010101010101)` the ratio is 99.0000000000000099, the cut gives 99 and Excel's rule gives
    99.99999999999999, while the plain ceiling is 101.010101010101: 1% apart (the tables show the same thing:
    CEILING.MATH/s1 A81 `CEILING.MATH(700,-1.01010101010101,2)` is 699.9999999999999 in Excel, a plain ceiling 694 steps, 701.0101010101009), far more than decision 0188 R3's
    G = 1e-4. An R3 mark there would say Excel is inaccurate where its rule is decimal by design.
  - A table literal has at most 15 significant digits, so its double cut to 15 digits is the written decimal.
- **Decision:**
  - The math of these six functions is taken at the written decimals of `number` and `significance`: their exact
    ratio, cut to 15 significant digits with a 16th-digit tie away from zero, stepped up (CEILING forms) or down (FLOOR
    forms; a nonzero `mode` of the .MATH forms steps a negative number away from zero), times the significance
    exactly. The .MATH and .PRECISE forms take |significance|.
  - Domain errors, each matched by Excel's code: CEILING and FLOOR of a positive number by a negative significance
    (`#NUM!`), FLOOR of a nonzero number by a zero significance (`#DIV/0!`), and a ratio at least that of FLOOR.PRECISE/s1 A119 (48.98 times the double range)
    (`#NUM!`, numbers.md "CEILING and FLOOR are #NUM! when number/significance overflows", which this task found).
  - A ratio below 2^-1022 in magnitude is 0, so the result is 0 (numbers.md, "CEILING and FLOOR of a subnormal ratio
    are 0", VERIFIED).
  - A ratio between 0.5563 and 48.98 times the double range has no math value: the evidence brackets Excel's overflow
    boundary there (CEILING/s1 A75 and FLOOR/s1 A72 are numbers, FLOOR.PRECISE/s1 A119 is `#NUM!`) but does not pin
    it, nor whether the binary quotient or its 15-digit cut overflows.
  - A result beyond the double range, which no literal draw reaches, has no math value (not evidenced), nor does
    FLOOR(0,0), which no Excel cell evidences (the engine gives 0).
  - Modes are drawn as whole numbers only; how Excel reads a fractional mode is not evidenced.
  - Definition: `tools/src/rustsheet_tools/accuracy/defs/ceiling_floor.py`.
- **Consequences:**
  - On T-3673's tables (880 points, excel-graph 2026-11-01) every point is in the envelope, and every math domain error
    is an error of the same code in Excel (32 points, 15 of them overflowing ratios). FLOOR/e-T-3673 (60 log-uniform
    points) adds 9 more, 5 of them FLOOR's own overflowing ratios, so the overflow rule is evidenced for all six.
  - Excel is more than G = 1e-4 off this math (relative, with no floor) at no point, so there is no R3 candidate.
  - Excel cuts the binary quotient (numbers.md, "CEILING and FLOOR cut the binary quotient"), which can sit on the other side of a 16th-digit tie or a step from the exact
    decimal ratio. There Excel is one unit of the cut's 15th digit from this math (one step of the significance where the ratio
    is below 1E+15): CEILING/s1 A58 `CEILING(9023487987474.29,0.002)` (exact ratio 4511743993737145, a tie that goes up
    to ...7474.3; Excel 9023487987474.28) and FLOOR.MATH/s1 A99, and at ratios of 1E+105 and more FLOOR.MATH/s1 A173
    and CEILING.MATH/s1 A170; at most 4.9e-15 relative, far inside ε = 1e-9, so it needs no rule here. The
    accuracy check passes these points through both sides of the envelope (within ε of the math, and the engine within
    15 digits of Excel), with no known-failures line.
  - MROUND, TRUNC, ISO.CEILING and ECMA.CEILING are not covered; they need their own reading when they get tables.
