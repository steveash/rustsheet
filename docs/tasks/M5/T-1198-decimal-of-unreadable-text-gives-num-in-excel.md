---
id: T-1198
title: "DECIMAL of unreadable text gives #NUM! in Excel, engine gives #VALUE!"
milestone: M5
lane: functions-math
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
DECIMAL of text it cannot read in the given radix returns `#NUM!` like Excel.

## Context
From T-1192's gen-m5 run: Excel gives `#NUM!` for a DECIMAL argument it cannot read where the engine gives `#VALUE!`. Re-check the exact case with `make eval-show` on gen-m5 and probe neighbours (empty string, invalid digit, over-long text, radix bounds) with the `excel-probe` skill; record in docs/excel-semantics/.

## Acceptance criteria
- [ ] Probe covering the neighbouring inputs with oracle values.
- [ ] DECIMAL matches every probed cell.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
