---
id: T-1352
title: "Probe and match TRIMRANGE with a fractional trim_rows or trim_cols (gen-m7 bcad709c A20)"
milestone: M6
lane: functions-lookup
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
TRIMRANGE with a non-integer trim_rows or trim_cols (e.g. 2.5) behaves as Excel does, which the evidence suggests is `#VALUE!`.

## Context
T-1275's gen-m7 suite (excel-graph sidecars, 2026-10-11): gen-m7/bcad709c30cfdf64 Calc!A20 `=(UPPER(IFERROR(Data!B1,Data!C11))&REPLACE("abc",TRIMRANGE(Data!C10:E11,2.5,3),1,PROPER("")))`:
Excel `#VALUE!`, engine `#SPILL!` (the engine reads trim_rows 2.5 as 2 and returns the 2x3 range, which REPLACE lifts).
The left operand is "42", so the `#VALUE!` comes from the REPLACE part; the likeliest cause is TRIMRANGE rejecting 2.5, but
REPLACE with a 2x3 reference start_num is the alternative (docs/excel-semantics/lookup.md "TRIMRANGE with a fractional
trim argument", UNVERIFIED). Probe first with the excel-probe skill: `TRIMRANGE(A1:C3,2.5)`, `(A1:C3,1.5,3)`, `(A1:C3,3,2.5)`,
`(A1:C3,0.9)`, `(A1:C3,"2")`, `(A1:C3,TRUE)`, `(A1:C3,4)`, `(A1:C3,-1)`, and `REPLACE("abc",A1:C2,1,"")` with numbers in A1:C2.
Code: crates/rustsheet-engine/src/functions/lookup_and_reference/trimrange.rs.

## Acceptance criteria
- [ ] gen-m7/bcad709c30cfdf64 A20 passes in `./target/release/rustsheet-eval show gen-m7/<case>.case.json`, with no new failing gen-m7 cell and no new known failures in eval/baseline/.
- [ ] The probe cells pass; lookup.md's entry is VERIFIED with the probed rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
TRIMRANGE now rejects any non-integer trim_rows/trim_cols with `#VALUE!` (probe `probes/trimrange-fractional-trim`, excel-graph; `0.9`, `1.5`, `2.5`, `3.9` all `#VALUE!`; `"2"` and `TRUE` coerce). gen-m7/bcad709c30cfdf64 A20 passes; lookup.md entry VERIFIED. Probe cases added to the probes baseline.
