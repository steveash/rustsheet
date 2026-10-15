---
id: T-0220
title: "Text comparison uses Excel collation, not code-point order (2+ gen-m2 known failures)"
milestone: M2
lane: engine
status: in-progress
priority: P2
size: M
tier: opus
depends_on: [T-1469]
---

## Goal
Text comparison (`<`, `>`, `<=`, `>=`, sorting order inside lookups and criteria) uses Excel's
collation, not Unicode code-point order.

## Context
gen-m2 (T-0203) evidence (excel-graph):
- `gen-m2/2cfe279fe77a000d` Calc!A318 `IFERROR((Data!D7>=Data!E2),...)` with D7 = "x", E2 = "ü": Excel TRUE
  ("ü" sorts with "u", before "x"), engine FALSE (U+00FC > "x").
- `gen-m2/96a0952412cefa6c` Calc!A284 `IF((Data!E5>Data!A12),"a",...)` with E5 = "1.5", A12 = "?b": Excel
  TRUE (punctuation sorts before digits), engine FALSE ('1' U+0031 < '?' U+003F).
- Lookup cells that the code-point emulation in T-0212 gets wrong probably come from the same rule, e.g.
  `gen-m2/60a543744fc4a8f7` Calc!A280 (`HLOOKUP("?b",...)`) and `gen-m2/a339ea84ef64382e` Calc!A246.
- Added by T-0212 (decision 0037): two lookup cells that passed only while the search compared across
  types and now fail on code-point order: `gen-m2/2cfe279fe77a000d` Calc!A328 (`VLOOKUP("x~*",...)`
  over a column whose only text is "ü"; Excel ranks "ü" below "x") and `gen-m2/ef5c55a81783b7d5`
  Calc!A151 (`VLOOKUP("1",...)` over {blank, FALSE, "?b", 1}; Excel ranks "?b" below "1").

- Added 2026-10-13 (planner): `gen-m2/d04d501154afc771` Calc!A97 `VLOOKUP(Data!B8,Data!D3:E6,..,TRUE)` with B8 = "ü"
  over {-0.0001, FALSE, 0, "x"}: Excel `#N/A` ("ü" ranks below "x", so no text at or below it), engine "x"
  (code-point order puts U+00FC above "x").
- Probe split out on 2026-10-13: T-1469 (research) fills the pairwise collation probe. Fit the key against it.

- Added by T-0244: `functions::util::cmp_text_excel` is the one text-order function (CHAR(160) right
  after the space); extend it rather than adding another. It leaves `probes/text-order-nbsp-space`
  Calc!D17 failing: a hyphen is skipped, `"-1" > "0"` is TRUE in Excel.

This is cross-cutting (comparison operators, MATCH/VLOOKUP/HLOOKUP, criteria `<`/`>`, SORT). It needs a
probe workbook first. Use the excel-probe skill: ASCII punctuation, digits, letters, accented letters
and case, pairwise. Then write a short spec if the table is non-trivial (CLAUDE.md directive 4).

Code: `compare_values` and the text comparison in `crates/rustsheet-engine/src/` (`rg compare_values`).

## Acceptance criteria
- [ ] The collation rule fitted to T-1469's probe is recorded in docs/excel-semantics/coercion.md ("Text comparison uses Excel's
      collation"), and the T-1469 probe cells pass.
- [ ] One collation key function used by all comparison sites. The two cells above disappear from
      eval/baseline/gen-m2.txt with no new F lines, and so do the two decision-0037 cells, d04d501154afc771 A97
      and probes/text-order-nbsp-space Calc!D17.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Locale variants other than en-US.
