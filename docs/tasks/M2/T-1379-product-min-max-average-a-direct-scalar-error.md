---
id: T-1379
title: "PRODUCT/MIN/MAX/AVERAGE: a direct scalar error beats a referenced error; SUM: scalar direct error beats array error and array vs reference is leftmost"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
PRODUCT, MIN, MAX, AVERAGE and SUM match Excel's error precedence between direct scalar errors, array errors and referenced errors.

## Context
Follow-up of T-1334. Failing cells in probes/aggregate-error-precedence: C1, C5, C7, C9, C13, C15, C17, C21, C23, C25, C29, C31 (a referenced error before a direct scalar error; Excel returns the direct one, we return the referenced one) and SUM C50, C53, C61 (scalar direct error beats an array error; array error vs referenced error is leftmost). Rule: docs/excel-semantics/syntax.md, "Aggregate error precedence". Start from how SUM was done in T-1315.

## Acceptance criteria
- [ ] All cells of probes/aggregate-error-precedence pass and the baseline shrinks.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
