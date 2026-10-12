---
id: T-1296
title: "Probe SEARCH positions after case-folding that changes length (U+0130) and SEARCH of empty text"
milestone: M2
lane: functions-text
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
SEARCH reports the same positions as Excel when lowercasing changes a character's length, and SEARCH of "" in "" matches Excel.

## Context
Found by the T-0249 review. `find_element` in crates/rustsheet-engine/src/functions/text/common.rs
lowercases both texts with Rust `to_lowercase()` before counting UTF-16 positions; some characters
change length when lowercased (U+0130 "İ" becomes "i" + U+0307), which could shift the reported
position (e.g. `SEARCH("x","İx")`). Also unprobed: `SEARCH("","")` (the engine gives #VALUE!;
`FIND("","",1)` is 1, probes/text-surrogate-positions C5). See docs/excel-semantics/text.md
"Text positions count UTF-16 code units, except in LEFT and RIGHT".

## Acceptance criteria
- [ ] Probe case (excel-probe) with `SEARCH("x","İx")`, `SEARCH("i","İx")`, `SEARCH("x","ẞx")`, `SEARCH("","")`, `SEARCH("",""&"",1)`; oracle values recorded.
- [ ] Findings in docs/excel-semantics/text.md; the engine matches them (or a follow-up task is filed if the fix is large).
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed (probes/text-search-casefold-length, excel-graph). Excel folds case per character without changing length; `find_element` now uses `fold_case` instead of `to_lowercase`. The empty-text cases already matched. 5 probe cells fixed.
