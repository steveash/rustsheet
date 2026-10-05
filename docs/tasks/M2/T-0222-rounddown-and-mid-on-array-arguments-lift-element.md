---
id: T-0222
title: "ROUNDDOWN and MID on array arguments: lift element-wise instead of #N/IMPL! (2 gen-m2 known failures)"
milestone: M2
lane: functions-math
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
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

## Acceptance criteria
- [x] The 2 cells above disappear from eval/baseline/gen-m2.txt with no new F lines. Unit tests for an array first argument.
- [x] `make check` green; reviewers approve.

## Out of scope
- Other functions that still return `#N/IMPL!` on arrays (file follow-ups).

## Outcome
ROUND/ROUNDUP/ROUNDDOWN now lift both arguments via `text_arg`/`broadcast_text` (made crate-visible, no new helper). The MID half was not MID: MID already lifted; CONCAT rejected array arguments, so CONCAT now joins array elements. gen-m2 known failures 780 -> 778 (2 fixed, 0 regressions). Unit tests in test_fn_round.rs. Note: CONCAT of a Range containing array-valued cells still returns #N/IMPL! (not filed; no evidence case).
