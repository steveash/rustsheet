---
id: T-0907
title: "Engineering: IM* complex-function last-digit precision (COMPLEXs.xlsx)"
milestone: M9
lane: functions-engineering
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Fix the COMPLEX/IM* last-digit gaps in `ironcalc-calc/ENGINEERING/COMPLEXs.xlsx` using the
Excel-cached values already in the suite.

## Context
Filed in draft PR #11 (2026-10-02, never merged) together with CONVERT, which T-0922 now covers;
salvaged by T-0031. COMPLEXs.xlsx: IMREAL/IMAGINARY/IMABS/IMARGUMENT/IMTAN/IMCOT/IMCSC/IMSEC/IMSECH/
IMLOG2 differ in the last 1-3 digits, e.g. `IMREAL(IMTAN(D2))` expected -1.11280735120837e-5, got
-1.11280735120933e-5. Excel's output here has 15 significant digits, so the formula identity matters
for cancellation-prone cases. The oracle calibration (docs/architecture/oracle.md, "Known
divergences") found IMARGUMENT, IMREAL/IMSQRT, IMCOS and IMTAN differ by a few ulps between Excel for
the web and desktop: use the desktop cached values here, and say which a golden comes from.
Code: `crates/rustsheet-engine/src/functions/engineering/`.

## Acceptance criteria
- [ ] COMPLEXs.xlsx baseline lines in eval/baseline/ironcalc-calc.txt drop with no new F lines;
      record how many remain and why.
- [ ] Findings in docs/excel-semantics/engineering.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
CONVERT (T-0922), BESSEL (T-0900), trig/hyperbolic (T-0915, T-0917).
