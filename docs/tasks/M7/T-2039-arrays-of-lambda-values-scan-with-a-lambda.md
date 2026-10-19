---
id: T-2039
title: "Arrays of LAMBDA values: SCAN with a LAMBDA accumulator spills LAMBDAs that MAP can call"
milestone: M7
lane: engine
status: todo
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
- [ ] probes/lambda-escaping-closures H2, I2, J2 pass (and H3 once T-2038 lands).
- [ ] New probe cells for the bare array of LAMBDAs and INDEX of it, with oracle values.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Chained-call parsing (T-2038).
