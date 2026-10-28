---
id: T-3011
title: "Probe CELL(color|parentheses|prefix|protect|width) on excel-graph: the engine's #VALUE! there is an unevidenced placeholder"
milestone: M5
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel for the web's value of `CELL(info_type, ref)` for info_type "color", "parentheses", "prefix", "protect" and
"width" is recorded as oracle evidence in `eval/`, so the engine's answer there is evidenced rather than guessed.

## Context
`cell_info` (crates/rustsheet-engine/src/functions/information/mod.rs) returns `#VALUE!` with the message
"info_type not implemented" for "COLOR", "FORMAT", "PARENTHESES", "PREFIX", "PROTECT" and "WIDTH". Only "format"
(docs/excel-semantics/coercion.md, T-0280) and "filename" (information.md, T-2052) are evidenced as `#VALUE!` on
excel-graph; the other five are unprobed. The desktop (Mac) golden calipers/roundtrip/formulas_information Sheet1
E140:N160 gives color 0, parentheses 0, prefix "", protect 1, width 15 on General cells (owned by T-2663 as desktop
cells; gap triage 2026-10-28). If Excel for the web gives values here, those 105 calipers cells are a web-fixable
semantics gap rather than desktop-only, and an implementation task in functions-logic follows.

## Acceptance criteria
- [ ] A new probes case `info-cell-format-info-types` (excel-probe skill, oracle excel-graph, canary-verified)
      covers, for each of "color", "parentheses", "prefix", "protect", "width" and, as a control, "format": a blank
      cell, a number, a text, a text typed with a leading apostrophe if the spec format can express it, a cell
      formatted `0.00;[Red]-0.00`, a cell formatted `(0)` style negative-in-parentheses, a reference on another
      sheet, a multi-cell range and the info_type in upper and mixed case; plus `CELL("width")` on a column given
      an explicit width if the spec can set one.
- [ ] The results are recorded in docs/excel-semantics/information.md (VERIFIED for excel-graph, citing cells).
- [ ] If any result differs from the engine, a functions-logic task is filed naming the differing cells and the
      formulas_information cells it would fix (and T-2663's Context is updated to say which of its CELL cells
      are web-fixable); otherwise the probe passes and the outcome says the placeholder matches.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing any CELL info type (the follow-up task does that).
- Desktop behaviour (T-2663, M14).
