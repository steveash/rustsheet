---
id: T-1307
title: "Probe and match CLEAN and PROPER character rules: non-printing characters above 31, letters after digits and apostrophes, non-ASCII letters"
milestone: M5
lane: functions-text
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
CLEAN and PROPER match Excel on the characters their argument-mode probes never reached. Each
function gets at least 25 Excel cells, so it leaves the "below 25" list in report.md's "Excel
evidence per function" (decision 0031).

## Context
Evidence gap (eval/runs/latest/report.md, 2026-10-10): CLEAN has 6 Excel cells and PROPER 15. The
existing cells only check argument modes and lifting (probes/text-argument-modes C21, C28;
probes/indirect-text-broadcast A84, A90; ironcalc-calc and poi cells; docs/excel-semantics/text.md
"Text function argument modes"). These engine choices are unverified:
- `fn_clean` (crates/rustsheet-engine/src/functions/text/char_code.rs) removes code points 0-31
  only. Microsoft documents that CLEAN keeps 127, 129, 141, 143, 144 and 157 (unverified here), and
  what it does with other Unicode controls and format characters (U+0080-U+009F, U+200B, U+FEFF)
  is unknown.
- `proper_case` (crates/rustsheet-engine/src/functions/text/string_format.rs) capitalises the
  first letter after any non-alphabetic character, using Rust's `char::is_alphabetic`,
  `to_uppercase` and `to_lowercase`. So `PROPER("ßa")` becomes "SSa", because Rust upper-cases ß
  to "SS". Letters after digits and apostrophes ("2nd", "o'neil", "x-ray"), Greek and Cyrillic
  final forms, titlecase digraphs (U+01C6) and combining marks are unprobed.

Suggested cells: `CLEAN(CHAR(n))` for n = 7, 9, 10, 13, 31, 32, 127, 129, 141, 143, 144, 157
(as `LEN(CLEAN(…))` and `CODE(CLEAN(…)&"x")`), `LEN(CLEAN(UNICHAR(8203)))`,
`LEN(CLEAN(UNICHAR(65279)))` and `LEN(CLEAN(UNICHAR(133)))`. Also `PROPER` of "2nd place",
"o'neil mc-donald", "x-ray", "ßa straße", "élan ÉCOLE", "ǆemal", "ΣΊΣΥΦΟΣ", "école"
(combining acute), "hello_world", "a1b2c3" and "ＡＢＣ ｄｅｆ" (full-width).

## Acceptance criteria
- [ ] A probe case (excel-probe skill, excel-graph sidecar) with the cells above; report.md lists
      CLEAN and PROPER with at least 25 Excel cells each.
- [ ] The engine matches every cell, or each mismatch has its own functions-text task naming its
      cells.
- [ ] docs/excel-semantics/text.md records the CLEAN character set and the PROPER word-start and
      case-mapping rules, citing the probe cells.
- [ ] `make check` green; no new F lines in other suites; reviewers approve.

## Out of scope
- UPPER and LOWER case mapping (file a follow-up if the ß or final-sigma results differ there too).
- TRIM (already evidenced) and UTF-16 counting (T-0249).

## Outcome
Added probes/clean-proper-chars (88 Excel cells; CLEAN 38, PROPER 50 with their LEN companions). CLEAN already matched. PROPER now uses Excel's one-to-one case mapping (ß, ŉ, ǰ, ligatures kept; İ to I/i; ΐ, ΰ to Ϊ, Ϋ) and the final sigma ς; documented in text.md. Eval: probes 0 known failures in the new case, no regressions. Follow-up: UPPER/LOWER of the same characters unprobed.
