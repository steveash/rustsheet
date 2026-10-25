---
id: T-2660
title: "XLOOKUP match_mode 3 (regex) as XMATCH reads it"
milestone: M6
lane: functions-lookup
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
XLOOKUP accepts `match_mode` 3 (regular-expression match), as Excel does, instead of rejecting it as `#VALUE!`.

## Context
Found by T-2656. Excel reads a match_mode of 3, or anything that floors to 3, as the regex mode in XLOOKUP as well as
XMATCH: `XLOOKUP(2.5,{1,2,3},{10,20,30},"nf",3)` is "nf" (no match, so if_not_found), and so are 3.5 and 3-2^-51. In the
same probes the XMATCH twins give `#N/A`, and both reject an invalid pattern (`XMATCH("?",{"z","a"},3)` and
`XLOOKUP("?",...,3)` are `#VALUE!`).
Failing cells (probes, excel-graph, 2026-10-25): probes/t2656-xmatch-mode-near-integer G7 (mode 3), G21 (3-2^-51);
probes/t2656-xmatch-mode-rounding G15 (3.5); probes/t2656-xlookup-mode-snap-edge C8 (3 - 2^-22*(1+2^-11), which snaps to 3). H7 and H21 (`"?"`, `#VALUE!`) already pass for the wrong reason (ours
rejects the mode).
Code: `xlookup_modes` in crates/rustsheet-engine/src/functions/xlookup.rs has no regex arm. XMATCH's regex scan
(`MatchMode::Regex` in functions/lookup_and_reference/xmatch.rs) is the one to reuse, never copy. XLOOKUP maps its
modes onto `XmatchMode` for `linear_scan`, so it likely needs the regex arm there too. Probe the binary search_modes
with regex, and XLOOKUP's regex over numbers and booleans, before implementing either.

## Acceptance criteria
- [ ] The four cells above pass; no baseline gains a line.
- [ ] A probe pins XLOOKUP's regex mode for text, numbers and the binary search modes, recorded in lookup.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- XMATCH's regex mode (implemented).
