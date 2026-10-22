---
id: T-2355
title: "REGEX functions: memoized backtracker mode for regular programs, the final-LF dollar and the differential test"
milestone: M7
lane: functions-text
status: in-progress
priority: P2
size: M
tier: opus
depends_on: [T-1900]
---

## Goal
Regular patterns can run on the backtracker in linear time, so that `$` and `\Z` before a final
`\n` are exact on every text. A differential test pins the backtracker to the automata.

## Context
Split from T-1900. SPEC-0013 section 3 ("Memoization ... a separate memoized mode") and section 2
("`$` and `\Z`"): a text that ends in `\n` and holds another `\n` cannot use either automata
translation. Until this task lands, `pcre::final_lf_for` returns `FinalLf::End` there (today's `\z`),
and a unit test in `pcre/tests.rs` pins that gap.
- The memoized program is compiled from the regular AST Thompson-style. Counted repeats are expanded
  and charged to the compile budget (`TooBig` as for the NFA). It has no empty-loop slot.
- The visited bit-set of `(pc, pos)` lives in `Caches`, with generation stamps. It is allocated in
  chunks and charged to held memory. When it would not fit what the call may still hold, the call is
  `#ERROR!`, recorded as a `DEVIATION` (decision 0014) unless T-1972 shows Excel failing there too.
- Section 3 states the rule for keeping marks across one iteration's searches: after a match ending
  at `e`, positions at or before `e` are unvisited again.
- T-1899's empty-match retry (section 4, step 2) runs on this mode for regular patterns.

## Acceptance criteria
- [ ] `$` and `\Z` match before a final `\n` on a text that holds another `\n`, from the automata
      translation's semantics (probes/regex-ascii-classes A34 family). The pinning test in
      `pcre/tests.rs` flips.
- [ ] Every regular program on the backtracker runs in memoized mode, counter-free and guard-free.
      Its bit-set is allocated only if it fits what the call may still hold, otherwise `#ERROR!`.
      `(a|a)*b$` over a 32,767-character text ending in `CHAR(10)` runs in linear time (release-mode
      test with a stated wall-clock bound).
- [ ] Differential unit test: random regular patterns give the same match and capture spans on the
      automata and on the backtracker (memoized mode and T-1900's plain mode).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Lookaround, conditionals, `\K` (T-2354).
