---
id: T-0281
title: "Probe signs, e-notation and zero inside parenthesised number text"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for sign, e-notation, currency-order and zero forms inside parenthesised number text.

## Context
T-0279 review: the engine returns +1 for `($-1)` and `($ -1)` (currency accepts a sign, parentheses negate again) and -0.0 for `(0)`; Excel's answers are unprobed. Probe `($-1)`, `(-$1)`, `(1e3)`, `(1E-3)`, `(1$)`, `(0)`, `(+1)` as text coercion; record in coercion.md; file an engine task if the engine differs.

## Acceptance criteria
- [ ] A probe case holds oracle values for each form and coercion.md records them.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
