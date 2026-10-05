---
id: T-0129
title: "Port IronCalc arithmetic error broadcasting over arrays (error_broadcast.xlsx)"
milestone: M1
lane: engine
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
Arithmetic operators and the functions that broadcast over arrays propagate errors element-wise
as Excel does, matching the upstream Excel-authored workbook `error_broadcast.xlsx`.

## Context
Upstream IronCalc `4deab8f6` ("Propagate arith errors properly (broadcast)"; found by the steward run
2026-10-05) reworks `base/src/arithmetic.rs` and the array paths in `date_and_time.rs`,
`math_and_trigonometry/mathematical.rs` and `text/common.rs`, adds `test_arrays_and_errors.rs`
(473 lines) and `xlsx/tests/calc_tests/INFORMATION/error_broadcast.xlsx`. Error propagation is M1
scope (docs/milestones.md), hence lane `engine` and P1. Use the T-0911 patch recipe (rewrite paths
`base/`→`crates/rustsheet-engine/`, `git apply --reject`) and resolve hunks by hand; `arithmetic.rs`
already carries RustSheet changes (POWER ranges from T-0920, MOD fmod from T-0912), keep them.
Split if the port exceeds size M.

## Acceptance criteria
- [ ] `4deab8f6` ported with `Ported-From: ironcalc@4deab8f6`, including its unit tests.
- [ ] `error_broadcast.xlsx` added to `crates/rustsheet-xlsx/tests/calc_tests`; its cells pass or are baselined as new coverage with reasons; 0 eval regressions.
- [ ] Error-precedence rules the workbook shows recorded in `docs/excel-semantics/`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- MAX/MIN (T-0223), ISREF (T-0940).
