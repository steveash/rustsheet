---
id: T-2663
title: "Desktop-only cells in the calipers goldens: CELL info types and filename, INFO(release), trig near pi/2 and CHAR(255) on Mac"
milestone: M14
lane: research
status: blocked
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Each calipers cell that differs only because the golden comes from desktop Excel (Mac) is settled
by desktop evidence, and either matched or recorded as a desktop-only rule.

## Context
Phase 1 assessment, 2026-10-25 (decision 0114). These visibly-off calipers cells follow desktop
behavior that Excel for the web, the Phase 1 oracle of record (decision 0039), does not show:
- `roundtrip/formulas_information`: `CELL("color"|"filename"|"format"|"parentheses"|"prefix"|"protect"|"width",$C$5)`
  (21 rows each). The visibly-off cells this task owns (2026-10-28 triage, `IFERROR(CELL(..),"Error")`, Excel a value, ours "Error"):
  formulas_information Sheet1 E140:E160, G140:G160, H140:H160, I140:I160, J140:J160, K140:K160, N140:N160 (147 cells).
  Excel for the web's answer for "color", "parentheses", "prefix", "protect" and "width" is unprobed (the engine's `#VALUE!` there is
  a not-implemented placeholder; "format" and "filename" are `#VALUE!` on excel-graph): T-3011 probes them.
  Excel for the web gives `#VALUE!` for "filename" (information.md, T-2052); the
  desktop golden gives 0 (color), the absPath-based path, `G`, 0, "", 1 and 15. Real models use
  `MID(CELL("filename",A1),FIND("]",CELL("filename",A1))+1,255)` to read the sheet name
  (pv26_011 and pv26_2_011 B3), so desktop parity here matters for everyday files.
- `roundtrip/formula_stress_test` `Information!B3` `INFO(C3)`: desktop "16.0", web `#VALUE!` (T-2182).
- `roundtrip/formulas_math_trig` row 22: `COS`, `TAN`, `SEC`, `COT` of `PI()/2` on Mac
  (COS 6.123233995736766E-17, ours 6.1257422745431E-17), like the Mac SIN(4) cells of decision 0074.
- `roundtrip/formulas_text` `C190` `CHAR(255)`: Mac "ˇ", ours "ÿ".

## Acceptance criteria
- [ ] With the excel-desktop backend (M14), each cell above is re-checked on Windows desktop
      Excel; matching rules are implemented behind the workbook's platform or recorded as
      desktop-only in docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing excel-graph-evidenced behavior before desktop evidence exists.

## Blocked because
Needs the excel-desktop oracle backend (decision 0039, M14): `cd tools && uv run rustsheet-oracle status`
must show `excel-desktop OK`.
