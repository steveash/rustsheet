---
id: T-2015
title: "REGEX* reach scan adds states in priority order so lazy patterns iterate in linear charge"
milestone: M7
lane: functions-text
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-1829]
---

## Goal
REGEX* iteration of a lazy pattern on the `PikeVM` fallback is charged about the work it does:
`COLUMNS(REGEXEXTRACT(REPT("é ab ",4000),"\bab.*?\b",1))` returns a value (as in Excel) and stays
fast, while every search stays bounded (CLAUDE.md directive 5).

## Context
T-1829 performance review. `Reach::scan` (functions/text/regex_reach.rs) keeps every thread until
none is left, including the lower-priority threads the `PikeVM` drops once a higher-priority one
matches (its `nexts` cuts them). A lazy `.*?` keeps a lower-priority greedy thread alive to the
end of the text, so each search's reach is the rest of the text and iteration is quadratic in its
charge: the formula above is `#ERROR!`, as on main before T-1829. Add the closure's states in
priority order (a stack, as the `PikeVM`'s `epsilon_closure` does) and stop following the threads
below one that has matched; the soundness argument in the module doc must then cover the cut.

## Acceptance criteria
- [ ] The formula above returns a value; a test pins it, and one that the matches are unchanged.
- [ ] `a.*z|a|\bq` iteration over `REPT("a",20000)&"é"` is still `#ERROR!` within about 1 s.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The look-around charge (T-1829).
