---
id: T-1769
title: "Fuzz timeout: MAP whose LAMBDA returns a multi-cell range materialises the range once per element"
milestone: M2
lane: functions-logic
status: in-progress
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
A MAP whose LAMBDA returns a multi-cell range, such as `=SUM(MAP(A1:A1000,LAMBDA(r,B1:B100000)))`,
evaluates in time that does not grow with the returned range's size, keeping today's value, so the
nightly `formula_eval` fuzz target stops timing out on it (CLAUDE.md directive 5).

## Context
Nightly fuzz run 39000004030 (2026-10-17, main 22a43ad7), job "fuzz (formula_eval)", artifact
`fuzz-crash-formula_eval-39000004030`, file `timeout-c35a3145eeefd97b304778a58028d48a76a29463`.
The reproducer is the 46-byte formula (bytes, as printed by libFuzzer, decode to this exact text):
`SUM(MAP(A1:A1000,LAMBDA(rSMU,((B1:B100000)))))`, typed into Sheet1!C3 of the fuzz model
(fuzz/fuzz_targets/formula_eval.rs). libFuzzer reported a timeout (> 30 s).

The steward reproduced it on ea76d91f with a release build through the facade (same model):
- `SUM(MAP(A1:A10,LAMBDA(r,B1:B100000)))`: 0.12 s; `A1:A100`: 0.51 s; `A1:A1000`: 5.3 s, all
  `#VALUE!` ("Error in array"). `ROWS(MAP(A1:A1000,LAMBDA(r,B1:B100000)))` is 1000 in 5.2 s.
- The cost is calls x range height. `BYROW(A1:A1000,LAMBDA(r,B1:B100000))` (#CALC!) and
  `MAKEARRAY(1000,1,LAMBDA(r,c,B1:B100000))` take well under 1 ms.
- Root cause: `fn_map` (functions/logical/map_reduce.rs) passes every call's result through
  `lambda_result_value` (functions/logical/bycol_byrow.rs), which turns a multi-cell
  `CalcResult::Range` into a full `CalcResult::Array` with `result_to_array` before the element is
  stored, so each of the 1,000 calls reads 100,000 cells.

Excel semantics are not settled here: what MAP shows for an element whose LAMBDA returns a
multi-cell range (today `#VALUE!` per element; BYROW gives `#CALC!`) needs oracle evidence before
any value changes (excel-probe skill). This task only removes the per-element materialisation and
keeps today's values; a value change, if a probe shows one, is a separate task.

Repro: `cd fuzz && cargo fuzz run --target "$(rustc -vV | sed -n 's/^host: //p')" formula_eval <file>`
with the formula above as the file's bytes (decision 0035).

## Acceptance criteria
- [ ] The reproducer is a unit test (decision 0035) that runs in well under a second in a debug test
      build and returns the same value as before the fix.
- [ ] A MAP element whose LAMBDA returns a multi-cell range costs O(1) or O(used area), not the
      range's full size; check the other LAMBDA helpers that share `lambda_result_value` (SCAN,
      REDUCE, MAKEARRAY, BYROW/BYCOL) for the same path.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing what MAP returns for such an element (needs a probe first).
