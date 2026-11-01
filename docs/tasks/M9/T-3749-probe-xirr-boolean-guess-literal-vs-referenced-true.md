---
id: T-3749
title: "Probe XIRR boolean guess: literal vs referenced TRUE/FALSE, equal vs different counts, and formatted numeric text in ranges"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Excel's XIRR is probed for a boolean guess (literal and cell-held TRUE/FALSE, beside equal and different value/date counts) and for formatted numeric text (dates, currency, percent) in value and date ranges, and our behavior follows the evidence.

## Context
T-3267 made a boolean guess beside different counts `#VALUE!` on one cell (gen-m9/b6a02893a8d6b151 A226, a guess cell holding TRUE) and made XIRR read range text through `cast_number`. Literal TRUE, equal counts and formatted text are unprobed (financial.md XIRR order paragraph). Code: `fn_xirr` and `array_of_numbers_from` in crates/rustsheet-engine/src/functions/financial/mod.rs. Use the `excel-probe` skill.

## Acceptance criteria
- [ ] A probe case with oracle values covers each unprobed form above; financial.md states the result; any mismatch is fixed or baselined.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
