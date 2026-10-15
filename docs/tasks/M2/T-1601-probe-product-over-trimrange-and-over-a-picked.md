---
id: T-1601
title: "Probe PRODUCT over TRIMRANGE and over a picked error cell with a later direct error"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for PRODUCT over a TRIMRANGE pick and over a picked error cell followed by a direct error.

## Context
T-1571 made PRODUCT read picks like SUM (evaluate_aggregate_arg); these two cases follow SUM's rule without PRODUCT oracle values (docs/excel-semantics/syntax.md, UNVERIFIED). Use the excel-probe skill.

## Acceptance criteria
- [ ] A probe case with oracle values covers both; SPEC-0009 PRODUCT row and syntax.md updated, and ours fixed if it differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
