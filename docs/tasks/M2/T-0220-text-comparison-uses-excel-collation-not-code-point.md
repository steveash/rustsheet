---
id: T-0220
title: "Text comparison uses Excel collation, not code-point order (2+ gen-m2 known failures)"
milestone: M2
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: []
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

This is cross-cutting (comparison operators, MATCH/VLOOKUP/HLOOKUP, criteria `<`/`>`, SORT). It needs a
probe workbook first. Use the excel-probe skill: ASCII punctuation, digits, letters, accented letters
and case, pairwise. Then write a short spec if the table is non-trivial (CLAUDE.md directive 4).

Code: `compare_values` and the text comparison in `crates/rustsheet-engine/src/` (`rg compare_values`).

## Acceptance criteria
- [ ] Probe suite of pairwise text comparisons filled by the oracle. docs/excel-semantics/ entry (new
      `text.md` or `numbers.md` sibling) with the rule and evidence.
- [ ] One collation key function used by all comparison sites. The two cells above disappear from
      eval/baseline/gen-m2.txt with no new F lines.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Locale variants other than en-US.
