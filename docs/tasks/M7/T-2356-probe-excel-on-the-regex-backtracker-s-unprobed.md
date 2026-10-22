---
id: T-2356
title: "Probe Excel on the REGEX backtracker's unprobed PCRE2 behaviors and its work budget"
milestone: M7
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The PCRE2 behaviors T-1900's backtracker implements without Excel evidence are either confirmed
by the oracle or corrected, and the work-budget DEVIATION has Excel's answer beside it.

## Context
Raised by the Excel-fidelity review of T-1900. The behaviors are expected from PCRE2 but are
UNVERIFIED in docs/excel-semantics/text.md ("REGEX* patterns and match iteration are PCRE2
(T-1688)"), and the regex budget DEVIATION entry lists the budget patterns. Use the `excel-probe`
skill, with one probe case in `eval/suites/probes/`:
- `REGEXTEST("aA","(a)(?i)\1")` and `REGEXTEST("aA","(a)\1")`: a backreference's case folding is
  scoped by `(?i)`;
- `REGEXTEST("b","(a)?b\1")`: a backreference to an unset group fails (no `PCRE2_MATCH_UNSET_BACKREF`);
- `REGEXREPLACE("aab","\Ga","-")` and `REGEXREPLACE("xab","\Gab|a","[$0]",1)`: does `\G` stand at
  each search's start offset during iteration?
- `REGEXEXTRACT("aab","(?:a*)*+b")` and `REGEXEXTRACT("b","(a?)*+b")`: PCRE2's empty-loop rule;
- `REGEXTEST("abab","^(a|b\1)+$")` (FALSE in PCRE2) and `REGEXEXTRACT("abab","^(a|b\1)+")` ("aba");
- `ISNA(INDEX(REGEXEXTRACT("aay","(?:(?>(a))z|a(a)?)y",2),1))`: a capture inside an atomic group
  is undone on backtracking;
- `REGEXEXTRACT("abcabc","(?P<n>abc)(?P=n)")`: PCRE2 accepts `(?P=name)`, while ours is `#VALUE!`;
- the budget: `REGEXTEST(REPT("a",30)&"c","\G(a|a)*b")` (ours `#ERROR!`; PCRE2 answers FALSE at
  once through its required-character check) and `REGEXTEST(REPT("a",32767),"\G(a|a)*b")`,
  `"(.*)\1x"` (ours `#ERROR!` or FALSE; Excel may fail at its match limit).

## Acceptance criteria
- [ ] A probe case with the cells above, oracle-filled (excel-graph), merged.
- [ ] text.md updated: each behavior VERIFIED or corrected, with a fix task filed for any mismatch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing fixes the probe finds (filed as tasks).
