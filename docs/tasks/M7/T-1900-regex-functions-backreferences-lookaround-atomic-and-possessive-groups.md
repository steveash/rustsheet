---
id: T-1900
title: "REGEX functions: backtracker core with backreferences, atomic and possessive groups"
milestone: M7
lane: functions-text
status: in-progress
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

## Split (2026-10-22)
The task was larger than M (a VM, a memoized mode, a differential test and the iteration
integration), so it is split per its own clause. T-1900 keeps the core: `pcre::Program` and
`pcre::Backtracker` with backreferences, atomic and possessive groups, routed from
`regex_functions.rs` for non-regular patterns. Cells: C1-C5, C11, B29 and t1626-regextest C29.
- T-2354: lookaround, conditionals and `\K` (C7-C10, C13, C14).
- T-2355: the memoized mode for regular programs, the final-LF `$` on texts with another `\n`,
  and the differential test.

## Acceptance criteria
- [ ] Visible parity: the eight visibly-off cells T-1900 keeps (regex-pcre2-features C1-C5, C11, B29 and
      t1626-regextest C29) have Excel's value, type, error and text. Their lines leave eval/baseline/probes.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline. The other six cells are T-2354's.
- [ ] The regex-pcre2-features C1-C19 and B27-B29 cells that use only backreferences, atomic or possessive
      groups pass; baseline shrinks.
- [ ] Directive 5:
  - no recursion;
  - the stack lives in `Caches` and is capped in bytes during the search;
  - work is charged to the call's one work counter, `Backref` and lookbehind per character and barrier pops per
    frame, at conservative placeholder costs (overcharge, never undercharge);
  - a release-mode test runs `(a|a)*b`, `(a*)*b`, `((a+)+)+b` and `(.*)\1x` on 32,767 characters, which end in
    `Spent` (`#ERROR!`) or an answer within a stated wall-clock bound, without a stack overflow.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Iteration after empty matches (T-1899).
- Lookaround, conditionals and `\K` (T-2354); the memoized mode, the final-LF `$` and the differential test (T-2355).

More cells (T-1829, probes/regex-ascii-classes, excel-graph 2026-10-19; its POSIX-class cells A15, A16 are T-1903's):
`(*UCP)`, `\X`, `\h` and `\R` are accepted (A17, A27, A41, A42, ours `#VALUE!`); `$` matches
before a final LF (A34, ours FALSE). text.md "REGEX* character classes are Unicode".

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (`#VALUE!` where Excel accepts backreferences, lookaround, `\K`, conditionals and possessive groups), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it.
