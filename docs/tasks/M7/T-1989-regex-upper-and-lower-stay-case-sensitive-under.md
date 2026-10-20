---
id: T-1989
title: "REGEX* [:upper:] and [:lower:] stay case-sensitive under (?i) when nested or negated"
milestone: M7
lane: functions-text
status: done
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
- [x] probes/regex-posix-classes-b A8-A11 and probes/regex-posix-classes-c A3, A8, A9, A14, A16, A18, A23, A26 pass;
  probes/regex-posix-classes stays green.
- [x] probes/regex-posix-classes-c A29 (`[[:space:]]` matches U+180E) passes.
- [x] `make check` green; reviewers approve.

## Out of scope
- POSIX syntax errors (T-1985).

## Probe result (T-1989, 2026-10-20)
probes/regex-posix-classes-c (excel-graph) answers the "Probe first" questions; docs/excel-semantics/text.md "REGEX* POSIX
classes are Unicode" records them. Brackets do nest in Excel (`[[a]]` is not `[` then `]`: A4, A5), no POSIX class is
case-folded (`(?i)[[:ascii:]]` misses the Kelvin sign, A8-A9), and every other item of the bracket still folds, also under
negation, `&&`, nesting and scoped flags (A11-A26). Ours fails A3, A8, A9, A14, A16, A18, A23, A26 and A29 (U+180E is space).

Design (for whoever implements): the special case `case_sensitive_split` cannot express a negation or an intersection as an
alternation. Instead, while walking the AST in `rewrite`, track whether case-insensitive matching is on (start from
`case_insensitive`, which `unicode_posix_classes` must then take; an `Ast::Flags` changes it for the rest of its group,
across `|`; a flagged `Ast::Group` scopes it), and when a bracket holding any POSIX class is reached with it on, print it as
`(?-i:[...])` with each non-POSIX leaf (literal, range, `\p`, Perl class) replaced by its case closure, taken from
regex-syntax's own folding (translate `(?i)[leaf]` to HIR and print its ranges; merge a union's leaves into one class
first so the text stays bounded by Unicode's range count, and cap the output, directive 5). The two cache slots in
regex_functions.rs (`index = usize::from(case_insensitive)`) already key the compiled pattern by the flag.

## Outcome
`unicode_posix_classes` (functions/text/regex_posix.rs) now takes the case_sensitivity flag and tracks `(?i)` through the
AST as regex-syntax does. Where it is on, a bracket holding any POSIX class, at any depth, negated or under `&&`, becomes
`(?-i:[...])` with every other item replaced by its case closure (regex-syntax's own folding, a union's plain items folded
together), so no POSIX class folds and the rest still does. This replaces `case_sensitive_split`. `[:space:]` gained U+180E.
A bracket that would print past 64 KiB is `#VALUE!` (DEVIATION, directive 5, recorded in text.md).
Eval delta: 13 known failures fewer (regex-posix-classes-b A8-A11, regex-posix-classes-c A3, A8, A9, A14, A16, A18, A23,
A26, A29), none new; the new probe case adds 36 Excel cells.
