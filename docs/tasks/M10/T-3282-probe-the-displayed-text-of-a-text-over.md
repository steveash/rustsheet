---
id: T-3282
title: "Probe the displayed text of a text over 255 characters under a text section"
milestone: M10
lane: functions-text
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A cell's displayed text for a text over 255 characters under a text-section format (`"pre "@`, `_(@_)`) matches Excel, from oracle evidence.

## Context
T-1736 routes a text or boolean cell through the format's text section. TEXT() is `#VALUE!` past 255 characters, and `formatter::format::displayed_text` keeps the plain text then; that is a guess, not Excel evidence (number-formats.md "Displayed text"). Use the `excel-probe` skill.

## Acceptance criteria
- [x] A probe records Excel's displayed text for 256- and 300-character texts under `"pre "@`, `@`, `_(@_)` and `;;;`.
- [x] The probe also covers a 250-character text whose formatted result passes 255, and a code with an unparsable number section, on a text cell.
- [x] `displayed_text` matches it, or a `DEVIATION` records why not.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probes `probes/displayed-text-long-text` (24 cells) and `-long-text-b` (11 cells), excel-graph: past 255
characters a text section that changes the text displays 255 `#`; no text section or a section that is exactly
`@` shows the text whole; `;;;` shows nothing. `displayed_text` now matches (35/35 cells pass). Codes TEXT()
rejects as invalid make excel-graph refuse the whole workbook (`FileCorruptTryRepair`), so that criterion is
recorded as unprobeable and the engine keeps the plain text. number-formats.md "Displayed text over an
edge-value pool" updated. Eval delta: +35 passing probe cells (new cases), no regressions.
