---
id: T-2812
title: "Fit the format-code complexity cost of a currency tag to the probed limits"
milestone: M10
lane: functions-text
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
TEXT rejects a format code holding a currency tag (`[$€]`, `[$€-40C]`) exactly where Excel does, for the quoted-text and space ladders.

## Context
Found by T-1883's probes/t1883-currency-locale-edges (excel-graph). Excel: `[$€-40C]` + 117 quoted characters works, 118 fails (A26, A27); `[$€-40C]` + 77 spaces works, 78 fails (A49, A50); `[$€]` + 117 quoted works, 118 fails (A63, A60). The engine rejects the first (A26) and accepts `[$€]` + 118 and + 121 (A60, A61). Cost model: formatter/complexity.rs (`Token::Currency`, `LOCALE_COST`); numbers in docs/excel-semantics/number-formats.md "TEXT format code complexity limit". Fit tier is opus (numeric fitting); probe finer boundaries (spaces ladder for `[$€]`, a currency tag with a longer symbol) if needed.

## Acceptance criteria
- [ ] probes/t1883-currency-locale-edges A26, A60, A61 pass and leave the baseline; no passing cell regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
