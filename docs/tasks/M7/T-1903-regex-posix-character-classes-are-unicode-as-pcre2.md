---
id: T-1903
title: "REGEX* POSIX character classes are Unicode, as PCRE2 UCP's are"
milestone: M7
lane: functions-text
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-1973, T-1972]
---

## Goal
REGEX* POSIX bracket classes (`[[:alpha:]]` and the rest) match what Excel's PCRE2 matches, including non-ASCII letters.

## Context
Excel: `REGEXTEST("é","[[:alpha:]]")` is TRUE (probes/regexreplace-limit-and-syntax A23, excel-graph); ours is FALSE
because the regex crate (regex-syntax) defines POSIX classes as ASCII-only, while `\w`, `\pL` and `\b` are already
Unicode in both. Excel's other classes are unprobed: probe each of `alnum alpha ascii blank cntrl digit graph lower
print punct space upper word xdigit` (and a negated `[[:^alpha:]]`) on ASCII, Latin-1 (é, ß, ²), Greek, CJK, fullwidth
digits and an emoji before implementing (PCRE2 with UCP maps most to Unicode properties, but `[:punct:]`, `[:graph:]`,
`[:print:]` and `[:xdigit:]` have special rules; follow the oracle, not PCRE2's docs). Design (SPEC-0013, 2026-10-19): there is no pre-compile rewrite. T-1973's PCRE2
parser builds each POSIX class as a `ClassUnicode`, and this task changes that mapping to the classes T-1972's
per-class probe shows (the probe moved there). Size S once T-1972 and T-1973 have landed.

## Acceptance criteria
- [ ] T-1972's per-class cells pass; text.md records the rule.
- [ ] probes/regexreplace-limit-and-syntax A23 passes and the new cells pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other PCRE2 dialect gaps (text.md "REGEX* patterns and match iteration are PCRE2 (T-1688)").
