---
id: T-1814
title: "Lifted text calls stop cloning delimiter lists and building whole arrays per element"
milestone: M7
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A lifted TEXTSPLIT, TEXTBEFORE, TEXTAFTER or REGEX* call does not copy its delimiter list or its argument values for
every element, and an element that keeps only its first value does not build the rest of its array.

## Context
T-1272's performance and security reviews (MINOR): `text_around_element` (functions/text/before_after.rs) builds a
`Vec<&str>` of the delimiters and clones `values[0]` per element; TEXTSPLIT and REGEXEXTRACT build their whole array for
a lifted element when `lift_text_call` (functions/text/lifted.rs) keeps only [0][0]. The total work is already bounded
by `text_call_cost`, so this is speed only. No result may change: `make eval-check` and the text unit tests must stay
identical.

## Acceptance criteria
- [ ] The delimiter list is borrowed once per call, not rebuilt per element.
- [ ] A lifted TEXTSPLIT/REGEXEXTRACT element computes only its first value.
- [ ] `make check` green with no eval change; reviewers approve.

## Out of scope
- Moving the text functions onto `lift_call` (T-1775).
