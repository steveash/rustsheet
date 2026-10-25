---
id: T-0066
title: "Parity page reads each function's within-15-digit share from report.json v15_checkpoint instead of recomputing it"
milestone: M0
lane: harness
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The parity page takes each function's within-15-digit share and Phase 1 standing from report.json's
`v15_checkpoint` (crates/rustsheet-eval/src/v15_checkpoint.rs), so the measure has one definition.

## Context
T-0065 review (reviewer-architecture, MINOR): `FunctionParity.within15_share` in
tools/src/rustsheet_tools/parity_report.py recomputes `cells - visible` beside
`V15Function::rate` and `meets_rate` in Rust, and `PHASE1_SHARE` repeats the 0.999 threshold of
`milestone_exit::RATE_NUM`/`RATE_DEN`. Decision 0114.

## Acceptance criteria
- [ ] parity_report.py reads `v15_checkpoint.functions[].cells`, `visible` and `meets_rate`
      instead of computing the share and the threshold itself; a report without them fails loudly.
- [ ] tools/tests/test_parity_report.py covers it; `make check` green; reviewers approve.

## Out of scope
- Changing the measure.
