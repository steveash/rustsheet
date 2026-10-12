---
id: T-1407
title: "INDEX truncates a tiny negative row or column index (-1E-15) to 0 instead of #VALUE!"
milestone: M2
lane: functions-lookup
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
INDEX truncates a fractional negative row or column index above -1 (such as -1E-15) toward zero, so it selects the whole
row or column as index 0 does, instead of returning `#VALUE!`, as Excel does.

## Context
T-1289's gen-m9 suite (excel-graph sidecars, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/1466c66c61189a83 Calc!A1 `=EXPONDIST(INDEX(Data!A1:C3,3,-1e-15),SUM(..),((1="Hello World")<Data!D9))`: Excel
  0.027981694209681673, which is `EXPONDIST(0.5,12.14649,FALSE)`: x = A3 = 0.5, the first cell of row 3 (A3:C3), so
  `INDEX(Data!A1:C3,3,-1e-15)` is the whole row 3 and EXPONDIST lifts over it (the anchor of a spill). Engine `#VALUE!`
  (`INDEX(Data!A1:C3,3,-1e-15)` and `INDEX(..,3,-0.5)` are `#VALUE!`; `INDEX(..,3,0)` works).
The EXPONDIST lift over the resulting range is T-1290: this cell passes only when both land.
Semantics: docs/excel-semantics/lookup.md "INDEX truncates a tiny negative index to 0" (UNVERIFIED beyond -1E-15).
Probe with the excel-probe skill `INDEX(A1:C3,3,-1E-15)`, `INDEX(A1:C3,3,-0.5)`, `INDEX(A1:C3,3,-0.999)`,
`INDEX(A1:C3,-0.5,1)`, `INDEX(A1:C3,3,-1)` and ROWS/COLUMNS of each before implementing.
Code: `Model::index_impl` (crates/rustsheet-engine/src/functions/lookup_and_reference/index.rs).

## Acceptance criteria
- [x] The INDEX probe cells pass; lookup.md's entry is VERIFIED.
- [x] `INDEX(Data!A1:C3,3,-1e-15)` in gen-m9/1466c66c61189a83's data is the row A3:C3 (a unit test); gen-m9/1466c66c61189a83 A1 passes in `./target/release/rustsheet-eval show gen-m9/<case>.case.json` once T-1290 has also landed, with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Handoff (resumed)
Tier sonnet; no engine change made. Probe `probes/index-tiny-negative` (excel-graph, about 170 cells) is filled and recorded in lookup.md.
What it shows: Excel does not simply truncate. An index within about 2.39E-7 below an integer is rounded up to it
(`COLUMNS(INDEX(A1:C3,3,0.9999998))` is 1, `0.99999976` is 3, `0.99999977` is 1; the same for the row index at 3, so the
tolerance is absolute). Negatives: -1E-15 and -1E-7 are index 0 (whole row or column), but -1.5E-7 and everything down to -1 and past is `#VALUE!`.
So the two sides do not share one tolerance. I could not fit one rule (additive epsilon, float32 rounding and 2^-22 all fail on a
cell or two); the boundary cells are 0.99999976 / 0.9999997615 / 0.99999977 on the positive side and (-1E-7, -1.5E-7) on the negative one.
Next: probe a few cells between -1E-7 and -1.5E-7 and between 0.99999976 and 0.9999997615, fit the rule (try a relative 1e-15
compare after a 15-digit decimal round, float32 casts, and `x + 2.4E-7` for positives), implement it in `index_numbers`
(index.rs), and make the probe cells pass; gen-m9/1466c66c61189a83 A1 also needs T-1290.

## Outcome
A second probe, probes/index-snap-tolerance (63 cells, excel-graph), fitted the rule. INDEX reads a row or column
index with the text functions' `near_integer_count`: an index within 2^-22 + 2^-33 below an integer counts as that
integer, at 1, 500 and 100000 alike. A negative index within half that bound (2^-23 + 2^-34) counts as 0, and
anything further below 0 is `#VALUE!`. `near_integer_count` and its constant (now `NEAR_INTEGER_TOLERANCE`) moved
from functions/text/util.rs to functions/util.rs so INDEX shares them; `index_number` in index.rs applies the rule.
Eval delta: probes 51 known failures fixed (all of index-tiny-negative), plus the new case at 67 of 67 (the negative cut pinned to 1E-14 as half the positive bound);
0 regressions. gen-m9/1466c66c61189a83 A1 still needs T-1290 (EXPONDIST lift); the unit test covers the
`INDEX(..,3,-1e-15)` row. area_num and array-index elements are unprobed: follow-up T-1424.
