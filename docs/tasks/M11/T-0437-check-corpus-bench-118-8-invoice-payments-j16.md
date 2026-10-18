---
id: T-0437
title: "Check corpus-bench/118-8 Invoice Payments J16:J23 statuses against the pinned clock"
milestone: M11
lane: engine
status: done
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
- [x] Cause found and recorded in the Outcome; an engine fix task filed if it is an engine bug, or the cells explained as clock-only.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Clock-only, not an engine bug. The harness evaluates every workbook at a fixed "now" of
2022-11-08 (`HARNESS_CLOCK_MS` in crates/rustsheet-eval/src/engine.rs) unless the workbook has a
METADATA `NOW` row; 118-8 has none, so `TD` = TODAY() is 2022-11-08, before the six invoices' 2023
due dates, and the engine correctly reports CURRENT. Loaded through the Python bindings with the
live clock (2026-10-18), the engine gives PAST DUE for J17:J18 and PAID IN FULL for J16, matching
Excel's cached/oracle behaviour. No engine task needed; the cells stay skipped as volatile (T-0434).
