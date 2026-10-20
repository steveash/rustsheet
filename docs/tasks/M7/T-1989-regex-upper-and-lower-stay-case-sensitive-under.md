---
id: T-1989
title: "REGEX* [:upper:] and [:lower:] stay case-sensitive under (?i) when nested or negated"
milestone: M7
lane: functions-text
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`[:upper:]` and `[:lower:]` ignore case-insensitive matching wherever they appear in a bracket, as Excel's do.

## Context
probes/regex-posix-classes-b (excel-graph, T-1903):
- A8: `REGEXTEST("a","(?i)[[[:upper:]]]")` is FALSE in Excel; ours is TRUE.
- A9: `(?i)[^[:upper:]]` on "a" is TRUE in Excel; ours is FALSE.
- A10: `(?i)[[:^upper:]]` on "a" is TRUE in Excel; ours is FALSE.
- A11: `(?i)[[:^lower:]]` on "A" is TRUE in Excel; ours is FALSE.
So Excel never folds these two classes, and a negation complements the unfolded class.
T-1903's `case_sensitive_split` (functions/text/regex_posix.rs) only handles a non-negated
top-level bracket of plain items, by turning it into an alternation with `(?-i:\p{Lu})`. The
other forms reach regex-syntax, which folds the whole bracket. A general fix probably builds the
bracket's HIR class itself, folding only the other items, or wraps the bracket in `(?-i:...)`
and folds the other items explicitly. The case_sensitivity argument 1 already behaves (A16-A18).
Probe first: A8 does not prove nesting (PCRE2 reads `[[[:upper:]]]` as `[[[:upper:]]` and a
literal `]`). Add `REGEXTEST("a]","(?i)[[[:upper:]]]")`, `REGEXTEST("[","[[a]]")`, `REGEXTEST("a","[[a]]")`,
`(?i)[[:ascii:]]` on U+212A and U+017F, and the unprobed code points (U+061C, U+180E, U+2066-U+2069,
U+0903 in word) before designing. See docs/excel-semantics/text.md "REGEX* POSIX classes are Unicode (T-1903)".

## Acceptance criteria
- [ ] probes/regex-posix-classes-b A8-A11 pass; probes/regex-posix-classes stays green.
- [ ] `make check` green; reviewers approve.

## Out of scope
- POSIX syntax errors (T-1985).
