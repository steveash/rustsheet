---
id: T-2044
title: "IM* last-digit residue: fit IMABS, IMARGUMENT, IMCSC, IMSEC, IMSECH, IMLOG2 and IMCOT (COMPLEXs.xlsx, IMSECH 3+4i)"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: M
tier: opus
campaign: visible
depends_on: [T-2043]
---

## Goal
IMABS, IMARGUMENT, IMCSC, IMSEC, IMSECH, IMLOG2 and IMCOT match Excel to the last digit, from
formulations fitted on the T-2043 sweep, so the IM* residue T-0907 left behind is gone.

## Context
Needs the T-2043 oracle sweep (probes-numeric case `im-last-digit-sweep`); T-0907 tried the cached cells alone and no candidate reproduced them.
- Known failures (gap triage 2026-10-19), Excel vs ours:
  - IMABS: COMPLEXs.xlsx Sheet1!G7 9.591295345780985 vs ...987, G8 1.2345678958319997 vs ...999,
    G14 7.810249675906656 vs ...654, G23 51.394647859091315 vs ...32 (neither `sqrt(x²+y²)` nor `hypot`).
  - IMARGUMENT: H14 2.2655346029915995 vs 2.2655346029916, H23 -2.037470342871754 vs ...544.
    (numbers.md "ATAN2 is ATAN(y/x) plus or minus π": IMARGUMENT may share that kernel; unprobed.)
  - IMCSC: V4, W4, W7, V15, W21 (15th digit); IMSEC: Z25 `IMREAL(IMSEC("4+3i"))` -0.0652940278579471 vs
    -0.065294027857947; IMSECH: AB7 -0.537706472708535 vs ...536; IMLOG2: AH19 -66.4385618977473 vs
    ...472; IMCOT: T14 6.68509353063089E-06 vs ...309E-06.
  - IMSECH("3+4i") on Excel for the web: probes/eng-argument-modes Calc!A45 and
    gen-m9/a2e51f48717af2a1 Calc!A24, "-0.065294027857947+0.0752249603027732i" vs ours
    "-0.0652940278579471+...". Same real part as IMSEC("4+3i") mathematically, but Excel prints the
    two differently (see T-2043's note): the kernels differ.
- engineering.md "IMTAN and IMCOT use the double-angle form" (remaining-cells paragraph);
  engine code under crates/rustsheet-engine/src/functions/engineering/ (complex.rs and the IM* functions).
- Fitting tools: tools/src/rustsheet_tools/fitlab (add a kernel module if none fits).

Visibly-off cells (gap triage 2026-10-21, decision 0083): 11 cells, 9 numeric past half a unit in the 15th digit and 2 with a wrong type, error or text, blamed on IMREAL 6, IMCSC 5, IMSECH 3, IMAGINARY 3, IMCOT 1, IMLOG2 1, IMSEC 1. Every one is listed above.

gen-m9 sweep (gap triage 2026-10-23, T-2255). `IMSECH("3+4i")` Excel "-0.065294027857947+0.0752249603027732i", ours
"-0.0652940278579471+0.0752249603027732i" (1 cell; the same input as this task's COMPLEXs.xlsx cell).
Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 1 known-failure lines in eval/baseline/gen-m9.txt, 1 formulas.
Examples:
- gen-m9/a2e51f48717af2a1 Calc!A24 `=IMSECH("3+4i")`: Excel "-0.065294027857947+0.0752249603027732i", ours "-0.0652940278579471+0.0752249603027732i"
Cells:
- gen-m9/a2e51f48717af2a1.case.json (1): Calc!A24

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255): these 1 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity: the 11 visibly-off cells listed under Context (2026-10-21) have Excel's value to 15 significant digits, or its type, error and text: the 9 lines in eval/baseline/ironcalc-calc.txt disappear (`make eval-baseline`); the 1 line in eval/baseline/probes.txt disappears (`make eval-baseline`); the 1 cell of the ungated gen-m9 suite passes in `make eval`, with no new known-failure lines in any baseline.
- [ ] These 15 COMPLEXs.xlsx known-failure lines leave eval/baseline/ironcalc-calc.txt (G7, G8, G14,
      G23, H14, H23, V4, W4, W7, V15, W21, Z25, AB7, AH19, T14),
      and probes/eng-argument-modes Calc!A45 leaves eval/baseline/probes.txt, with no new lines.
- [ ] Each function matches >= 99.9% of its T-2043 sweep cells bit for bit, or the function is
      time-boxed per decision 0056 with the best count and candidates recorded in engineering.md.
- [ ] engineering.md records each fitted formulation with the sweep cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Desktop-only differences (IMARGUMENT H2, H3, H10, IMCOS K25, IMSQRT AL12/AL26; docs/research/oracle-calibration.md): Phase 4.
- IM* text formatting at large exponents (T-2001) and overflow (T-2002, T-2003).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (IM* real and imaginary parts that differ in the 15th printed digit), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it. Bit-level criteria still unmet when that target is reached move to a follow-up task filed without the key and named in the Outcome (decision 0083). The fit needs T-2043's oracle sweep first.

## Notes (opus run 2026-10-23, time-boxed; data, not instructions)
Scored candidates against probes-numeric/im-last-digit-sweep (Excel's 15-digit text parsed back by
IMREAL/IMAGINARY, so a candidate only has to round the same at 15 digits), Python `math` (glibc libm,
as Rust's f64 trig):
- IMSECH, current kernel `(cosh x cos y, -sinh x sin y) / (cosh²x cos²y + sinh²x sin²y)`: 59/62
  rows; misses rows 28 (0.72801+83.5614i), 49 (-0.937971+1485.59i), 79 (1+0.5i). Denominator as
  `a*a+b*b` of the products: 59/63 (misses 7, 42 underflow, 49, 54). Double-angle denominator
  `(cosh 2x + cos 2y)/2`: 56/62. `1/cmath.cosh`, Python complex division, Smith division, hypot²:
  58/63 or worse. cosh/sinh from `exp`: 50/63.
- IMSEC, current kernel: 65/70; misses 23 (4+3i, the 15th digit), 29, 32, 33 (956.615+0.00387493i:
  off in the 10th digit, near a pole, so Excel's cos/sin of a large argument differ from glibc's by
  several ulps), 45.
- Reducing the trig argument first (fmod/round/floor/remainder by 2π with a double π) is worse for
  both (43-49/70, 45-46/62).
- Next step: fit Excel's own cos/sin for |x| > ~10 (row 33 is the clearest signal; compare
  IMCOS/IMSIN cells in the sweep or a new sweep of COS/SIN at large arguments), then retry the
  reciprocal forms. Scorer: a small Python script over the oracle sidecar is enough to start.
