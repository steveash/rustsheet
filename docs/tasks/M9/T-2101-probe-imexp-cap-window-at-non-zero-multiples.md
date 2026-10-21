---
id: T-2101
title: "Probe IMEXP cap window at non-zero multiples of pi/2 and numeric DBL_MAX text"
milestone: M9
lane: functions-engineering
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The IMEXP overflow-cap window around non-zero multiples of pi/2 is pinned exactly, and numeric DBL_MAX text is known.

## Context
T-2087 fitted 2^-26/sqrt(2) in `imexp_cap_window` (complex.rs). Brackets so far: pi below (1.04,1.06]E-8, pi above and 2pi both sides (1.05,1.10]E-8, pi/2 and 3pi/2 (1.0,1.2]E-8. Also unprobed: the tie at exactly 2^-26, `1000-2E8i`, large multiples (100*pi), and numeric `DBL_MAX&""` / TEXT (avoid a DBL_MAX literal, excel-graph rejects it; try `=2^1023*(2-2^-52)`).

## Acceptance criteria
- [ ] Probe cases with oracle values narrow each bracket; `imexp_cap_window` and engineering.md match them.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
