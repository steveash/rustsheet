---
id: T-3775
title: "Owner ruling: is SIN/COS/IMSIN/IMCOS at the 15-digit neighbour of k·pi/2 (Excel 4.1E-4 off the math) an R3 case?"
milestone: M9
lane: research
status: blocked
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Record the owner's ruling on whether Excel's trigonometry at the 15-digit neighbours of k·pi/2 is a decision 0188 R3
(`MATH-OVER-EXCEL`) case. Until the owner rules, the engine keeps matching Excel.

## Blocked because
Blocked because: needs an owner ruling (decision 0188 R3: "no agent, steward included, adds a case on its own").

## Context
SIN, COS and the IM* trigonometry reduce x by the x87 64-bit pi (numbers.md "SIN / COS at multiples of 90°", "Trig of
a huge angle"; engineering.md "IMSIN and IMCOS reduce the real part by the x87 64-bit pi", T-3726). Where x is the
15-digit text nearest k·pi/2, the remainder is a few 1E-16, and the 64-bit pi's error of about 1E-20 per quarter turn
leaves Excel's result above R3's G = 1e-4 relative to 50-digit math:
- `COS(RADIANS(90))` is 6.1257422745431E-17 where the math gives 6.123233995736766E-17 (4.1E-4).
- probes-numeric/t3726-imsin-imcos-near-half-pi-sweep: 36 of its 800 parts are 4.1E-4 off. All of them are at
  x = ±4.71238898038469, e.g. `IMCOS("4.71238898038469+79.1718046156471i")`, whose real part is Excel
  -2223964481544680000 against math -2.22305384529439E+18. Tool: docs/research/t3726/excel_vs_math.py.
The engine matches Excel at all of these points (directive 1). The options for the owner: (a) keep matching Excel, so
no R3 case; (b) an R3 case over a bounded region (which functions, which |x - k·pi/2|, which k), with a decision record.

## Acceptance criteria
- [ ] A decision record carrying the owner's ruling, or the owner's "keep matching Excel" written into the numbers.md
      and engineering.md entries.
- [ ] If (b): a follow-up implementation task with its region, and the region's evidence committed first
      (decision 0188 R3).

## Out of scope
- Changing the engine before the ruling.
