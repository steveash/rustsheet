---
id: T-1248
title: "Probe DBCS on non-ASCII text and a number cell, and PHONETIC of an error or empty cell"
milestone: M5
lane: research
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel-graph evidence for the DBCS and PHONETIC cases T-1213 left unprobed, recorded in docs/excel-semantics/text.md "Text function argument modes" (VERIFIED or still UNVERIFIED per case).

## Context
T-1213 implemented DBCS as a pass-through and PHONETIC as `#N/A` from the ASCII and plain-cell probes only (`fn_dbcs`/`fn_phonetic` in functions/text/char_code.rs). Unprobed: DBCS of half-width katakana and ASCII-range punctuation (does en-US widen them?), DBCS of a number cell and of a range (lift?), PHONETIC of a cell holding an error (`#N/A` or the error?) and of an empty cell. Use the excel-probe skill; literal PHONETIC arguments are refused by the oracle.

## Acceptance criteria
- [ ] Probe cases for each listed input under eval/suites/probes with excel-graph oracle values.
- [ ] text.md updated; any mismatch filed as a functions-text task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
