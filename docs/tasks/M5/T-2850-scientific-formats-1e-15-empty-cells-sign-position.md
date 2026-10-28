---
id: T-2850
title: "Scientific formats: 1E+15 empty cells, sign position before literal, #E+0 ties"
milestone: M5
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Scientific formats show the 1E+15 / 1.23457E+14 cells and place the minus sign before a literal as Excel does.

## Context
Follow-up of T-2227 (gen-m10): the 1E+15 / 1.23457E+14 cells that show nothing, and "-x1E+15" vs "x-1E+15". Find the cells with `make eval-show` on the gen-m10 known failures. `#E+0` ties (2.5 -> "3E+0") look like T-2206. Code: `crates/rustsheet-engine/src/formatter/format.rs`.

## Acceptance criteria
- [x] The remaining gen-m10 scientific text cells pass, no regressions; number-formats.md updated.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
The only remaining gen-m10 scientific text cell (bfbcd16c F!D19, `"x"General` on -1E15) was the General minus-position bug; fixed together with T-2097 in format.rs (minus leads the literals, General prints the absolute value). No empty 1E+15 cells remain in the report; `#E+0` ties are not in the gen-m10 failures (left to T-2206).
