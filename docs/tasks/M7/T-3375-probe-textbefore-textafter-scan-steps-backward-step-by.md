---
id: T-3375
title: "Probe TEXTBEFORE/TEXTAFTER scan steps: backward step by match length or shortest delimiter; forward step past a short entry inside a longer match"
milestone: M7
lane: functions-text
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Pin the two scan steps of TEXTBEFORE/TEXTAFTER's disjoint instance count that T-3335 inferred but did not probe, and
match Excel if ours differs.

## Context
T-3335 (probes/t3335-textbefore-overlapping-instances, gen-m7/a470dc9e165bdb20 Calc!A1441) established that the scans
count disjoint matches: forward resumes after a counted match, backward steps back from its start by the counted match's
length and checks the whole rest there. Every cell so far fits both "step by the match's length" and "step by the
shortest delimiter's length" (docs/excel-semantics/text.md "TEXTBEFORE and TEXTAFTER: overlapping delimiter
instances", Unprobed). Ours: `text_sensitive` / `text_sensitive_reverse` in crates/rustsheet-engine/src/functions/text/util.rs.
Discriminating cells to probe (excel-graph), e.g.:
- backward: `TEXTAFTER("abcab",{"ab","c"},-2)` (match length: "cab"; shortest delimiter: "ab"), and the list reversed;
- forward: `TEXTBEFORE("abcb",{"abc","b"},2)` (resume after the match: "abc"; resume by the shortest: "a"),
  `TEXTAFTER("abcbc",{"abc","bc"},2)`;
- the same with match_mode 1 and with a surrogate pair inside the counted match (UTF-16 step vs char step).

## Acceptance criteria
- [x] One probe case (M7 tag) oracle-filled and committed; the text.md entry's Unprobed line is replaced by the finding.
- [x] Every probe cell passes, or a remaining one is named in a follow-up task.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe probes/t3375-textbefore-scan-steps (24 cells, excel-graph, 2026-10-31). Excel steps back by the counted match's
own length in UTF-16 units (not the shortest or longest delimiter, not characters: A1, A5, A6, A9) and a forward scan
resumes after the counted match even when a shorter entry starts inside it (B1-B8). text.md's Unprobed line is replaced
by the finding. Ours (`text_sensitive` / `text_sensitive_reverse`) already did both: every probe cell passes, no code
change. Eval delta: +24 passing probe cells (new case), no regressions.
