---
id: T-2092
title: "Probe whether REGEX* \p{..} and Perl classes fold under (?i), inside and outside a POSIX bracket"
milestone: M7
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Know, from Excel, whether `(?i)` folds a `\p{..}` class and `\w`, `\d`, `\s` in REGEX* patterns, inside and outside a bracket that holds a POSIX class, and match it.

## Context
T-1989 made `unicode_posix_classes` (functions/text/regex_posix.rs) fold every non-POSIX item of a bracket under `(?i)`,
`\p{..}` and Perl classes included, as the regex crate folds them everywhere. Only literals and ranges are probed
(probes/regex-posix-classes-c A11-A22). PCRE2's docs say caseless matching does not affect `\p` escapes, so Excel likely
gives FALSE where ours gives TRUE. Probe cells: `REGEXTEST("a","(?i)\p{Lu}")`, `REGEXTEST("a","(?i)[\p{Lu}]")`,
`REGEXTEST("a","(?i)[[:digit:]\p{Lu}]")`, `REGEXTEST("A","(?i)[^\p{Ll}]")`, `REGEXTEST(UNICHAR(8490),"(?i)[[:digit:]k]")`,
and the same with case_sensitivity 1. docs/excel-semantics/text.md "REGEX* POSIX classes are Unicode (T-1903)" carries
the UNVERIFIED line to replace. If `\p` does not fold, `folded` in regex_posix.rs keeps `\p` items unfolded and a
`\p` outside a bracket needs the same `(?-i:...)` treatment (perhaps opus-sized: re-tier then).

## Acceptance criteria
- [ ] A probe case with the cells above has excel-graph values, recorded in text.md.
- [ ] Ours matches them, or a follow-up task is filed for the part that does not.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
