---
id: T-2355
title: "REGEX functions: memoized backtracker mode for regular programs, the final-LF dollar and the differential test"
milestone: M7
lane: functions-text
status: done
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
- [x] `$` and `\Z` match before a final `\n` on a text that holds another `\n`, from the automata
      translation's semantics (probes/regex-ascii-classes A34 family). The pinning test in
      `pcre/tests.rs` flips.
- [x] Every regular program on the backtracker runs in memoized mode, counter-free and guard-free.
      Its bit-set is allocated only if it fits what the call may still hold, otherwise `#ERROR!`.
      `(a|a)*b$` over a 32,767-character text ending in `CHAR(10)` runs in linear time (release-mode
      test with a stated wall-clock bound).
- [x] Differential unit test: random regular patterns give the same match and capture spans on the
      automata and on the backtracker (memoized mode and T-1900's plain mode).
- [x] `make check` green; reviewers approve.

## Out of scope
- Lookaround, conditionals, `\K` (T-2354).

## Outcome
- `pcre::program::compile_memoized` compiles a regular AST guard-free and counter-free, laying out
  unbounded repeats as `regex_automata`'s `c_at_least` does (`x*` as `(?:x+)?` when `x` can match
  empty). The backtracker keeps a visited bit-set of `(pc, pos)` in `BacktrackCache` (chunks of 64
  positions, one word per instruction, generation stamps; reset per operation by
  `Backtracker::new`), held within the call's room with the stack, `#ERROR!` past it (DEVIATION
  in text.md). After a match ending at `e`, marks at positions up to `e` are forgotten; later ones
  stay (SPEC-0013 section 3).
- `FinalLf::End` became `FinalLf::InnerLf`: a regular pattern holding `$` or `\Z` over a text with
  a `\n` before its last character now runs memoized, so `$` matches before the final `\n` there
  too. The pinning test flipped. Other patterns stay on the automata.
- Differential test `backtracker_agrees_with_the_automata_on_regular_patterns`: 3,000 random
  regular patterns × 11 texts, same match, groups and iteration on automata, memoized mode and
  (for patterns without an empty-matching loop body) plain mode. Mutations of the forget rule and
  of the `(x+)?` layout each fail it.
- Release timings over `"a"&CHAR(10)&REPT("a",32764)&CHAR(10)`: `(a|a)*b$` 4.6 ms,
  `(a|a)*a$` 2.0 ms, REGEXREPLACE `(a|a)*b$|a` 6.5 ms (`memoized_mode_is_linear`, 1 s bound).
- Eval delta: none expected (no eval case holds `$` over a text with an inner `\n`); see the PR.
- T-1899 now depends on T-2355 too (its retry runs on this mode).
