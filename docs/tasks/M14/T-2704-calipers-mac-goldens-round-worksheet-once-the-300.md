---
id: T-2704
title: "Calipers Mac goldens round worksheet * once: the 300 formulas_nested chain cells that excel-graph's double rounding moves by one ulp"
milestone: M14
lane: research
status: blocked
priority: P3
size: S
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
The 300 last-digit known failures of `calipers/roundtrip/formulas_nested` are settled by Windows desktop
evidence. Either the engine matches each desktop platform's `*`, or the cells are recorded as a
desktop-only conflict that the web rule wins (decision 0039 item 4).

## Context
Gap triage 2026-10-25 (`rustsheet-eval run --suite calipers` on main c30bebf7). The 300 F lines of
`calipers/roundtrip/formulas_nested/golden.xlsx` in eval/baseline/calipers.txt (Extreme Depth 150,
XSheet Chain A 76, XSheet Chain B 74) are all NumericUlp, 1 ulp, none visibly off, and no open task
names them. Four products cause all of them. Every other cell only carries the ulp down a chain of
`*1.001`, `-0.5`, `MOD(ABS(x),100)` and `+1`:
- `Extreme Depth!E1834` `=E1833*1.001`, with E1833 49.81856401224343 (ours equal): Excel
  49.86838257625566, ours 49.86838257625567.
- `Extreme Depth!E4446` `=E4445*1.001`, with E4445 10.161093439757583: Excel 10.17125453319734, ours
  10.171254533197338.
- `XSheet Chain A!E919` and `E2225`: the same two products, read across sheets.

Exact simulation over all 5,518 `x*constant` cells of the workbook, from Excel's cached operands:
- the singly rounded product matches Excel's cached value in 5,518 of 5,518 cells;
- the twice-rounded one (64-bit significand, then a double) matches in 5,514.

The four cells that tell them apart are the four above, and each holds the singly rounded product.
excel-graph rounds `*` twice (docs/excel-semantics/numbers.md "Division rounds twice",
probes-numeric/double-rounding-ops A1-A16, T-1498), and the engine follows it
(`arithmetic::extended_product`). So the desktop Excel that saved this golden rounds `*` once.

T-2663 found that the sibling roundtrip goldens are Mac saves (Mac trig near pi/2, like the SIN(4) cells
of decision 0074). A build without x87 arithmetic (ARM, or SSE2 only) has no double rounding, so the
likely reading is a platform difference, not an error in our model. The quirk is recorded in numbers.md
"Desktop goldens round `*` once (calipers formulas_nested)".

Excel for the web is the Phase 1 oracle of record (decision 0039), so nothing changes before desktop
evidence exists. The open question for M14 is what Windows desktop Excel does:
- if it rounds twice like the web, this is a Mac-only rule;
- if it rounds once, `*` (and likely `+`, `-`, `/` and SQRT) differs between the web and Windows
  desktop, and a decision record must come before any platform switch.

## Acceptance criteria
- [ ] With the excel-desktop backend (M14), probes-numeric/double-rounding-ops (its `*`, `+`, `-`, `/`
      and SQRT pairs) is recalculated on Windows desktop Excel, and formulas_nested E1834 and E4446
      are re-checked there. The result goes into numbers.md, citing the cells.
- [ ] Then one of these holds:
      - a decision record adopts a platform rule, and the 300 formulas_nested lines leave
        eval/baseline/calipers.txt with no new lines in any baseline;
      - numbers.md records the 300 cells as a desktop-only conflict with the web rule, and the
        Outcome says so.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the excel-graph-evidenced double rounding before desktop evidence exists.
- The other calipers last-digit cells, which have owners: MEDIAN pv23_049 and pv23_050 (T-2476),
  PERCENTILE.INC formulas_statistical (T-1811), CONVERT formulas_engineering (T-2047), IRR pv23_006
  and pv23_064 (T-2131).

## Blocked because
Needs the excel-desktop oracle backend (decision 0039, M14). `cd tools && uv run rustsheet-oracle status`
must show `excel-desktop OK`.
