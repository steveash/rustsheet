---
id: T-2044
title: "IM* last-digit residue: fit IMABS, IMARGUMENT, IMCSC, IMSEC, IMSECH, IMLOG2 and IMCOT (COMPLEXs.xlsx, IMSECH 3+4i)"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-2043]
---

## Goal
IMABS, IMARGUMENT, IMCSC, IMSEC, IMSECH, IMLOG2 and IMCOT match Excel to the last digit, from
formulations fitted on the T-2043 sweep, so the IM* residue T-0907 left behind is gone.

## Context
Needs an oracle sweep: T-2043 (T-0907 tried the cached cells alone and no candidate reproduced them).
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

## Acceptance criteria
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
