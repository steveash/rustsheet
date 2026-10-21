---
id: T-2278
title: "Probe gen-m6 lookup residue: descending MATCH with a blank key, LOOKUP's extended result vector as a reference, INDIRECT over XLOOKUP"
milestone: M6
lane: functions-lookup
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
The three unexplained gen-m6 lookup cells are explained by a probe and matched: a descending MATCH with a blank key over an unsorted vector, a LOOKUP whose result vector is shorter than the match position (under MMULT), and INDIRECT over an XLOOKUP with an array search_mode.

## Context
Gap triage of gen-m6 (T-2180, excel-graph oracle, 2026-10-21). Observations: docs/excel-semantics/lookup.md "Unexplained gen-m6 lookup cells"
(added by this triage).
- 50f0dd9a8ee2cb57 A204: MATCH with blank key, match_type -2.2, over 0.5, 0.3, 0, -2.5, 2.5, -1, 1, 0.1, 100 is 3 in Excel; the documented
  descending search ("Approximate-match lookups", "MATCH descending search skips backward") gives 9, as ours. Probe the same vector with keys
  0, blank, 0.0001, -0.0001 and match_type -1, and shorter prefixes, to see whether a blank key or an exact 0 changes the search.
- 50f0dd9a8ee2cb57 A315: LOOKUP(TRUE, B2:B10, E11:E11) matches the 9th cell; ours returns 0 (E19's blank value) and MMULT gives 0. Probe
  `ISREF(LOOKUP(..))`, `MMULT(LOOKUP(..),2.5)`, `ROWS(LOOKUP(..))`, `CELL("address",LOOKUP(..))`.
- e3a562557696e2fc A33: `INDIRECT(XLOOKUP(..,101,INDEX(Data!D4:E11,0)),0.2)` is `#REF!`, ours `#VALUE!`. Probe XLOOKUP with an array
  search_mode alone and under INDIRECT.
Code: crates/rustsheet-engine/src/functions/lookup_and_reference/lookup_table.rs, xlookup.rs. MATCH and LOOKUP are M2 functions (catalog);
filed at M6 with the gen-m6 triage. Why opus: three probes and an unexplained search.

Failing cells (`make eval-show CASE=gen-m6/<case>.case.json`), 3 in eval/baseline/gen-m6.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m6/50f0dd9a8ee2cb57 Calc!A204 | `=DATE(2088,MATCH(Data!B2,Data!E4:E12,(Data!$E5+Data!E7)),30)` | 68757.0 | 68941.0 |
| gen-m6/50f0dd9a8ee2cb57 Calc!A315 | `=WEEKDAY(MMULT(LOOKUP(Data!A$1,Data!B2:B10,Data!E11:E11),Data!C$2))` | "#VALUE!" | 7.0 |
| gen-m6/e3a562557696e2fc Calc!A33 | `=INDIRECT(XLOOKUP(Data!D3,Data!E5:E9,Data!C3:D8,(Data!C$5+Data!E$4),(Data!$E9+100),INDEX(Data!D4:E11,0)),Da...` | "#REF!" | "#VALUE!" |

## Acceptance criteria
- [ ] These 3 known-failure lines disappear from eval/baseline/gen-m6.txt (`make eval-baseline`), with no new known failure in any suite, or each remaining one has a filed task named in the Outcome.
- [ ] A new probes case (excel-graph) holds the probes above; lookup.md replaces the "Unexplained" entry with the rules found.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other lookup functions.

## Visible-parity stage
Decision 0083: every cell above is visibly off (a wrong error, a wrong type or a value off in a leading digit). The acceptance criteria above are the stage.

## Progress (opus run, 2026-10-21, session budget reached)
Probes committed with oracle values: probes/lookup-match-descending-blank (18 cells, ours 8 off) and
probes/lookup-short-result-vector (7 cells, ours 2 off); findings in lookup.md "Unexplained gen-m6 lookup cells". Their
known failures are not baselined yet: run `make eval-baseline` only as part of the fix, or record them first if the
fix is split. Left to do: reconcile the MATCH result (a scan that stops at the first value below the key?) with
lookup.md "MATCH descending search skips backward" and its evidence before changing lookup_table.rs; make an
extended LOOKUP result the blank-cell value (lookup.rs); probe the XLOOKUP/INDIRECT cell in a smaller case (Excel for
the web refused one holding INDIRECT over XLOOKUP with array search_mode and CELL("address",..)).

