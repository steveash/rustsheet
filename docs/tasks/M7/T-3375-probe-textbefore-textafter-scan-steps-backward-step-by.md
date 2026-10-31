---
id: T-3375
title: "Probe TEXTBEFORE/TEXTAFTER scan steps: backward step by match length or shortest delimiter; forward step past a short entry inside a longer match"
milestone: M7
lane: functions-text
status: todo
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
- [ ] One probe case (M7 tag) oracle-filled and committed; the text.md entry's Unprobed line is replaced by the finding.
- [ ] Every probe cell passes, or a remaining one is named in a follow-up task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
