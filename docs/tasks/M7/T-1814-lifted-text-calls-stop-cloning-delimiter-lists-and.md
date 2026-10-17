---
id: T-1814
title: "Lifted text calls stop cloning delimiter lists and building whole arrays per element"
milestone: M7
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-1272]
---

## Goal
A lifted TEXTSPLIT, TEXTBEFORE, TEXTAFTER or REGEX* call does not copy its delimiter list or its argument values for
every element, and an element that keeps only its first value does not build the rest of its array.

## Context
T-1272's performance and security reviews (MINOR): per-element copies left after T-1272 itself fixes the unbounded parts
(its Handoff items 3 and 4: folding/cloning the delimiter lists per element and building a lifted TEXTSPLIT element's
whole array are directive-5 work and land in T-1272). What remains here is speed only, for example the per-element
`Vec<&str>` of the delimiters and the `values[n]` clones in `text_around_element` (functions/text/before_after.rs) and
`regex_operands`. No result may change: `make eval-check` and the text unit tests must stay identical.

## Acceptance criteria
- [ ] The delimiter list is borrowed once per call, not rebuilt per element.
- [ ] A lifted TEXTSPLIT/REGEXEXTRACT element computes only its first value.
- [ ] `make check` green with no eval change; reviewers approve.

## Out of scope
- Moving the text functions onto `lift_call` (T-1775).
