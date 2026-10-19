---
id: T-2001
title: "Match IM* text formatting at large exponents and small parts (T-1827 A11 A40-A42)"
milestone: M9
lane: functions-engineering
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
IM* text keeps 15 digits past exponent 98 and writes small parts in full past 20 characters, as Excel does.

## Context
Known failures probes/eng-im-overflow S!A11, A17, A40, A41 (14-digit cut) and A42 (E notation, sin digits). `Complex::fmt` goes through `number_to_text`. See engineering.md "IM* results that overflow".

## Acceptance criteria
- [ ] A11, A17, A40, A41 pass; A42's formatting matches and its sin/cos digits are probed or split off; no finite result that passes changes.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
