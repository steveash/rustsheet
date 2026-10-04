---
id: T-0106
title: "ROUND/ROUNDUP/ROUNDDOWN decimal-correct rounding per Excel"
milestone: M1
lane: engine
status: todo
priority: P1
size: S
depends_on: []
---

## Goal
Match Excel for binary-inexact halves (ROUND(1.005,2), ROUND(2.675,2)).

## Context
Hypothesis in docs/excel-semantics/numbers.md; probe cases pending oracle.

## Acceptance criteria
- [ ] probes/precision-and-rounding A6–A8 pass bit-exact; no regressions; semantics entry VERIFIED with implementation link.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
