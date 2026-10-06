---
id: T-0991
title: "SUMIF, SUMIFS, SUMX2PY2, SUMXMY2, DEVSQ, STDEV and VAR overflow is #NUM!"
milestone: M1
lane: functions-stat
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
SUMIF, SUMIFS, SUMX2PY2, SUMXMY2, DEVSQ, STDEV and VAR return #NUM! on overflow, as Excel does.

## Context
Rule VERIFIED in docs/excel-semantics/numbers.md (Aggregate overflow), evidence probes/aggregate-overflow (T-0192, excel-graph 2026-10-06). Engine gives inf or NaN (B and C cells only, the A cells already match): B37/C37 (SUMIF), B38/C38 (SUMIFS), B42/C42 (SUMX2PY2), B43/C43 (SUMXMY2), B44/C44 (DEVSQ), B45/C45 (STDEV), B46/C46 (VAR).

## Acceptance criteria
- [ ] probes/aggregate-overflow cells above pass; baseline shrinks.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
