---
id: T-1573
title: "MAXA, MINA and AVERAGEA rank a scalar direct error above a referenced one, leftmost referenced error first"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
MAXA, MINA and AVERAGEA return a scalar direct error before a referenced error, and the leftmost referenced error otherwise, like MAX/MIN/AVERAGE.

## Context
probes/aggregate-picked-direct-a-variants D20-D27 (ours: first error at once, `MAXA(A5,#DIV/0!)` is #N/A instead of #DIV/0!). `for_each_value_a`; SPEC-0009 row AVERAGEA, MAXA, MINA.

## Acceptance criteria
- [ ] The probe cells named in Context pass (`rustsheet-eval show`), no new known failures; the SPEC-0009 row is updated from DIVERGES to matched.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
