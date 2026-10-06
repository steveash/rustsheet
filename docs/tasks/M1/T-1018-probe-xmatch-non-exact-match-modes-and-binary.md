---
id: T-1018
title: "Probe XMATCH non-exact match modes and binary search over blanks"
milestone: M1
lane: research
status: done
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
- [x] Probe cases (also `XMATCH(G1,A:A)` with a blank lookup value, bounded `XMATCH(0,A1:A10)` with a blank inside, and XLOOKUP exact/approximate over blanks, whose own `linear_search` in `functions/xlookup.rs` still matches blanks) in eval/suites/probes with oracle values; lookup.md updated; engine and unit tests match.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `xmatch-xlookup-blanks` (69 cells, Col/Err/Row) filled from excel-graph. Linear XMATCH/XLOOKUP
now share `linear_scan`: blanks never match a non-blank lookup value, a blank lookup value matches
a blank, and in match_mode -1/1 a blank sorts as empty text (docs/excel-semantics/lookup.md). Fixed 23 of
the 30 initial mismatches (this also corrects T-0126's blanket blank skip for a blank lookup value);
the remaining 7 are binary search_mode cells, filed as T-1019 (opus). Eval: probes +62 passing cells,
7 known failures (new case), no regressions.
