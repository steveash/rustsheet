---
id: T-2354
title: "REGEX functions: lookaround, conditionals and the K escape on the T-1900 backtracker"
milestone: M7
lane: functions-text
status: todo
priority: P2
size: M
tier: opus
campaign: visible
depends_on: [T-1900]
---

## Goal
Lookahead, lookbehind (fixed-length per alternative), conditionals and `\K` in REGEX* patterns
match as Excel does instead of returning `#VALUE!`.

## Context
Split from T-1900 (its own split clause: "the core VM with backreferences, atomic and possessive
groups first, then lookaround, conditionals and `\K` as a follow-up"). Design: SPEC-0013 section 3,
instructions `LookStart(kind, len)` / `LookEnd`, `CondGroup` / `CondAssert` and `KeepOut` on T-1900's
`pcre::Program` and `pcre::Backtracker`. Lookaround runs as a sub-match on the same explicit stack,
delimited by a barrier frame. A lookbehind of fixed length L starts at `pos - L` in characters and is
charged per character stepped back. The parser already builds `Ast::LookAround`, `Ast::Conditional`
and `Ast::KeepOut`, and it already rejects a lookbehind that is not fixed-length (`ast::fixed_length`).
Assertion conditions (`(?(?=x)..)`) are invalid in the parser today. Probe their Excel answer
(`excel-probe`) before accepting them.

Visibly-off cells (from T-1900's list, decision 0083):

| case / cell | formula | Excel | ours |
|---|---|---|---|
| probes/regex-pcre2-features Sheet1!C7 | `=REGEXEXTRACT("aXbXc","(?<=X)\w")` | "b" | #VALUE! |
| probes/regex-pcre2-features Sheet1!C8 | `=REGEXEXTRACT("price 30 dollars","\d+(?= dollars)")` | "30" | #VALUE! |
| probes/regex-pcre2-features Sheet1!C9 | `=REGEXEXTRACT("a1b2","[a-z](?!1)")` | "b" | #VALUE! |
| probes/regex-pcre2-features Sheet1!C10 | `=REGEXEXTRACT("1a2b","(?<!1)[a-z]")` | "b" | #VALUE! |
| probes/regex-pcre2-features Sheet1!C13 | `=REGEXEXTRACT("foobar","foo\Kbar")` | "bar" | #VALUE! |
| probes/regex-pcre2-features Sheet1!C14 | `=REGEXEXTRACT("ab","(a)?(?(1)b\|c)")` | "ab" | #VALUE! |

## Acceptance criteria
- [ ] Visible parity: the six cells above have Excel's value, and their lines leave
      `eval/baseline/probes.txt` (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The rest of regex-pcre2-features C1-C19 that use these constructs passes.
- [ ] Directive 5: no recursion; lookbehind charged per character stepped back, barrier pops per
      frame, at T-1900's conservative placeholder costs; a release-mode test runs a nested-lookaround
      pathological pattern on 32,767 characters to `Spent` or an answer within a stated wall-clock bound.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The memoized mode for regular programs (T-2355).
- Iteration after empty matches (T-1899).
