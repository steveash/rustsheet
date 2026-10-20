---
id: T-2091
title: "Trig cases in the math_lift bench: SUM(SIN(range)) and SUM(TAN(range))"
milestone: M5
lane: perf
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The tracked `math_lift` bench times SIN and TAN, so a regression in the x87-reduced trig (`functions/math_and_trigonometry/trig.rs`) shows up in the bench suite, not only in ad-hoc microbenchmarks.

## Context
T-1637 made SIN/COS/SEC/CSC about 3.7x and TAN/COT about 2.6x faster with a fast sum plus Ziv's rounding test, measured with an uncommitted microbenchmark (T-1637 Outcome; performance review asked for a tracked case). `crates/rustsheet/benches/math_lift.rs` already has the EXP pattern to copy: a per-row case (`=SIN(A{r})`, 100,000 rows) and a lifted case (`=SUM(SIN(A1:A1000000))`), and the same for TAN. Column A holds `row / 7`, which spreads angles over [0, 142857] and hits every quadrant.

## Acceptance criteria
- [ ] `math_lift` has per-row and lifted SIN and TAN cases, listed in its module doc.
- [ ] Their timings on the current main are in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing trig.rs.
