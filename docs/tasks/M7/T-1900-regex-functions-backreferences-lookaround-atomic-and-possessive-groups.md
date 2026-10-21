---
id: T-1900
title: "REGEX functions: backreferences, lookaround, atomic and possessive groups, K, conditionals"
milestone: M7
lane: functions-text
status: todo
priority: P2
size: M
tier: opus
campaign: visible
depends_on: [T-1973]
---

## Goal
The REGEX* functions accept the PCRE2 pattern features Excel accepts and match as Excel does.

## Context
Found by T-1688 (probes/regex-pcre2-features C1-C19, B27-B29; t1626-regextest C29): backreferences `\1`, `\g1`, `\k<n>` (case-insensitive too), lookahead, fixed-length lookbehind (`(?<=a+)` is `#VALUE!`), possessive `a++` and atomic `(?>...)` (no match: `#N/A`), `\K`, conditionals, `(?#...)` comments, `\Q...\E`, `(?x)` and `(?-i)`. Design: SPEC-0013 section 3 (rollout step 3). Compile T-1973's `Ast` to `pcre::Program` (flat instructions, counted repeats) and run it on `pcre::Backtracker`. The backtracker has one explicit stack, no recursion, and barrier frames for lookaround and atomic groups. Possessive quantifiers compile as atomic groups. It also needs PCRE2's empty-loop rule. Every instruction and frame is charged to the call's `Spent` budget (placeholder costs; T-1974 fits them), and the stack is capped in bytes. `Pattern` routes non-regular patterns here. Add a differential test that runs random regular patterns on both engines and compares the match and capture spans.

Visibly-off cells (gap triage 2026-10-21, decision 0083): 14 cells, 0 numeric past half a unit in the 15th digit and 14 with a wrong type, error or text, blamed on REGEXEXTRACT 11, REGEXTEST 3, ISERROR 1.

| case / cell | formula | Excel | ours |
|---|---|---|---|
| probes/regex-pcre2-features Sheet1!B29 | `=ERROR.TYPE(REGEXEXTRACT("aaa","a++a"))` | 7 | #N/A |
| probes/regex-pcre2-features Sheet1!C1 | `=REGEXTEST("abab","^(ab)\1$")` | TRUE | #VALUE! |
| probes/regex-pcre2-features Sheet1!C10 | `=REGEXEXTRACT("1a2b","(?<!1)[a-z]")` | "b" | #VALUE! |
| probes/regex-pcre2-features Sheet1!C11 | `=ISERROR(REGEXEXTRACT("aaa","a++a"))` | TRUE | FALSE |
| probes/regex-pcre2-features Sheet1!C13 | `=REGEXEXTRACT("foobar","foo\Kbar")` | "bar" | #VALUE! |
| probes/regex-pcre2-features Sheet1!C14 | `=REGEXEXTRACT("ab","(a)?(?(1)b\|c)")` | "ab" | #VALUE! |
| probes/regex-pcre2-features Sheet1!C2 | `=REGEXTEST("abac","^(ab)\1$")` | FALSE | #VALUE! |
| probes/regex-pcre2-features Sheet1!C3 | `=REGEXEXTRACT("abcabc","(?<n>abc)\k<n>")` | "abcabc" | #VALUE! |
| probes/regex-pcre2-features Sheet1!C4 | `=REGEXEXTRACT("xaay","(a)\g1")` | "aa" | #VALUE! |
| probes/regex-pcre2-features Sheet1!C5 | `=REGEXEXTRACT("xAay","(a)\1",0,1)` | "Aa" | #VALUE! |
| probes/regex-pcre2-features Sheet1!C7 | `=REGEXEXTRACT("aXbXc","(?<=X)\w")` | "b" | #VALUE! |
| probes/regex-pcre2-features Sheet1!C8 | `=REGEXEXTRACT("price 30 dollars","\d+(?= dollars)")` | "30" | #VALUE! |
| probes/regex-pcre2-features Sheet1!C9 | `=REGEXEXTRACT("a1b2","[a-z](?!1)")` | "b" | #VALUE! |
| probes/t1626-regextest Sheet1!C29 | `=REGEXTEST(A1,"(\w)\1")` | TRUE | #VALUE! |

## Acceptance criteria
- [ ] Visible parity: the 14 visibly-off cells listed under Context (2026-10-21) have Excel's value to 15 significant digits, or its type, error and text: the 14 lines in eval/baseline/probes.txt disappear (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] t1626-regextest C29, regex-pcre2-features C1-C19 and B27-B29 pass; baseline shrinks.
- [ ] Differential unit test: random regular patterns run on the automata and on the backtracker give the same match
  and capture spans.
- [ ] Directive 5:
  - no recursion;
  - the stack lives in `Caches` and is capped in bytes during the search;
  - work is charged to the call's one work counter, `Backref` and lookbehind per character and barrier pops per
    frame, at conservative placeholder costs (overcharge, never undercharge);
  - a release-mode test runs `(a|a)*b`, `(a*)*b`, `((a+)+)+b` and `(.*)\1x` on 32,767 characters, which end in
    `Spent` (`#ERROR!`) or an answer within a stated wall-clock bound, without a stack overflow.
- [ ] Every regular program on the backtracker runs in the memoized mode (SPEC-0013 section 3), counter-free and
  guard-free. Its bit-set is allocated only if it fits what the call may still hold; otherwise the call is
  `#ERROR!`. `(a|a)*b$` over a text ending in `CHAR(10)` is linear.
- [ ] If the task proves bigger than M, split it: the core VM with backreferences, atomic and possessive groups
  first, then lookaround, conditionals and `\K` as a follow-up.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Iteration after empty matches (T-1899).

More cells (T-1829, probes/regex-ascii-classes, excel-graph 2026-10-19; its POSIX-class cells A15, A16 are T-1903's):
`(*UCP)`, `\X`, `\h` and `\R` are accepted (A17, A27, A41, A42, ours `#VALUE!`); `$` matches
before a final LF (A34, ours FALSE). text.md "REGEX* character classes are Unicode".

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (`#VALUE!` where Excel accepts backreferences, lookaround, `\K`, conditionals and possessive groups), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it.
