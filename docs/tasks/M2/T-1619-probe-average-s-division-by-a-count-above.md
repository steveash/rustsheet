---
id: T-1619
title: "Probe AVERAGE's division by a count above 1024 for double rounding"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Establish with an oracle probe whether AVERAGE's division of the sum by a large count (n between 1025 and about 4096 cells) rounds twice (64-bit, then double), and fix the engine if it does.

## Context
Left over from T-1516 (docs/excel-semantics/numbers.md, "Division rounds twice"). A small count cannot discriminate; pick sums and counts by exact simulation so that single and double rounding of sum/n differ, using at most 15-digit operands.

## Acceptance criteria
- [ ] A probes-numeric case with oracle values for AVERAGE over a large count; the engine matches or a fix task is filed
- [ ] `make check` green; reviewers approve.

## Out of scope
-
