---
id: T-1198
title: "DECIMAL of unreadable text gives #NUM! in Excel, engine gives #VALUE!"
milestone: M5
lane: functions-math
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
DECIMAL of an argument it cannot read in the given radix (the failing input is the number 0.5, radix 13, gen-m5/98ed0b6354e3394a Calc!A9, Data!C12) returns `#NUM!` like Excel.

## Context
From T-1192's gen-m5 run: Excel gives `#NUM!` for `DECIMAL(0.5,13)` (hypothesis to probe: the number becomes the text "0.5" and "." is an invalid digit) where the engine gives `#VALUE!`. Re-check the exact case with `make eval-show` on gen-m5 and probe neighbours (empty string, invalid digit, over-long text, radix bounds) with the `excel-probe` skill; record in docs/excel-semantics/.

## Acceptance criteria
- [x] Probe covering the neighbouring inputs with oracle values.
- [x] DECIMAL matches every probed cell.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed 68 DECIMAL cells (probes/decimal-unreadable) and rewrote the text reader (`decimal_from_text`): `#NUM!` for unreadable text, leading blanks skipped, empty text 0, radix-16 `0x`/`x` prefix, float sum, 255-character limit. The rules are in docs/excel-semantics/engineering.md.
