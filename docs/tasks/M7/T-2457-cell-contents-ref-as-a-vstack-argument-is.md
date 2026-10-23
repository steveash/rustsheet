---
id: T-2457
title: "CELL('contents', ref) as a VSTACK argument is #VALUE!"
milestone: M7
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
CELL("contents", ref) passed to VSTACK makes the VSTACK one #VALUE!, as Excel does.

## Context
Evidence: eval/suites/probes/t2414-gen-m7-unexplained (A41, A61, A81) and docs/excel-semantics/information.md "CELL(\"contents\", ref) as a VSTACK argument". Excel: `VSTACK(CELL("contents",Data!C11),1)` is `#VALUE!`; `CELL` alone and `VSTACK(0,..)` are fine. HSTACK and other callers are unprobed (probe them first if the fix needs the boundary).

Owned gen-m7 cells (T-2414):
- 8c4570978f18f4c2 A782 `=VSTACK(CELL("contents",Data!$C11),Data!B7:E12)`: Excel #VALUE!, ours 0.0; cells (28): A782:D788

## Acceptance criteria
- [ ] probes/t2414-gen-m7-unexplained cells A41, A61, A81 (and their spill cells) pass; known-failure lines leave eval/baseline/probes.txt.
- [ ] Visible parity: gen-m7/8c4570978f18f4c2 A782:D788 (28 cells, `=VSTACK(CELL("contents",Data!$C11),Data!B7:E12)`) pass; those lines leave eval/baseline/gen-m7.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Arguments the probe did not cover.

## Visible-parity stage
Decision 0083: every cell this task owns has the wrong type, error or value, so the task is in the campaign; done when they have Excel's.
