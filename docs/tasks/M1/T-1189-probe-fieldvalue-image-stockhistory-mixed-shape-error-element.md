---
id: T-1189
title: "Probe FIELDVALUE/IMAGE/STOCKHISTORY mixed-shape, error-element and URL-shape arguments"
milestone: M1
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---


## Goal
Excel's results for the unprobed argument shapes of IMAGE, STOCKHISTORY and FIELDVALUE are established by an oracle probe and the engine matches them.

## Context
T-1179 guessed four behaviors (see information.md, "IMAGE and STOCKHISTORY never fetch", UNVERIFIED list): FIELDVALUE with two array arguments of different shapes, an error element in an array `field_name`, an array `source`/`stock` with an error in a later argument, and which IMAGE sources count as a URL. Code: functions/information/offline_errors.rs.

## Acceptance criteria
- [ ] A probe case (excel-probe skill) covers the four behaviors; the engine and information.md match the oracle values.
- [ ] `make check` green; reviewers approve.

## Out of scope
Probing other arguments than those four.
