---
id: T-1023
title: "Probe WEEKNUM(0,21), ISOWEEKNUM(0) and WEEKNUM error precedence at serial 0 (return_type 0/3)"
milestone: M1
lane: research
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle values for WEEKNUM(0,21), ISOWEEKNUM(0), WEEKNUM(0,0), WEEKNUM(0,3) and WEEKNUM(0,99), and the engine matches them.

## Context
T-0995 made WEEKNUM accept serial 0 for return types 1, 2, 11-17 (`functions/date_and_time/weeknum.rs`). Serial 0 with type 21 still gives #NUM! and the error precedence of an invalid return_type at serial 0 changed (return_type error now first); neither is probed.

## Acceptance criteria
- [ ] Probe case in eval/suites/probes with oracle values; dates.md updated; engine matches.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
