---
id: T-3314
title: "Probe on desktop Excel the displayed text of a cell whose format TEXT() rejects as invalid"
milestone: M14
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The displayed text of a text or boolean cell whose number format TEXT() rejects as invalid (`42abc`, `@;0`, `x@0`) is known from desktop Excel evidence, and `formatter::format::displayed_text` matches it or a `DEVIATION` says why not.

## Context
T-3282 found that excel-graph refuses a workbook that carries such a code (`FileCorruptTryRepair`), so `displayed_text` keeps the plain text as a guess (number-formats.md "Displayed text over an edge-value pool"). Only desktop Excel can answer this (decision 0039, M14 Phase 4). Also check whether desktop Excel repairs the code on load, and what it shows for a whitespace-only text section (`@ `) on a text over 255 characters.

## Acceptance criteria
- [ ] A desktop probe records the displayed text of "abc", TRUE and 5 under each of `42abc`, `@;0` and `x@0`, and of a 256-character text under `@ `.
- [ ] number-formats.md records the result, and the engine matches it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing the probe results beyond `displayed_text`.
