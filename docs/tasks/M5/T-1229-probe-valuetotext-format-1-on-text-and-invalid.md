---
id: T-1229
title: "Probe VALUETOTEXT format 1 on text and invalid format values"
milestone: M5
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
VALUETOTEXT matches Excel for the format argument: format 1 on text, other format values, fractional formats and error values.

## Context
T-1212 accepts the format argument but ignores it (fn_valuetotext in functions/text/common.rs), because only `VALUETOTEXT(TRUE,1)` is probed (text-argument-modes C31). Microsoft documents format 1 as quoting text, unverified. See docs/excel-semantics/text.md "Text function argument modes". Use the excel-probe skill.

## Acceptance criteria
- [ ] Probe cases for VALUETOTEXT("ab",1), VALUETOTEXT("a""b",1), a blank cell and an error with format 1, VALUETOTEXT(A1:A3,{0,1}), format 2, 1.5, -1 and an error value with format 1 have oracle values, and the engine matches them.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/valuetotext-format` (37 cells, excel-graph). VALUETOTEXT now honours the format: 1 quotes text (embedded quotes doubled), the format must be in [0, 2) after number coercion (else `#VALUE!`), an error format propagates, and the format lifts over arrays (static_analysis no longer reads it as a scalar). Unit test `fn_valuetotext.rs`; documented in text.md. Eval: probes 37/37 on the new case, no regressions.
