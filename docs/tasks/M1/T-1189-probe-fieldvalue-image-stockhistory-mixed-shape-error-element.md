---
id: T-1189
title: "Probe FIELDVALUE/IMAGE/STOCKHISTORY mixed-shape, error-element and URL-shape arguments"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---


## Goal
Excel's results for the unprobed argument shapes of IMAGE, STOCKHISTORY and FIELDVALUE are established by an oracle probe and the engine matches them.

## Context
T-1179 guessed four behaviors (see information.md, "IMAGE and STOCKHISTORY never fetch", UNVERIFIED list): FIELDVALUE with two array arguments of different shapes, an error element in an array `field_name`, an array `source`/`stock` with an error in a later argument, an error element inside an array `source`/`stock`, `ERROR.TYPE` over a direct range, and which IMAGE sources count as a URL. Code: functions/information/offline_errors.rs.

## Acceptance criteria
- [x] A probe case (excel-probe skill) covers the four behaviors; the engine and information.md match the oracle values.
- [x] `make check` green; reviewers approve.

## Out of scope
Probing other arguments than those four.

## Outcome
New probe case probes/rich-errors-shapes-urls (51 cells, excel-graph). FIELDVALUE mixed shapes and
error elements in an array source/stock were as the engine guessed. Three things differed and are fixed in
offline_errors.rs:
- an error element of an array `field_name` stays itself;
- a later argument's error lifts over an array source/stock;
- the IMAGE URL rule is any-case `http:`/`https:` after leading spaces with something after the scheme.
49 of 51 cells pass. A16:B16 (IMAGE `sizing` 9 is `#VALUE!`) concern argument validation outside this
task's four behaviors and stay in the baseline for T-1345. Recorded in information.md.
