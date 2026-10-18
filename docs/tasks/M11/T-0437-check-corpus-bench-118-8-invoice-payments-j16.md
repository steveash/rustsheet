---
id: T-0437
title: "Check corpus-bench/118-8 Invoice Payments J16:J23 statuses against the pinned clock"
milestone: M11
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Establish whether the engine's statuses for corpus-bench/118-8 `Invoice Payments`!J16:J23 are right for the pinned clock.

## Context
T-0434 left it unassessed: the engine counts the six invoices as CURRENT although their due dates are in 2023 (`TD` = TODAY()), while Excel (run 2026-10-17) reports 6 PAST DUE. The cells are skipped as volatile now, so scoring is unaffected, but it may hide an engine bug (e.g. a table-reference or date-compare problem). Run with the case's clock and compare J16:K23 with the Excel cached values.

## Acceptance criteria
- [ ] Cause found and recorded in the Outcome; an engine fix task filed if it is an engine bug, or the cells explained as clock-only.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
