---
id: T-1688
title: "REGEXREPLACE and REGEXTEST: backreferences and lazy empty matches (t1626 probes)"
milestone: M7
lane: functions-text
status: todo
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
- [ ] REGEXREPLACE C18, C34 and REGEXTEST C29 pass; baseline shrinks.
- [ ] Cells left pending (invalid patterns, `$3`) are re-filled once the oracle decodes the error type.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The optional-argument cells (T-1360).

## Handoff
Sonnet run read the task and the engine's regex setup (`regex-automata` with `nfa-pikevm`, `regex-lite`, `regex`; none supports backreferences). Pattern backreferences (REGEXTEST C29 `(\w)\1`) and PCRE2-style lazy empty-match global replace (C34) need either a backtracking engine (a new dependency: supply-chain review, directive 5 limits on backtracking time and memory) or a hand-written matcher, which is a design question beyond a sonnet run. The `\1` replacement form (C18) alone is small and could be split out. Next: decide the engine (with a decision record and a step/memory cap), or split C18 into its own sonnet task. Nothing is implemented.
