---
id: T-1590
title: "Probe collation equality in exact-match lookups, SWITCH and text criteria, and the unprobed accents"
milestone: M2
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for where Excel's collation equality (`"æ"="ae"`, `"ß"="ss"`, hyphen-sensitive) applies beyond the `=`/`<`/`>` operators, and for the accent order T-0220 took from Windows' sort keys.

## Context
T-0220 reviews (Excel fidelity, MINOR): `compare_values` now uses `functions::util::collation::cmp_text_excel`, so exact MATCH/XMATCH/XLOOKUP/VLOOKUP and SWITCH treat `"æ"` and `"ae"` as equal (inferred, coercion.md). The text criteria disagree among themselves: `"ae"` and `"<>ae"` (criteria.rs `result_is_equal_to_string`, `result_is_not_equal_to_string`) compare lowercased code points, while `<`, `<=`, `>`, `>=` use the collation. Also unprobed: the order of tilde, cedilla and stroke (`ã` vs `å`, `ç`), letters outside Latin-1 (`œ`, `þ`, `ð`, Greek), Latin-1 symbols (`§`, `¿`, `«`), control characters, and the hyphen/apostrophe position rule beyond the T-1469 examples. Use the excel-probe skill; file engine follow-ups for every mismatch.

## Acceptance criteria
- [x] A probe case (oracle-filled) covering `MATCH(...,0)`, `XLOOKUP`, `VLOOKUP(...,FALSE)` and `SWITCH` with æ/ae, ß/ss, a-b/ab, a/á; `COUNTIF` with `"ae"`, `"<>ae"`, `"<=ae"` over {"æ","ae","a-e"}; and the unprobed characters above ranked as in probes/text-collation-order.
- [x] Findings in docs/excel-semantics/coercion.md and criteria.md; engine follow-up tasks filed for mismatches.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Added probes/text-collation-lookups and probes/text-collation-unprobed (oracle-filled, excel-graph). Findings: XMATCH/XLOOKUP/SWITCH and COUNTIF use collation equality, MATCH/VLOOKUP do not (engine already agrees); COUNTIF `=`/`<>` text criteria and 89 rank cells of the unprobed characters mismatch the engine. Recorded in coercion.md and criteria.md; follow-up T-1594. Eval delta: +363 probe cells, 100 new known failures (all T-1594).
