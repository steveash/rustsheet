---
id: T-1056
title: "Probe error-argument vs range-check order in REPLACE, MID and FIND"
milestone: M1
lane: research
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Probes record whether Excel lets a range failure (start<1, negative length, not found) beat a later error argument, and the engine matches.

## Context
Follow-up of T-1022 (reviewer note). `broadcast_text` now returns an error argument only if it is the first failure. `replace_element` (functions/text/string_format.rs) range-checks `start_num`/`num_chars` before coercing later arguments, so `REPLACE("abc",0,NA(),"x")` and `REPLACE("abc",1,-1,NA())` now give #VALUE! with no oracle evidence. Cells to probe: those two, `MID("abc",0,NA())`, `FIND("z","abc",NA())`. See docs/excel-semantics/arrays.md.

## Acceptance criteria
- [ ] Each cell is recorded with its Excel value in arrays.md; the engine matches or the gap is a filed task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
