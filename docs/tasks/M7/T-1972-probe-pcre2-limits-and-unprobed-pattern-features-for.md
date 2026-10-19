---
id: T-1972
title: "Probe PCRE2 limits and unprobed pattern features for the REGEX functions"
milestone: M7
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's answers to the open questions of SPEC-0013 ("Excel behavior to match", Unknowns) are in eval and recorded in
docs/excel-semantics/text.md.

## Context
SPEC-0013 (docs/specs/SPEC-0013-pcre2-regex-matcher.md) designs a PCRE2 parser and backtracker for REGEXTEST,
REGEXEXTRACT and REGEXREPLACE, and leaves constants and accept/reject lists to this probe. Write
probes/regex-pcre2-limits (excel-probe skill, excel-graph; tag M7). Use REGEXTEST, plus REGEXEXTRACT where the match
matters and ERROR.TYPE for every error cell. Cover:
- match limit: `(a|a)*b` and `(a*)*b` on `REPT("a",n)` for n = 20, 25, 30, 1000, 32767;
- nesting: 250 and 251 nested groups;
- bounded variable lookbehind: `(?<=a{1,3})b` and `(?<=ab|c)d`;
- recursion and subroutines: `(?R)`, `(?1)`, `(?&n)`;
- `.` over `UNICHAR(128512)`, both REGEXEXTRACT all-matches and LEN;
- escapes and classes: `\X`, `\R`, `\h`, `\v`, `\N`, `\x{1F600}`, `\p{Greek}`, `[[:^alpha:]]`, `\d` on
  `UNICHAR(1633)`;
- `$` and `\Z` before a final `CHAR(10)`; `.` on `CHAR(10)` and `CHAR(13)`; `(?s)`, `(?m)`, `(?n)`;
- verbs and callouts: `(*SKIP)(*FAIL)`, `(*UTF)`, `(?C1)`;
- `\G`;
- `x*+` and `x{2,}+` possessives.

## Acceptance criteria
- [ ] probes/regex-pcre2-limits has excel-graph values for every question above. Record a question excel-graph
  cannot answer as an M14 desktop question (decision 0039).
- [ ] text.md has one entry per finding (VERIFIED), and SPEC-0013's Unknowns list points to them.
- [ ] Each feature group Excel accepts and SPEC-0013's rollout does not cover (recursion, `\X`, scripts, ...) gets
  a follow-up task (`make new-task`, functions-text, depends on T-1973).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing anything.
