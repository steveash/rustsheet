---
id: T-2171
title: "INT cuts to 15 significant digits first, as TRUNC does (INT(1-2^-53) = 1)"
milestone: M5
lane: functions-math
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
INT rounds down after cutting the value to 15 significant digits, as TRUNC and ROUNDDOWN do, so `INT(1-2^-53)` is 1.

## Context
Gap triage of gen-m5 (T-2114, excel-graph oracle, 2026-10-20). `fn_int` (mathematical.rs) is `f64::floor`. Excel cuts to 15 significant digits first (docs/excel-semantics/numbers.md "INT cuts to
15 significant digits first", added by this triage; the cut itself is ROUND's stage 1, `round_decimal` in
crates/rustsheet-engine/src/decimal_round.rs, used by TRUNC). Reuse that helper; do not write a new one. Check FLOOR.PRECISE,
which passes once INT is right (38b1b8e5 A212).

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 5 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/0362d48f56108faa Calc!A364 | `=INT(1-2^-53)` | 1.0 | 0.0 |
| gen-m5/043ee562f32eb901 Calc!A62 | `=INT(1-2^-53)` | 1.0 | 0.0 |
| gen-m5/1962de140dae0111 Calc!A352 | `=INT(1-2^-53)` | 1.0 | 0.0 |
| gen-m5/38b1b8e51dbad326 Calc!A212 | `=FLOOR.PRECISE(INT(1-2^-53))` | 1.0 | 0.0 |
| gen-m5/b0d597ec5af5fb92 Calc!A173 | `=INT(((1.005+Data!E11)+(1000000000000000.0-Data!A8)))` | 999999876543312.0 | 999999876543311.0 |

## Acceptance criteria
- [ ] These 5 known-failure lines disappear from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] A unit test for `INT(-(1-2^-53))` documents the engine's choice (unprobed; follow the same 15-digit cut).
- [ ] `make check` green; reviewers approve.

## Out of scope
- ODD/EVEN (T-2173), other rounding functions.
