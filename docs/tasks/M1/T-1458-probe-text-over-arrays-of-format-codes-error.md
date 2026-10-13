---
id: T-1458
title: "Probe TEXT over arrays of format codes: error precedence, broadcast shape, per-format complexity (follow-up of T-1451)"
milestone: M1
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for TEXT over an array or range of format codes, replacing the INFERRED notes in docs/excel-semantics/number-formats.md and text.md.

## Context
T-1451 implemented the lift in `fn_text` (functions/text/common.rs) with hand-written expectations (fn_text_arrays.rs). Probe with excel-probe: `TEXT(NA(),{"0","0"})`, `TEXT({1,#N/A},{"0",#DIV/0!})`, `TEXT({1,2},{"0.0","00"})`, `TEXT({1;2},{"0","00"})` (2x2), mismatched sizes (#N/A padding), a range of formats, and `TEXT(1,{"0",<too-complex format>})`.

## Acceptance criteria
- [ ] A probe case under eval/suites/probes covers each expression above with oracle values; implementation and the INFERRED notes are corrected to match.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Added probes/text-format-array (37 scored cells, excel-graph oracle). The engine already matched Excel on every scored cell (rustsheet-eval, 37 of 37), so the INFERRED notes in text.md and number-formats.md are now VERIFIED. No engine change; eval known failures unchanged.
