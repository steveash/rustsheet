---
id: T-1135
title: "Probe XMATCH/XLOOKUP text and boolean lookups over blanks, and binary-search runs of equal values"
milestone: M1
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence settles three XMATCH/XLOOKUP questions that docs/excel-semantics/lookup.md lists as unprobed:
- text and boolean lookup values with match_mode -1/1 over blank and error cells, in linear and binary (2/-2) search modes;
- a blank lookup value against 0, "" and FALSE candidates;
- binary search (search_mode 2/-2) over runs of equal values, including a run that starts at the first cell.

## Context
T-1019 matched the binary search to Excel over blanks and errors. That work is the "XMATCH / XLOOKUP binary search" entry, `binary_search` in functions/binary_search.rs. Equal runs keep the earlier searches' answers: the run's first cell for match_mode 0/-1, never one below the second cell for match_mode 1, and the run's last cell when descending. Probe runs that start at the first cell, e.g. {40,40,40} and {40,40,40,40,40}, with match modes 0/-1/1 and search modes 2/-2. Linear modes live in `linear_scan` (functions/lookup_and_reference/xmatch.rs). Write a new probe case next to probes/xmatch-xlookup-blanks with the excel-probe skill and fill it with the oracle. If the oracle shows the engine is wrong, file the fix as its own task.

## Acceptance criteria
- [ ] A probe case with oracle values covers all three questions, and lookup.md records each finding (or says what still differs, with a fix task filed).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes; moving `linear_scan`/`MatchMode` to a neutral lookup module (a quality task if wanted).

## Outcome
Probe `probes/xmatch-text-bool-equal-runs` (134 cells) filled from excel-graph. Equal-run binary search and all Run cells match the engine. Six cells differ (blank vs text/boolean candidates, blank lookup value in XLOOKUP, descending boolean binary search); recorded in lookup.md, fix filed as T-1153. Eval: probes gains 134 cells, 6 new known failures baselined by the gate if required.
