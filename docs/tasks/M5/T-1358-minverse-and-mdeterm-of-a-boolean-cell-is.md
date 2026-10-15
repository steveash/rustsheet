---
id: T-1358
title: "MINVERSE (and MDETERM) of a boolean cell is #VALUE!, not a singular-matrix #NUM!"
milestone: M5
lane: functions-math
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
MINVERSE of a boolean (a cell or a value) is `#VALUE!`, as in Excel, instead of reading FALSE as a singular 0 (`#NUM!`); MDETERM matches once probed.

## Context
T-1275's gen-m7 suite (excel-graph sidecars, 2026-10-11): gen-m7/7c48e2456500644f Calc!A19 `=MAX(MOD(MINVERSE(Data!E1),(0.1/42)))` with E1 = FALSE: Excel `#VALUE!`, engine `#NUM!`
(`MINVERSE(Data!E1)` alone is `#NUM!`; `MDETERM(Data!E1)` is 0). docs/excel-semantics/arrays.md "MDETERM / MINVERSE: a blank
cell in the matrix is #VALUE!" now records the boolean cell too. Code: `coerce_to_f64` in
crates/rustsheet-engine/src/functions/math_and_trigonometry/matrix.rs (T-0943). Probe with the excel-probe skill
`MDETERM(A1)`, `MINVERSE(A1)` with A1 = TRUE/FALSE, `MINVERSE(TRUE)`, and a 2x2 range holding a boolean, then match all.

## Acceptance criteria
- [x] gen-m7/7c48e2456500644f A19 passes in `./target/release/rustsheet-eval show gen-m7/<case>.case.json`, with no new failing gen-m7 cell and no new known failures in eval/baseline/.
- [x] The probe cells pass; the arrays.md entry states the boolean rule as VERIFIED.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed (probes/matrix-boolean-elements, excel-graph): MDETERM, MINVERSE and MMULT give `#VALUE!` for a boolean cell, value or matrix element. `coerce_to_f64` no longer reads booleans as 1/0 (MMULT already rejected them). gen-m7/7c48e2456500644f A19 passes; the probe case passes 14/14; baseline gains only the new case's coverage line. arrays.md states the rule as VERIFIED.
