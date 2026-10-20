---
id: T-2077
title: "Probe complex overflow edges: blank before the overflow in IMPRODUCT, and IMCOSH near 710"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Two unprobed edges of the T-2003 complex-overflow rules are evidenced by excel-graph and the engine matches them.

## Context
T-2003 review: `IMPRODUCT(<blank>,"1E200","1E200")` (blank before the overflow) is assumed `#VALUE!`; IMCOSH/IMSINH/IMSIN/IMCOS near the overflow edge (e.g. `IMCOSH("710.4+1.5i")`, where cosh x overflows but the product may be finite) return `#NUM!` here and may return text in Excel. See engineering.md "IM* results that overflow".

## Acceptance criteria
- [ ] a probe of both edges, filled by excel-graph, with mismatches fixed in complex.rs and recorded in engineering.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
