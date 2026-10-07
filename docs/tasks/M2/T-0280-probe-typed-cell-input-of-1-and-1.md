---
id: T-0280
title: "Probe typed cell input of (1) and (1,000) needs a typed-entry oracle path"
milestone: M2
lane: research
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence for the value and number format Excel stores when `(1)`, `(1,000)`, `( 1 )`, `(-1)`, `($1)`, `(1%)` are typed into a cell.

## Context
T-0277 could only probe text coercion: case-file strings are written as text cells, so Excel never parses them as typed input. Needs an oracle path that enters values as typed (e.g. Graph range `values` writes, or a `typed` cell tag in spec2xlsx/oracle fill) and reads back `numberFormat`; then compare with the engine's `set_user_input` (-1 with no format, -1000 with `#,##0`).

## Acceptance criteria
- [ ] Oracle can enter typed input and return its number format; a probe case holds the six forms; coercion.md records them; an engine task is filed if the engine differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
