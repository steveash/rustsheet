---
id: T-2039
title: "Arrays of LAMBDA values: SCAN with a LAMBDA accumulator spills LAMBDAs that MAP can call"
milestone: M7
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A SCAN whose accumulator is a LAMBDA gives an array of LAMBDAs that MAP can call, as in Excel:
`=MAP(SCAN(LAMBDA(z,z),{1,2,3},LAMBDA(f,x,LAMBDA(z,f(z)*10+x))),LAMBDA(g,g(0)))` spills 1, 12, 123.

## Context
Found by T-0439. probes/lambda-escaping-closures H2:J2 (excel-graph) spill {1,12,123}; the engine gives
`#VALUE!`: an array element (`ArrayNode`) cannot hold a LAMBDA, so SCAN's result loses them. Closures
themselves work since T-0439 (`Model::store_closure`). Probe what a cell shows for the bare SCAN, and
INDEX/TAKE of such an array, before choosing a representation; mind memory (an array of N LAMBDAs).

## Acceptance criteria
- [x] probes/lambda-escaping-closures H2, I2, J2 pass (and H3 once T-2038 lands).
- [x] New probe cells for the bare array of LAMBDAs and INDEX of it, with oracle values.
- [x] `make check` green; reviewers approve.

## Out of scope
- Chained-call parsing (T-2038).

## Outcome
An array element can hold a LAMBDA: `ArrayNode::Lambda(id)`. SCAN, MAKEARRAY, BYROW, BYCOL and MAP keep a LAMBDA
result as one (`calc_result_to_element`), and `result_to_array` makes a lone LAMBDA a one-element array (VSTACK of LAMBDAs,
MAP over one LAMBDA). MAP and REDUCE arguments, INDEX and XLOOKUP hand the LAMBDA on (`array_element_value`), so do the lifted
kernels (ISERROR FALSE); a cell shows `#CALC!`; every value read is `#VALUE!` (`array_node_to_calc_result`, SUM, COUNT/COUNTA,
comparison, statistics, NPV). A LET or LAMBDA call whose result is an array of LAMBDAs gives each its captures
(`ScopeResult::lambdas`, walked only when the scope made a LAMBDA). This drops the `lambda_elements_shown` workaround of T-2406.
Found on the way and fixed: an omitted TAKE/DROP count keeps every row or column (`TAKE(x,,-1)` was `#CALC!`).
Probed on excel-graph: probes/lambda-arrays (72 cells) and probes/take-drop-omitted-rows (15 cells); arrays.md "Arrays of
LAMBDAs" and "TAKE and DROP with an omitted count".
Eval: probes +2 cases (87 cells, all pass), lambda-escaping-closures H2, I2, J2 fixed (3 known failures fewer), 0 regressions.
Follow-ups: T-2477 (UNIQUE/SORT/XMATCH/comparison over LAMBDA elements); T-2463 updated for the new behaviour.
