---
id: T-1903
title: "REGEX* POSIX character classes are Unicode, as PCRE2 UCP's are"
milestone: M7
lane: functions-text
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
REGEX* POSIX bracket classes (`[[:alpha:]]` and the rest) match what Excel's PCRE2 matches, including non-ASCII letters.

## Context
Excel: `REGEXTEST("é","[[:alpha:]]")` is TRUE (probes/regexreplace-limit-and-syntax A23, excel-graph); ours is FALSE
because the regex crate (regex-syntax) defines POSIX classes as ASCII-only, while `\w`, `\pL` and `\b` are already
Unicode in both. Excel's other classes are unprobed: probe each of `alnum alpha ascii blank cntrl digit graph lower
print punct space upper word xdigit` (and a negated `[[:^alpha:]]`) on ASCII, Latin-1 (é, ß, ²), Greek, CJK, fullwidth
digits and an emoji before implementing (PCRE2 with UCP maps most to Unicode properties, but `[:punct:]`, `[:graph:]`,
`[:print:]` and `[:xdigit:]` have special rules; follow the oracle, not PCRE2's docs). The pattern is passed to
regex-automata unchanged (functions/text/regex_engine.rs `Engine::new`), so the fix is a rewrite of POSIX classes
inside bracket expressions before compiling, which must respect escapes, `\Q...\E` and nested brackets, and must not
grow a pattern past the compile limits (T-1272). Size M; needs a design call on where the rewrite lives, so opus.

## Acceptance criteria
- [x] A probe case with oracle values covers every class; text.md records the rule.
- [x] probes/regexreplace-limit-and-syntax A23 passes and the new cells pass.
- [x] `make check` green; reviewers approve.

## Out of scope
- Other PCRE2 dialect gaps (text.md "REGEX* patterns and match iteration are PCRE2 (T-1688)").

## Outcome
Probe `probes/regex-posix-classes` (excel-graph): 47 characters × 19 patterns (the 14 classes, two
negated classes, a negated bracket and `(?i)` upper and lower), plus column Z for syntax edges.
New `functions/text/regex_posix.rs` parses the pattern with regex-syntax's AST, rewrites each
POSIX class item to the Unicode class Excel matches and prints the pattern back before it is
compiled. Upper and lower become a `(?-i:...)` alternative, because Excel's `(?i)` does not fold
them. text.md "REGEX* POSIX classes are Unicode (T-1903)" records the rule. Eval: probes 905 new
cells, of which 898 pass; regexreplace-limit-and-syntax A23 is fixed. Seven new known failures:
the six syntax-error edges Z1, Z2, Z3, Z8, Z9 and Z10 go to follow-up T-1985, and Z7 (`\Q..\E`)
stays with T-1688. Review follow-up: probes/regex-posix-classes-b (18 cells, excel-graph) pins
the bracket rewrite (a literal `^`/`&` next to a class) and `&&`. A4 and A5 (a range from a class
is an error) go to T-1985, and A8-A11 (`(?i)` upper and lower when nested or negated) to T-1989.
