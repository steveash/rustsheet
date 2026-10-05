---
id: T-0107
title: "Number→text conversion with Excel's 15-significant-digit rule"
milestone: M1
lane: engine
status: todo
priority: P1
size: S
tier: opus
depends_on: [T-0009]
---

## Goal
`=CONCAT(0.1+0.2)` and `&` concatenation of numbers must produce Excel's text.

## Context
One shared number-to-text routine must be used by every coercion site (concatenation, text functions, criteria).

## Acceptance criteria
- [ ] gen-m2 number-to-text cells leave eval/baseline/gen-m2.txt (T-0203 triage): 78bc88199fc96650 A154; 8eb18149d4245e1a A102; a339ea84ef64382e A32; b4392a6514a36ade A336 (`7+1E15` → "1000000000000010"); d0c29dd5c530e28c A34 (Calc sheet).
- [ ] probes A9 passes; all coercion sites use one routine (reviewer-quality checks).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
