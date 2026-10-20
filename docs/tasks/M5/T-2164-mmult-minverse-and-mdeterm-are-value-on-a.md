---
id: T-2164
title: "MMULT, MINVERSE and MDETERM are #VALUE! on a blank cell"
milestone: M5
lane: functions-math
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
MMULT, MINVERSE and MDETERM are `#VALUE!` on a single blank cell argument, as they already are on a range holding a blank.

## Context
Gap triage of gen-m5 (T-2114, excel-graph oracle, 2026-10-20). Rule: docs/excel-semantics/arrays.md "MDETERM / MINVERSE: a blank cell in the matrix is #VALUE!" (gen-m5 paragraph added by
this triage). A one-cell reference to a blank is read as a scalar 0 here (`MINVERSE(B4)` `#NUM!`, `MDETERM(D1)` 0,
`MMULT(100,C9)` 0). Code: `coerce_to_f64` in matrix.rs and `fn_mmult` (mmult.rs).

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 4 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/3e134e52393061b1 Calc!A10 | `=MMULT(((Data!B$4*100)+ABS(Data!C9)),Data!C$9)` | "#VALUE!" | 0.0 |
| gen-m5/cf893ec19f53a7fc Calc!A204 | `=MINVERSE(Data!$B4)` | "#VALUE!" | "#NUM!" |
| gen-m5/d9fb1e918971111b Calc!A213 | `=FLOOR.MATH(59,Data!C11,AVERAGE(Data!$C5,Data!B2:B5,MMULT(Data!A5,Data!B11)))` | "#VALUE!" | 59.0 |
| gen-m5/fc81c2f704bef138 Calc!A381 | `=TRUNC(SIGN(MDETERM(Data!D$1)),-4)` | "#VALUE!" | 0.0 |

## Acceptance criteria
- [ ] These 4 known-failure lines disappear from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Near-singular MINVERSE (T-2067).
