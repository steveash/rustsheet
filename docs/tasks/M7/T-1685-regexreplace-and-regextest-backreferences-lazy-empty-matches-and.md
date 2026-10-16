---
id: T-1685
title: "REGEXREPLACE and REGEXTEST: backreferences, lazy empty matches and range lift (t1626 probes)"
milestone: M7
lane: functions-text
status: todo
priority: P2
size: M
tier: sonnet
depends_on: []
---

## Goal
REGEXREPLACE and REGEXTEST match Excel on the t1626 probe cells that T-1360 does not cover.

## Context
Found by T-1626 (probes/t1626-regexreplace, t1626-regextest, excel-graph). Beyond the optional-argument #ERROR! cells (T-1360): a pattern backreference `(\w)\1` matches (REGEXTEST C29); a `\1` in the replacement is group 1, like `$1` (REGEXREPLACE C18); the lazy empty-matching `a*?` on "aaa" gives "bbbbbbb" (C34); a range argument lifts and spills (`#SPILL!` when blocked, C35; REGEXTEST C31) where the engine gives #VALUE!. See docs/excel-semantics/text.md ("REGEXTEST and REGEXREPLACE edge cases").

## Acceptance criteria
- [ ] REGEXREPLACE C18, C34, C35 and REGEXTEST C29, C31 pass; baseline shrinks.
- [ ] Cells left pending (invalid patterns, `$3`) are re-filled once the oracle decodes the error type.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The optional-argument cells (T-1360).
