---
id: T-2015
title: "REGEX* reach scan adds states in priority order so lazy patterns iterate in linear charge"
milestone: M7
lane: functions-text
status: done
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
- [x] The formula above returns a value; a test pins it, and one that the matches are unchanged.
- [x] `a.*z|a|\bq` iteration over `REPT("a",20000)&"é"` is still `#ERROR!` within about 1 s.
- [x] `make check` green; reviewers approve.

## Out of scope
- The look-around charge (T-1829).

## Outcome
`Reach::scan` now runs the `PikeVM`'s own search: its sets hold consuming and match states in
the `PikeVM`'s priority order (depth-first closure, first alternate first; the thread started at
a position after those running), and a match state at a character boundary ends the step and
drops the threads below it, as `nexts` does. An empty match inside a character (only searches
that start inside one, after an empty match, meet it) turns the cut off, so the retries of
`skip_splits_fwd` stay inside the span; the module doc carries the argument. The scan also
returns at once for a search that starts past the end (look-around there would have panicked;
latent on main, where the DFA answered such searches first).

- `COLUMNS(REGEXEXTRACT(REPT("é ab ",4000),"\bab.*?\b",1))` is 4000 (was `#ERROR!`).
- `a.*z|a|\bq` over `REPT("a",20000)&"é"` is still `#ERROR!` (0.8 s in a release test run).
- A seeded sweep (3000 random patterns/texts in the unit test; 60,000 run once in release)
  finds the same matches iterating over the scanned spans as over the rest of the text; it
  catches a reversed alternate order and a cut at split matches.
- Eval: no delta (no suite case exercises the `PikeVM` fallback at this size).
