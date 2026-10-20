---
id: T-1985
title: "REGEX* POSIX class syntax errors match PCRE2 (outside a bracket, unknown names, collating elements, class range ends)"
milestone: M7
lane: functions-text
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A REGEX* pattern whose POSIX class syntax PCRE2 rejects is `#VALUE!` (ERROR.TYPE 3), as Excel's is.

## Context
probes/regex-posix-classes column Z (excel-graph, T-1903): Excel gives `err 3` (the pattern
is invalid) for each of these, and ours compiles them:
- Z1 `REGEXTEST("a:","[:alpha:]")`: a POSIX class outside a bracket (ours TRUE).
- Z2 `[[:foo:]]` and Z3 `[[:ALPHA:]]`: an unknown or upper-case class name (ours FALSE).
- Z8 `[[.a.]]` and Z9 `[[=a=]]`: POSIX collating elements and equivalence classes (ours TRUE).
- Z10 `REGEXTEST("-","[[:alpha:]-z]")`: a range whose start is a class (ours TRUE); also
  probes/regex-posix-classes-b A4 and A5 (`[a[:upper:]-z]`).
Z7 (`\Q[[:alpha:]]\E`) belongs to the `\Q..\E` gap of T-1688, not to this task.
The pattern is parsed with regex-syntax's AST in functions/text/regex_posix.rs
(`unicode_posix_classes`, T-1903). The parser keeps these forms as literals (or as a class
for Z1), so the check probably goes on the pattern text or the AST there: it flags the pattern
invalid, and `compile` treats it as `BuildFailure::Invalid`. See docs/excel-semantics/text.md
"REGEX* POSIX classes are Unicode (T-1903)".

## Acceptance criteria
- [x] probes/regex-posix-classes Z1, Z2, Z3, Z8, Z9 and Z10 and probes/regex-posix-classes-b A4 and A5 pass; no other probes cell regresses.
- [x] `make check` green; reviewers approve.

## Out of scope
- `\Q..\E` (T-1688) and any other PCRE2 dialect gap.

## Outcome
`unicode_posix_classes` now returns `Err(InvalidPosixSyntax)` for a bracket shaped `[:x:]`, `[.x.]`
or `[=x=]` that the parser kept as a plain bracket (outside a bracket, unknown or upper-case name,
collating element, equivalence class) and for a class followed by `-` and another item;
`compile` treats it as an invalid pattern (`#VALUE!`). Eval: probes known failures 1876 -> 1868
(Z1, Z2, Z3, Z8, Z9, Z10, A4, A5 fixed), no regressions. Not done: a range that ends at a class
(`[a-[:alpha:]]`), which the task did not list and no probe covers.
