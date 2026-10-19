---
id: T-2043
title: "Oracle sweep for the IM* last-digit residue: IMABS, IMARGUMENT, IMCSC, IMSEC, IMSECH, IMLOG2, IMCOT"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
An oracle-filled probes-numeric case gives T-2044 enough Excel for the web points to fit the
kernels of IMABS, IMARGUMENT, IMCSC, IMSEC, IMSECH, IMLOG2 and IMCOT, instead of fitting blind to
the 19 Macintosh-cached cells of COMPLEXs.xlsx (T-0907 tried and found no candidate on those alone).

## Context
- Remaining failures (gap triage 2026-10-19): ironcalc-calc/ENGINEERING/COMPLEXs.xlsx Sheet1!G7, G8,
  G14, G23 (IMABS), H14, H23 (IMARGUMENT), V4, W4, W7, V15, W21 (IMCSC), Z25 (IMSEC), AB7 (IMSECH),
  AH19 (IMLOG2), T14 (IMCOT); probes/eng-argument-modes Calc!A45 and gen-m9/a2e51f48717af2a1 Calc!A24
  `IMSECH("3+4i")` (web: real part -0.065294027857947, ours -0.0652940278579471). Inputs are the
  workbook's D column (`12+6i`, `1-2j`, `1+i`, `1.23+9.5121i`, `0.00012+1.23456789i`, `1+2i`, `-5+6i`,
  `7-12.6i`, `1E-20`, `12-3i`, `-23.1234-45.899i`, `4+3i`, ...; read them with
  `rustsheet-eval snapshot`).
- docs/excel-semantics/engineering.md "IMTAN and IMCOT use the double-angle form" lists the residue;
  docs/research/oracle-calibration.md shows the web agrees with the Mac cache on all of these cells
  except IMARGUMENT H2, H3, H10 and IMCOS K25 (desktop-only differences, not part of this sweep).
- Note for the fitter: mathematically IMSEC("4+3i") and IMSECH("3+4i") have the same real part, yet
  Excel gives -0.0652940278579471 for the first (Mac Z25) and -0.065294027857947 for the second (web),
  and ours is the other way round in both: the two functions use different kernels in Excel.

## Acceptance criteria
- [ ] A probes-numeric case (excel-probe skill, `rustsheet-oracle fill`, excel-graph) with, per
      function, the 25 COMPLEXs.xlsx inputs plus at least 40 more inputs covering each quadrant,
      parts from 1E-5 to 1E3, a pure real and a pure imaginary number; IMREAL and IMAGINARY of each
      text result are separate cells (so each part is scored as a number), plus the plain text result.
      No pending cells.
- [ ] The case is cited in T-2044's Context, and any new quirk (e.g. the IMSEC/IMSECH split) is
      recorded in docs/excel-semantics/engineering.md with the cells.
- [ ] Known failures appear only in the ungated probes-numeric suite (no change to gated baselines);
      `make check` green; reviewers approve.

## Out of scope
- Fitting or implementing any formulation (T-2044).
- Desktop-only differences (IMARGUMENT H2, H3, H10, IMCOS K25, ATAN2_POWER D14, D26): Phase 4 desktop oracle.
