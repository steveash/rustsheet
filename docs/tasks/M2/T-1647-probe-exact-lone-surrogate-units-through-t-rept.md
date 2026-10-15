---
id: T-1647
title: "Probe exact lone-surrogate units through T, REPT and TEXT (T-1587 follow-up)"
milestone: M2
lane: research
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence that T, REPT and TEXT keep the exact lone-surrogate unit (not U+FFFD), and that REPT re-pairs at joins.

## Context
T-1587 implemented this from the UTF-16 model; existing cells (probes/text-lone-surrogates A62-A80) read lengths only. With H = MID("😀",1,1), L = MID("😀",2,1) add: `EXACT(T(H),H)`, `EXACT(REPT(H,2),H&H)`, `EXACT(TEXT(H,"@"),H)`, `UNICODE(MID(TEXT(L,"x@y"),2,1))`, `EXACT(REPT(L&H,2),L&H&L&H)`, `TEXT(H&REPT("a",127),"@@")` and the 126 variant (255-character limit). Use the excel-probe skill.

## Acceptance criteria
- [ ] Cells added to probes/text-lone-surrogates with oracle values; text.md and SPEC-0010 UNVERIFIED markers resolved; any divergence filed as a task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
