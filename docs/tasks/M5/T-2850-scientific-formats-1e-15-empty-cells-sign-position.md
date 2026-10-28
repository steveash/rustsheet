---
id: T-2850
title: "Scientific formats: 1E+15 empty cells, sign position before literal, #E+0 ties"
milestone: M5
lane: functions-text
status: in-progress
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
- [ ] The remaining gen-m10 scientific text cells pass, no regressions; number-formats.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
