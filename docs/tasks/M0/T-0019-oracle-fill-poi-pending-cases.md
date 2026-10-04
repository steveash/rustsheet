---
id: T-0019
title: "Oracle-fill the 237 pending POI cases (excel-graph, canary-verified)"
milestone: M0
lane: research
status: todo
priority: P0
size: M
tier: opus
depends_on: []
---

## Goal
Excel answers for the POI cases that have no usable cached values, so that suite measures parity
instead of reporting 237 of 336 cases as pending.

## Context
`make eval` 2026-10-04: poi 336 cases, 237 pending. Since T-0013 the oracle verifies every
workbook with a canary and makes Excel save Excel-authored files, so these can now be filled
safely. Expect refusals (SHEETS/INFO, protected or corrupt packages); those stay pending.

## Acceptance criteria
- [ ] `rustsheet-oracle fill -b excel-graph --suite poi` run; sidecars committed for every case the canary accepts; refusals listed with reasons in the PR.
- [ ] `make eval-baseline` records the new known failures (new coverage, 0 regressions); status.md table updated.
- [ ] New Excel quirks found are recorded in docs/excel-semantics/ (decision 0014); failing groups handed to the gap-triager.
- [ ] `make check` green; reviewers approve.

## Out of scope
Engine fixes for what the new evidence shows.
