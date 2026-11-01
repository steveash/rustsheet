---
id: T-2937
title: "Probe the file-name prefix of a workbook name for a quoted file name, an unnamed workbook and a definedName body"
milestone: M8
lane: research
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Know what Excel shows and stores for a workbook name read through a sheet in the three cases T-2834 left unprobed, and record it in docs/excel-semantics/references.md ("A sheet-qualified workbook name is rewritten to the workbook's file name").

## Context
T-2834 writes `=Feb!loc` as `=<file>.xlsx!loc`, quoting the file name by the sheet-name rule (`quote_name`), and
stores and saves it as `[0]!loc` (`expressions::utils::defined_name_text`). The evidence covers only hex file names
(excel-graph uploads under the input's sha256) and `[0]!loc` in a cell `<f>`. Unprobed: (1) a file name with a space,
`'` or other characters a sheet name would quote: needs a backend that keeps the file name (desktop Excel, M14 Phase 4,
decision 0039), or a Graph upload under a chosen name if the oracle client can do that; (2) an unnamed (never saved)
workbook, where ours keeps the typed `Feb!loc` until a re-parse shows `[0]!loc`; (3) `[0]!loc` inside a `<definedName>`
body on load (ours writes it there).

## Acceptance criteria
- [x] A probe case (or a desktop question in M14 Phase 4) per point above, with oracle values where a backend can answer.
- [x] references.md records each answer; any mismatch with ours gets an engine task.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (follow-up tasks if a probe shows a mismatch).

## Outcome
- (3) Probed on excel-graph: `probes/t2937-own-book-prefix-in-name-body` (12 cells, M8). `[0]!loc` inside a
  `<definedName>` body loads as the workbook name whatever the holding name's scope and whichever sheet reads it,
  even on a sheet with its own `loc`. Ours matches all 12 cells; no engine task needed.
- (1) and (2) cannot be answered on excel-graph (it uploads under hex file names and always saves): filed as
  desktop questions 21 and 22 in T-1075 (M14, Phase 4, decision 0039).
- references.md "A sheet-qualified workbook name is rewritten to the workbook's file name" records all three.
- Eval delta: +1 probe case, 12/12 cells pass; 0 regressions.
