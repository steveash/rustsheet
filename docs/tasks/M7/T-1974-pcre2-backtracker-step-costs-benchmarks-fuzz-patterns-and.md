---
id: T-1974
title: "PCRE2 backtracker step costs, benchmarks, fuzz patterns and the match-limit answer"
milestone: M7
lane: perf
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-1900, T-1972]
---

## Goal
The PCRE2 backtracker's work is charged at fitted costs, has benchmarks and fuzz coverage, and answers Excel's
match-limit cases as the probe shows (SPEC-0013 rollout step 5).

## Context
SPEC-0013 section 3 ("Budget") leaves `STEP` and `FRAME` to be fitted against the T-1272 unit (about 1 ns of a
release build; functions/text/regex_engine.rs). T-1972 records what Excel gives for catastrophic patterns
(probes/regex-pcre2-limits).

## Acceptance criteria
- [ ] `STEP` and `FRAME` are measured. Benchmarks in crates/rustsheet/benches cover: a backreference over 10,000
  cells, lookahead over a 32,767-character text, and the empty-match retry of `a*?`. The T-1272 cases do not move.
- [ ] Catastrophic patterns on 32,767-character texts finish in bounded time. They give Excel's probed answer, or
  `#ERROR!` with a `DEVIATION` entry in text.md (decision 0014) where Excel answers past our budget.
- [ ] The memoized mode's refusal (`#ERROR!` when its bit-set would pass the call's held memory, SPEC-0013 section 3)
  is a `DEVIATION` entry in text.md (decision 0014), unless T-1972 shows Excel failing there too.
- [ ] The fuzz `formula` target generates REGEX* calls with PCRE2 patterns.
- [ ] REGEXREPLACE over 100,000 cells with a pattern that never matches empty is no slower than before the empty-match
  retry (T-1899). `\b(\w+)\s+\1\b` over a 100,000-row column is benchmarked (start-position prefilter).
- [ ] `make check` green; reviewers approve.

## Out of scope
- New pattern features.
