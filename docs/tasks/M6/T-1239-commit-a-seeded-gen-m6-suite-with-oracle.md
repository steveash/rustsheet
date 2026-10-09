---
id: T-1239
title: "Commit a seeded gen-m6 suite with oracle values and file its gaps"
milestone: M6
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A committed, seeded `gen-m6` suite with excel-graph values scores the M6 date and time functions, and its gaps are filed as tasks.

## Context
Like T-1192 for gen-m5. T-1090's uncommitted sample (`rustsheet-gen --milestone M6 --count 25 --seed 13`) was all
accepted by excel-graph and the engine passed 957/1000 cells. Date and time misses seen there, to triage:
`YEARFRAC(11194,<text or blank>)` (Excel -31.65 in context, ours `#NUM!`), `DATEDIF(<cell>,36657,"MD")` (ours `#NUM!`),
`DATE(<cell>,NETWORKDAYS(<cell>,9857),<x>)` (ours `#NUM!`), `SUM(WORKDAY(MROUND(..),..,COUNTIF(..)))` (Excel 0, ours `#NUM!`),
`TIME(<cell>,8,34)` and `WEEKNUM(27631,NETWORKDAYS.INTL(..,6))` (Excel `#NUM!`, ours `#VALUE!`). Many other misses are M5
gaps already filed (FILTERXML, PHONETIC, BAHTTEXT, DBCS, PERCENTOF, MULTINOMIAL). T-1238's seed-17 sample also aborted the engine on `BASE(245,30,1E+15)` (T-1243, P0): land that fix first. Decide whether to commit after T-1238
lands (so the suite also covers lookup) or now for the date half.

## Acceptance criteria
- [x] eval/suites/gen-m6 is committed with oracle sidecars (non-gating, like gen-m5), and each distinct engine gap is a task.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Committed eval/suites/gen-m6: 25 cases, seed 13, with excel-graph sidecars for all 25. It is non-gating like gen-m5, so no baseline is written, and it covers the date and lookup halves now that T-1238 and T-1243 have landed. The engine scores 970/1000 cells. Where the 30 misses went:
- New tasks: TEXT bare letters, digits and empty format → T-1251; GCD/MULTINOMIAL `#VALUE!` beating `#NUM!` → T-1261; WEEKDAY serial 0 and return_type 0 → T-1253; WORKDAY/NETWORKDAYS serial 0 → T-1254; FILTERXML → T-1255; a reference-valued lookup_value in the lookup functions → T-1256; FIXED "-0" → T-1257; REPLACE with a huge start_num → T-1258; ASIN ulp → T-1259; INDEX with an array row_num → T-1260.
- Existing tasks: huge SIN/SEC arguments → T-1201; ACOSH NaN in ADDRESS → T-1202; DATE with a huge day → T-1237, whose planned fix is corrected by this evidence; BAHTTEXT → T-1214; TANH ulp → T-0917.
- No task: the five PERCENTOF cells, which are `#NAME?` web goldens (decision 0039).

Excel-semantics entries were added to number-formats.md, dates.md, coercion.md and text.md.
