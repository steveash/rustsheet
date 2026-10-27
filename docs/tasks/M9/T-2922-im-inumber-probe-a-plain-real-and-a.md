---
id: T-2922
title: "IM* inumber: probe a plain real and a leading-space-then-tab text with a trailing tab, NBSP, LF or CR (T-2641 applies the end rule to all IM* text)"
milestone: M9
lane: functions-engineering
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
IM* functions treat a trailing or leading tab, NBSP, LF or CR on a plain real (`"5"&CHAR(9)`) and after leading spaces as Excel does.

## Context
T-2641 probed these characters only around "3+4i" (probes/eng-inumber-currency-shapes A13:A17, all `#NUM!`) and applied the rule to all IM* text in `complex_of` (complex/mod.rs). A plain real, a real with leading spaces, and a trailing tab after a trailing space are unprobed.

## Acceptance criteria
- [ ] A probe case with oracle values covers each shape; `complex_of` matches them.
- [ ] docs/excel-semantics/engineering.md "How an inumber is read" updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other IM* readers.
