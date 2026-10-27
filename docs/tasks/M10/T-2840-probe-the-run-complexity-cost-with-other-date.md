---
id: T-2840
title: "Probe the unquoted-$ complexity cost with other date tokens, an escaped $, @ and literals after a locale tag"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
TEXT's format-code complexity limit matches Excel for `$` runs in the contexts that T-2836 only inferred.

## Context
T-2836 (probes/t2836-dollar-run-cost) fitted the unquoted `$` cost per context in formatter/complexity.rs: a section
holding `$` and no placeholder pays 3 once, 2 with a date token (only `d` probed); a closing run of only `$` after
quoted text pays 2; unquoted text directly after a locale tag pays 1 (only `$` and spaces probed). Inferred, unprobed:
- other date/time tokens: `"m"`, `"yyyy"`, `"h"`, `"ss"`, `"AM/PM"` + `REPT("$",n)` near the limit (n ~ 240..248);
- an escaped `\$` run: `REPT("\$",n)` and quoted `"a"` + `REPT("\$",n)` (each `\$` is two characters of the code);
- `$` in a section holding `@` (on a text value): `"@"&REPT("$",n)`;
- other unquoted text after a locale tag: `[$-409]` + `REPT("-",n)` and `REPT(":",n)` (a colon run costs 3 once).
Probe these first (excel-probe skill), then adjust complexity.rs if Excel differs. Semantics: number-formats.md
"TEXT format code complexity limit", "Unquoted `$`".

## Acceptance criteria
- [ ] A probe case with the ladders above is oracle-filled (excel-graph) and passes, with no new known-failure lines.
- [ ] number-formats.md "Unquoted `$`" states each context as probed instead of inferred.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Quoted-text or placeholder costs not involving `$`.
