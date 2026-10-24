---
id: T-2305
title: "Probe REPT/CODE/UNICODE/T over 255 units from a cell, an array element, and with an error count"
milestone: M5
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for the three inputs docs/excel-semantics/text.md lists as unprobed under "REPT, CODE, UNICODE and T refuse a text argument over 255 UTF-16 units".

## Context
T-2155 implemented the 255-unit cut from probes/t2199-text-length-and-combin-cuts, where the long text is always a
formula result. Unprobed (fidelity review of #1430): long text read from a cell (`REPT(A1,2)` with A1 holding 256
characters, also CODE/UNICODE/T of it), long text as an array element (`REPT({"<256 x>"},1)`, `CODE`, `UNICODE`,
`T({"<256 x>"})`; ours cuts REPT/CODE/UNICODE elements and keeps T's), and which error wins when REPT's count is also
an error (`REPT(REPT("x",256),#N/A)`; ours gives the text's #VALUE!). Use the `excel-probe` skill.
Code: `reject_over_short_limit` (functions/text/common.rs).

## Acceptance criteria
- [ ] A probe case under eval/suites/probes with oracle values for the inputs above.
- [ ] text.md entry updated with the results; a fix task filed for any cell ours gets wrong.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Three probe cases (text-255-units-from-cell, -array-element, -rept-error-count; 26 cells, excel-graph) filled and recorded in text.md. Ours differs on 5 cells (T of a long cell is "", T of a long array element is #VALUE!, REPT's count error wins); fix task T-2627 filed and the 5 lines are in eval/baseline/probes.txt as known failures.
