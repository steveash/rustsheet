---
id: T-0107
title: "Number→text conversion with Excel's 15-significant-digit rule"
milestone: M1
lane: engine
status: todo
priority: P1
size: S
depends_on: []
---

## Goal
`=CONCAT(0.1+0.2)` and `&` concatenation of numbers must produce Excel's text.

## Context
One shared number-to-text routine must be used by every coercion site (concatenation, text functions, criteria).

## Acceptance criteria
- [ ] probes A9 passes; all coercion sites use one routine (reviewer-quality checks).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
