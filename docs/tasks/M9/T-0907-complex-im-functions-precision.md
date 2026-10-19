---
id: T-0907
title: "Engineering: IM* complex-function last-digit precision (COMPLEXs.xlsx)"
milestone: M9
lane: functions-engineering
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Fix the COMPLEX/IM* last-digit gaps in `ironcalc-calc/ENGINEERING/COMPLEXs.xlsx` using the
Excel-cached values already in the suite.

## Context
Kernel map (T-1522, decision 0056): rank 18 in `docs/research/numeric-kernel-map.md`; fit with the fit lab (T-1523).
Filed in draft PR #11 (2026-10-02, never merged) together with CONVERT, which T-0922 now covers;
salvaged by T-0031. COMPLEXs.xlsx: IMREAL/IMAGINARY/IMABS/IMARGUMENT/IMTAN/IMCOT/IMCSC/IMSEC/IMSECH/
IMLOG2 differ in the last 1-3 digits, e.g. `IMREAL(IMTAN(D2))` expected -1.11280735120837e-5, got
-1.11280735120933e-5. Excel's output here has 15 significant digits, so the formula identity matters
for cancellation-prone cases. The oracle calibration (docs/architecture/oracle.md, "Known
divergences") found IMARGUMENT, IMREAL/IMSQRT, IMCOS and IMTAN differ by a few ulps between Excel for
the web and desktop: use the desktop cached values here, and say which a golden comes from.
Code: `crates/rustsheet-engine/src/functions/engineering/`.

More cells (T-1289's gen-m9 suite, excel-graph, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/a2e51f48717af2a1 Calc!A24 `=IMSECH("3+4i")`: Excel "-0.065294027857947+0.0752249603027732i", engine "-0.0652940278579471+0.0752249603027732i".

## Acceptance criteria
- [ ] COMPLEXs.xlsx baseline lines in eval/baseline/ironcalc-calc.txt drop with no new F lines;
      record how many remain and why.
- [ ] Findings in docs/excel-semantics/engineering.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
CONVERT (T-0922), BESSEL (T-0900), trig/hyperbolic (T-0915, T-0917).
