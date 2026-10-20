---
id: T-2186
title: "Probe where IM* text switches large parts to E notation"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The cut between plain and E notation for large IM* parts is known.

## Context
probes/eng-imtan-overflow B10: `IMCOT("1E-20")` is `100000000000000000000` (21 characters) in Excel, ours `1E+20`; `1E+200` is E notation (probes/eng-imdiv A19). Probe parts 1E+15 to 1E+25 through IMSUM/IMPRODUCT and fix complex_part_to_text. T-2056 covers small parts only.

## Acceptance criteria
- [ ] Probe case with oracle values for exponents 15 to 25; `complex_part_to_text` matches.
- [ ] `make check` green; reviewers approve.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
