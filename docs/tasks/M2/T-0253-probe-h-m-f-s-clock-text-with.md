---
id: T-0253
title: "Probe H:M.f:S clock text with more than 12 fraction digits"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's result for `H:M.f:S` clock text whose fraction has more than 12 significant digits is known, and the engine matches it or records a DEVIATION.

## Context
From T-0153 (PR #175, fidelity review MINOR). `time_text::fraction_as_seconds` (crates/rustsheet-engine/src/functions/date_and_time/time_text.rs) returns None past 12 significant digits: a bound that keeps the millisecond total exact in an f64, not Excel evidence. docs/excel-semantics/dates.md "Time text" marks it Unprobed. Probe e.g. `0+"10:11.1234567890123:12"`, `0+"0:1.123456789012:0"`, `0+"0:1.0000000000001:0"` with TIMEVALUE and VALUE (excel-probe skill). Note the at-most-one-out-of-range rule: with minutes below 60, whole seconds may be huge.

## Acceptance criteria
- [ ] Probe case oracle-filled; dates.md updated with the result.
- [ ] The engine matches every probe cell, or a DEVIATION entry (decision 0014) explains the bound.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
