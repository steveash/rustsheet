---
id: T-1862
title: "Legacy MATCH argument intersects a non-overlapping range to VALUE (corpus-bench 49300)"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
corpus-bench/49300 `Sheet1!C2` matches Excel's cached `#VALUE!`.

## Context
Split from T-1791. A legacy (no `cm`) cell `=INDEX(Data!C3:AY4,MATCH(1,(Sheet1!A2=Data!B3:B4)*(Sheet1!B2=Data!C2:AY2),0))` is `#VALUE!` in Excel and `#N/A` in the engine. Suspect implicit intersection of `Data!B3:B4` from row 2 of another sheet (no overlap: `#VALUE!`) inside a legacy cell's MATCH lookup_array argument; check how the xlsx import marks the formula legacy (T-1717 added `@` placement for legacy formulas) and which `@` the engine inserts here. Inspect with `./target/release/rustsheet-eval show corpus-bench/49300/1_49300_input.xlsx` (fetch the suite first). Out of scope: other legacy-parse work (T-1096).

## Acceptance criteria
- [x] corpus-bench/49300 C2 passes, with a unit or spec test for the cause; any quirk goes into docs/excel-semantics/.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- Cause: a legacy cell intersects arithmetic, comparisons and concatenation over a range in MATCH's `lookup_array` and in the table of VLOOKUP/HLOOKUP (FORMULATEXT `=MATCH(1,(@A1:A6="b")*(@B1:B6=2),0)`). The engine left them whole. Now `is_lookup_table` (static_analysis/legacy.rs) adds the `@`. XMATCH stays whole.
- Probing found two more gaps, both fixed:
  - A single-value table is `#VALUE!` for non-numeric text and `#N/A` otherwise (`Model::lookup_table`, now shared by MATCH, LOOKUP, VLOOKUP and HLOOKUP).
  - A one-cell reference (written, or picked by IF or INDEX) is searched as a range: `MATCH("b",A2,0)` was `#N/A` and is now 1.
- Evidence: probes/legacy-match-lookup-array-intersection and probes/match-single-value-lookup-array (excel-graph, 2026-10-18), recorded in references.md and lookup.md. Unit test `match_over_a_single_value`.
- Eval delta: corpus-bench known failures 16 -> 14 (49300 and 59734 fixed). The 2 new probe cases add 2 known failures (J2/Y2, an IF arm in a legacy lookup_array, filed as T-1935). 0 regressions in every suite.
- Follow-ups: T-1935 (legacy IF arm), T-1937 (other single-cell forms; the numeric-text boundary).
