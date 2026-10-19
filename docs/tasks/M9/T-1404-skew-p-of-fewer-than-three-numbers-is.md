---
id: T-1404
title: "SKEW.P of two numbers is #DIV/0!, not 0 (probe the bound)"
milestone: M9
lane: functions-stat
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
SKEW.P of two numbers is `#DIV/0!`, as in Excel, and the bound (two or three points) follows a probe.

## Context
T-1289's gen-m9 suite (excel-graph sidecars, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/3b8e9078261b5bd2 Calc!A13 `=LAMBDA(v,acc,v)(SKEW.P(Data!E11:E11,Data!A11:A12,Data!A5:A7),LENB(TRIM("?b")))`: the
  numbers are -0.0001 and 1 (E11 blank, FALSE, TRUE and "a" skipped in the ranges). Excel `#DIV/0!`, engine 0
  (`SKEW.P(1,2)` and `SKEW.P(1,2,3)` are 0 in the engine).
Code: `fn_skew_p` in crates/rustsheet-engine/src/functions/statistical/count_and_average.rs. The bound (fewer than 3, as SKEW,
or fewer than 2) is evidenced only for two numbers; `SKEW.P(1,2,3)` (0 or `#DIV/0!`?) is unprobed: probe it, and a
constant series of three or more, with the excel-probe skill before choosing.
Semantics: docs/excel-semantics/statistics.md "SKEW.P of two numbers is #DIV/0!".

## Acceptance criteria
- [ ] gen-m9/3b8e9078261b5bd2 A13 passes in `./target/release/rustsheet-eval show gen-m9/<case>.case.json`, with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [ ] The SKEW.P probe cells pass; statistics.md's entry is VERIFIED.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
