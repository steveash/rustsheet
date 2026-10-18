---
id: T-1688
title: "REGEXREPLACE and REGEXTEST: backreferences and lazy empty matches (t1626 probes)"
milestone: M7
lane: functions-text
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
REGEXREPLACE and REGEXTEST match Excel on the t1626 probe cells that T-1360 does not cover.

## Context
Found by T-1626 (probes/t1626-regexreplace, t1626-regextest, excel-graph). Beyond the optional-argument #ERROR! cells (T-1360): a pattern backreference `(\w)\1` matches (REGEXTEST C29); a `\1` in the replacement is group 1, like `$1` (REGEXREPLACE C18); the lazy empty-matching `a*?` on "aaa" gives "bbbbbbb" (C34). (A range argument already lifts and spills like Excel: C35 and REGEXTEST C31 pass.) See docs/excel-semantics/text.md ("REGEXTEST and REGEXREPLACE edge cases").

## Acceptance criteria
- [x] REGEXREPLACE C18 passes; baseline shrinks.
- [x] C29 (pattern backreference) and C34 (lazy empty matches) need a PCRE2-style matcher, which is architectural
  (CLAUDE.md directive 4): split into T-1898 (spec), T-1899 (iteration) and T-1900 (pattern features), with the
  Excel evidence they need probed here (probes/regex-pcre2-features).
- [x] Pending error cells decoded through ERROR.TYPE: invalid replacements and patterns are `#VALUE!`.
- [x] `make check` green; reviewers approve.

## Out of scope
- The optional-argument cells (T-1360).

## Handoff (resumed)
Sonnet run read the task and the engine's regex setup (`regex-automata` with `nfa-pikevm`, `regex-lite`, `regex`; none supports backreferences). Pattern backreferences (REGEXTEST C29 `(\w)\1`) and PCRE2-style lazy empty-match global replace (C34) need either a backtracking engine (a new dependency: supply-chain review, directive 5 limits on backtracking time and memory) or a hand-written matcher, which is a design question beyond a sonnet run. The `\1` replacement form (C18) alone is small and could be split out. Next: decide the engine (with a decision record and a step/memory cap), or split C18 into its own sonnet task. Nothing is implemented.

## Outcome
The REGEXREPLACE replacement is parsed as Excel's PCRE2 extended syntax (`Replacement::new` in
functions/text/regex_functions.rs): `\n` is group n, `$n` takes every following digit (`$1a` is group 1 then "a"),
`\$` and `\\` escape, `\n`/`\t` are LF/TAB, and a missing group, a lone `$`, a trailing `\`, `\0` or an unknown
letter escape is `#VALUE!` once something matches (with no match the text is unchanged). New probe probes/regex-pcre2-features (62 scored cells, excel-graph) records the replacement
syntax, the PCRE2 pattern features and PCRE2's global iteration (text.md "REGEXREPLACE replacement syntax (T-1688)"
and "REGEX* patterns and match iteration are PCRE2 (T-1688)"). Eval: t1626-regexreplace C18 fixed; the new case
passes 32 of 62, its 30 failures are T-1897 (`\U` case forcing), T-1898/T-1899/T-1900 (the PCRE2 matcher) and
T-1360 (optional arguments) and the surrogate-pair empty match (D13, T-1899). Follow-ups: T-1897, T-1898, T-1899, T-1900.
