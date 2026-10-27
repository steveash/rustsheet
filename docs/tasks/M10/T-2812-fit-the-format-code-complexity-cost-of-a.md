---
id: T-2812
title: "Fit the format-code complexity cost of a currency tag to the probed limits"
milestone: M10
lane: functions-text
status: done
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
- [x] probes/t1883-currency-locale-edges A26, A60, A61 pass and leave the baseline; no passing cell regresses.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
A currency tag now costs like quoted text of its symbol plus 13 (`CURRENCY_TAG_COST`), with its locale part free and
the quoted-text switch cost when unquoted text follows; the lexer tells a currency tag's symbol and locale apart
(`Lexer::at_currency_tag`, `has_pending_locale`). New probe probes/t2812-currency-tag-cost (54 cells, excel-graph) adds
`$` ladders, `[$€]` + spaces and `[$USD]`/`[$USD-40C]` + quoted; number-formats.md records the fit.
Eval delta: probes fixed 3 (t1883-currency-locale-edges A26, A60, A61), 7 new-coverage failures (t2812 `$` ladders:
A3, A11, A17:A20, A31), 0 regressions; visible parity 3496 (lock 3581). Follow-up: T-2836 owns the 7 `$` cells.

