---
id: T-1338
title: "Probe TEXT of tiny negatives under padded and h/m elapsed tokens"
milestone: M1
lane: functions-date
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's output for a negative under an elapsed token is established for every magnitude and token, and the engine matches it.

## Context
T-1329 made `elapsed_text` (formatter/format.rs) give "-0" for any negative of magnitude below one unit, for all six elapsed tokens. Only `TEXT(-1E-10,"[s]")` has oracle evidence (probes/date-system-1900-negatives A18). Unprobed: `[ss]`/`[hh]`/`[mm]`/`[h]`/`[m]`, fractions such as -0.9 s and -1.7 s (floor vs truncate toward zero), in both date systems. Use the `excel-probe` skill.

## Acceptance criteria
- [ ] A probe case with oracle values covers `TEXT(-0.9/86400,"[s]")`, `TEXT(-1.7/86400,"[s]")` and `TEXT(-1E-10,"[ss]")`, `"[hh]"`, `"[m]"`, `"[mm]"` in both date systems.
- [ ] The engine matches them or a fix task is filed; docs/excel-semantics/number-formats.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
