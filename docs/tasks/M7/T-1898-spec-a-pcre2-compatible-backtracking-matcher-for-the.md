---
id: T-1898
title: "Spec: a PCRE2-compatible backtracking matcher for the REGEX functions"
milestone: M7
lane: functions-text
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A spec in docs/specs/ decides how RustSheet matches Excel's PCRE2 regular expressions: the engine (a hand-written backtracking matcher, or a crate), how it is bounded under directive 5, and how it shares work with the current lazy-DFA engine.

## Context
Found by T-1688. Excel's REGEX* functions are PCRE2: probes/regex-pcre2-features shows backreferences (`\1`, `\g1`, `\k<n>`, case-insensitive `(a)\1` matching `Aa`), lookahead and fixed-length lookbehind (`(?<=a+)` is `#VALUE!`), possessive and atomic groups, `\K`, conditionals `(?(1)b|c)`, comments, `\Q..\E`, `(?x)` and scoped inline flags `(?-i)`, all of which the `regex` crate rejects or lacks; and PCRE2's global iteration (after an empty match, a non-empty match anchored at the same position is tried first: `a*?` on `aaa` replaces to `bbbbbbb`, `|b` on `abc` to `-a---c-`; D1-D12, t1626-regexreplace C34), which Rust's leftmost-first iterator cannot express because it has no "not empty at start" search. Pattern errors are `#VALUE!` and a possessive pattern that cannot match is `#N/A` (B27-B29). The current engine (functions/text/regex_engine.rs) counts every search's work (DEVIATION, decision 0014); a backtracking matcher needs a step budget and recursion-free design (directive 5), and Excel's own limits (PCRE2 match and depth limits; what `(a|a)*b` on a long text gives) should be probed. Consider: one engine for everything versus the backtracker only for patterns or iterations that need it; Unicode classes (`[[:alpha:]]` is Unicode in Excel, T-1817); UTF-16 versus UTF-8 positions.

## Acceptance criteria
- [ ] docs/specs/ has the spec, with a decision record if it adds a dependency.
- [ ] The implementation is split into S/M tasks (T-1899 and T-1900 are rewritten or replaced to follow it).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing the matcher.
