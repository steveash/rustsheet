---
id: T-1018
title: "Probe XMATCH non-exact match modes and binary search over blanks"
milestone: M1
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for XMATCH over blanks with match_mode -1/1/2 and search_mode 2/-2 (whole column/row and bounded ranges), and the engine matches it.

## Context
T-0126 verified exact XMATCH never matches a blank (docs/excel-semantics/lookup.md, blank tail). Other modes in `functions/lookup_and_reference/xmatch.rs` still compare blanks as 0/""/FALSE; also probe a blank lookup value and error cells in exact mode.

## Acceptance criteria
- [ ] Probe cases in eval/suites/probes with oracle values; lookup.md updated; engine and unit tests match.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
