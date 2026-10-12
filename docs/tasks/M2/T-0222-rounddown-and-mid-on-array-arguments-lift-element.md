---
id: T-0222
title: "ROUNDDOWN and MID on array arguments: lift element-wise instead of #N/IMPL! (2 gen-m2 known failures)"
milestone: M2
lane: functions-math
status: done
priority: P3
size: S
tier: sonnet
depends_on: [T-0270]
---

## Goal
ROUNDDOWN and MID accept array arguments (e.g. a row from `INDEX(range,1,0)`) and lift element-wise like
Excel, instead of returning `#N/IMPL!`.

## Context
Lane: `functions-math` (decision 0022): ROUNDDOWN is the main module; the MID half touches `text/common.rs` through the shared array-lift helper.

gen-m2 (T-0203) evidence (excel-graph):
- `gen-m2/893302d6c5b99e2e` Calc!A313 `=ROUNDDOWN((Data!D5*INDEX(Data!C7:D11,1,0)),((Data!B8/Data!D6)/(0/Data!A$1)))`:
  expected `#DIV/0!` (the num_digits error), actual `#N/IMPL!`.
- `gen-m2/d0c29dd5c530e28c` Calc!A246 `=CONCAT(MID(INDEX(Data!B3:E8,0.1,0),7,0))`: expected "", actual `#N/IMPL!`.
Find where `#N/IMPL!` comes from (`rg "N/IMPL|Error::NIMPL"`) and reuse the existing array-lifting helper
used by other scalar functions. Do not write a new one.

Redo on current `main` (2026-10-07): the first attempt (#129, branch
`agent/functions-math/T-0222-rounddown-mid-array-lift`, closed) added a second lifting primitive
(`array_lift.rs`). Main now has `functions/text/text_arg.rs` (T-0257, T-1022), so reuse or generalize
that for ROUND/ROUNDUP/ROUNDDOWN, and keep that branch's oracle-filled probe
`probes/round-concat-array-lift`. Its CONCAT half overlaps T-0243 (#203, merged), so check what is
still failing there. Lifting ROUND regresses `gen-m2/caa981b0753f455c` Calc!A26 until HLOOKUP lifts
over an array lookup_value, which is why this depends on T-0270.

## Acceptance criteria
- [ ] The 2 cells above disappear from eval/baseline/gen-m2.txt with no new F lines. Unit tests for an array first argument.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other functions that still return `#N/IMPL!` on arrays (file follow-ups).

## Outcome
- ROUND/ROUNDUP/ROUNDDOWN/TRUNC now go through the shared `lift_numbers` helper (as MOD does), so array arguments lift element-wise and an error argument next to an array broadcasts. MID already lifted on main (T-0257).
- The remaining `#N/IMPL!` for A246 was CONCAT with an array argument: it now joins the elements row by row.
- Eval delta: gen-m2 known failures 28 -> 26 (Calc!A313 of 893302d6c5b99e2e and Calc!A246 of d0c29dd5c530e28c). No regressions.
