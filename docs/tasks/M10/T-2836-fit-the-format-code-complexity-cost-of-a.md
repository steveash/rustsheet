---
id: T-2836
title: "Fit the format-code complexity cost of a long $ run after a locale or currency tag"
milestone: M10
lane: functions-text
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
TEXT rejects a format code ending in a long run of unquoted `$` after a locale or currency tag exactly where Excel does.

## Context
Found by T-2812. probes/t2812-currency-tag-cost (excel-graph, 2026-10-26) ladders `$` (the engine's `DOLLAR_COST`, 1
each) after a tag. Excel: `[$€-40C]` and `[$€]` + 230 `$` work and 231 fail; `[$USD]` + 226 work and 227 fail; `[$-409]`
+ 233 already fails (the smallest probed). The engine's model is fitted on quoted and space ladders, where `[$-409]`
costs 14 and a currency tag is quoted text of its symbol plus 13 (and +4 for the unquoted text after it), and it
accepts each of these one `$` too long, or more for `[$-409]`. No additive `$` cost fits both: `[$-409]` needs
f(233) >= 237, while the currency tags need f(230) <= 231. The only `$` evidence before this is `"d"&REPT("$",160)`
works and `REPT("$",250)` fails (probes/text-format-length-limit-d, -e), so probe a bare `$` ladder (`REPT("$",n)` and
`"d"&REPT("$",n)` near the limit) and `[$-409]` + 225..232 `$` first (excel-probe skill). Add two inferences of T-2812 to the batch: `[$USD]` + 70..80
spaces (the +4 after a multi-character symbol) and `[$€-F800]` + quoted near 117 (a system-locale tag's locale priced at 0).
Cells (Excel #VALUE!, ours the text):
- probes/t2812-currency-tag-cost S!A3 `[$€-40C]` + 231 `$`, S!A11 `[$€]` + 231 `$`, S!A31 `[$USD]` + 227 `$`.
- probes/t2812-currency-tag-cost S!A17, S!A18, S!A19, S!A20: `[$-409]` + 233 to 236 `$`.
Semantics: number-formats.md "TEXT format code complexity limit" (currency tags). Code: formatter/complexity.rs.

## Acceptance criteria
- [ ] Visible parity: the 7 cells above leave eval/baseline/probes.txt (`make eval-baseline`) with no new known-failure
      lines in any baseline.
- [ ] number-formats.md states the `$` cost found.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Visible-parity stage
Decision 0083: every cell this task owns has the wrong type (Excel #VALUE!, ours text), so the task is in the campaign:
done when each owned cell has Excel's type and value.
