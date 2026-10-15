---
id: T-1306
title: "Probe and match CODE and UNICHAR edge code points: characters outside Windows-1252, surrogates and out-of-range values"
milestone: M5
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
CODE and UNICHAR match Excel on the code points their argument-mode probes never reached. Each
function gets at least 25 Excel cells, so it leaves the "below 25" list in report.md's "Excel
evidence per function" (decision 0031).

## Context
Evidence gap (eval/runs/latest/report.md, 2026-10-10): CODE has 4 Excel cells and UNICHAR 4, all
argument-mode cells (probes/text-argument-modes C19, C20, C26, F5; probes/decimal-unreadable A82,
A83; catalog availability). No Excel cell checks what they return for an unusual code point. Some
engine choices in crates/rustsheet-engine/src/functions/text/char_code.rs are unverified:
- `fn_code`: a first character outside Windows-1252 (`CODE("Ω")`, `CODE("あ")`, `CODE("😀")`,
  `CODE(UNICHAR(256))`) is `#VALUE!` (`char_to_win1252` returns None). Excel may return the code
  page's replacement `?` (63) instead. `CODE("")` is `#VALUE!`.
- `fn_unichar`: `f.floor() as u32` then `char::from_u32`, so surrogates (`UNICHAR(55296)`,
  `UNICHAR(57343)`) are `#VALUE!`, not `#N/A`. A noncharacter (`UNICHAR(65535)`) and
  `UNICHAR(1114111)` return a character. `UNICHAR(1114112)`, `UNICHAR(0)`, `UNICHAR(-1)` and
  `UNICHAR(1E+10)` are `#VALUE!`, and `UNICHAR(65.9)` is "A".
CHAR's undefined 1252 slots are already Excel-evidenced and pass (ironcalc-calc
TEXT/CHAR.xlsx B129, B141, B143, B144, B157 and B256 `#VALUE!`). Don't change `win1252_to_char`
unless the probe shows a CHAR mismatch.

## Acceptance criteria
- [ ] A probe case (excel-probe skill, excel-graph sidecar) with at least 25 CODE and UNICHAR cells
      covering the inputs above, plus `CODE(CHAR(128))`, `CODE(UNICHAR(8364))`, `CODE(" ")`,
      `UNICHAR(8364)`, `UNICHAR(128512)`, `LEN(UNICHAR(128512))` and `UNICODE(UNICHAR(65535))`.
- [ ] The engine matches every cell. The exception is a mismatch caused by UTF-16 position
      counting: add it to T-0249's context instead of fixing it here.
- [ ] docs/excel-semantics/text.md records the CODE and UNICHAR rules, citing the probe cells.
- [ ] report.md lists CODE and UNICHAR with at least 25 Excel cells; no new F lines in other suites.
- [ ] `make check` green; reviewers approve.

## Out of scope
- LEN/MID/UNICODE counting of astral characters (T-0249).
- ASC, DBCS, JIS (T-1248, T-1081).

## Outcome
Probes `probes/text-code-unichar-edges` (30 cells) and `probes/text-code-c1-controls` (22 cells), both excel-graph, cover every input in the task. One mismatch: `CODE` of a C1 control character (`CODE(UNICHAR(128..159))`) is the code point itself in Excel, we returned 63 for the slots 1252 fills with another character. Fixed in `char_to_win1252`. Everything else already matched. CODE and UNICHAR were already above 25 Excel cells in report.md (34 and 77) before these probes, now more. text.md "CLEAN and CODE scalar edges" records the rules.
