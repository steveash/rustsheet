---
id: T-1238
title: "Add generator signatures for the M6 lookup and reference functions"
milestone: M6
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
`rustsheet-gen --milestone M6` can generate formulas for the 19 in-scope M6 lookup and reference functions: each has a full signature (`returns`, `params` with `mode`, `type`, `domain`, `prior`).

## Context
Split from T-1090, which signed the 20 M6 date and time functions. Left: ADDRESS, AREAS, CHOOSE, CHOOSECOLS,
CHOOSEROWS, COLUMN, COLUMNS, FORMULATEXT, GETPIVOTDATA, HYPERLINK, INDIRECT, LOOKUP, OFFSET, ROW, ROWS, TRANSPOSE,
TRIMRANGE, XLOOKUP, XMATCH. Take argument modes from oracle evidence (excel-probe skill), as T-1090 did with
probes/date-argument-modes. Watch for: OFFSET and INDIRECT are volatile (mark `volatile: true` so the generator skips
them, or decide otherwise with evidence); functions that spill (TRANSPOSE, CHOOSECOLS, CHOOSEROWS, TRIMRANGE) can
overwrite the next generated formula, so check how the generator handles spilling results (ISFORMULA used `cell: true`);
GETPIVOTDATA needs a pivot table; reference params (AREAS, ROW, COLUMN, FORMULATEXT) may make excel-graph refuse a
literal (as ISFORMULA did). Extend test_generator.py's `emitted` checks.

## Acceptance criteria
- [x] Every `scope: in` M6 lookup entry (no `oracle: none`) has a signature whose required-argument count matches excel-functions.tsv, or a recorded reason it cannot be generated.
- [x] A seeded `rustsheet-gen --milestone M6` sample is all accepted by excel-graph.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
18 of the 19 in-scope M6 lookup entries now carry signatures in eval/catalog/functions.yaml; GETPIVOTDATA has none (it
reads a pivot table, which generated workbooks lack; the catalog says so). INDIRECT's ref_text is an enum of Data
references (a Calc one could be circular). The generator gained `vector: true` (a one-column range for the LOOKUP,
XLOOKUP and XMATCH lookup vectors), tested by test_vector_params_get_one_column; test_generator.py checks all 18 are
emitted. Spilling results (TRANSPOSE, CHOOSECOLS, CHOOSEROWS, TRIMRANGE,
OFFSET) are generated as is: at the top of a formula they spill into the next one, which Excel and the engine both score
as `#SPILL!`, and nested they feed their consumer. OFFSET and INDIRECT stay generated: the harness skips only RAND* and the clock as volatile. Modes come from the
new probe probes/lookup-argument-modes (117 cells, excel-graph 2026-10-09): every scalar argument lifts, vector and array
arguments take array literals whole (lookup.md "Lookup and reference function argument modes"). A seeded sample
(`rustsheet-gen --milestone M6 --count 25 --seed 17`, not committed) was all accepted by excel-graph (0 failures); one
case aborted the engine on `BASE(245,30,1E+15)` (T-1243, P0), and the other 24 passed 918/960 cells. Eval delta: probes
+1 case, 117 cells, 34 pass, 83 new known failures (T-1241, T-1242). Follow-ups: T-1241, T-1242, T-1243.
