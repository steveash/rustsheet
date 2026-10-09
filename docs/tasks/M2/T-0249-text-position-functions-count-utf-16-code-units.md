---
id: T-0249
title: "Text position functions count UTF-16 code units (LEN, LEFT, MID, RIGHT, FIND, SEARCH, REPLACE)"
milestone: M2
lane: functions-text
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Text functions that count or index characters do so in UTF-16 code units, as Excel does, so an
emoji or other non-BMP character counts two (`LEN("😀b")` is 3).

## Context
Found by T-0233. Evidence: probes/concatenate-limit-spill E13 (`LEN(CONCATENATE("😀","b"))` = 3)
and E7 (LEN of a 32,767-unit string with emoji is 32,767); see coercion.md "Text longer than
32,767 characters". `len_element` and the LEFT/RIGHT/MID helpers in
`functions/text/common.rs` count `chars()`. The 32,767 limit itself already counts UTF-16 units
(`cast::truncate_text`, REPT; T-0233). Open question for a probe first: what LEFT/MID/RIGHT
return when a position splits a surrogate pair. Excel keeps a lone surrogate, which a Rust
`String` cannot hold; decision 0033 says the engine drops the whole pair instead (E8–E11 stay
known failures for that reason, not this task's).

## Acceptance criteria
- [x] Probe (excel-probe) LEFT/MID/RIGHT/FIND/SEARCH/REPLACE positions around a surrogate pair; oracle values recorded.
- [x] LEN and the position-based text functions count UTF-16 code units; probes/concatenate-limit-spill E7, E13 and G97 pass.
- [x] A position that splits a surrogate pair drops the whole pair (decision 0033), documented in docs/excel-semantics/text.md as `Status: DEVIATION (decision 0033)`.
- [x] `make check` green; reviewers approve.

## Outcome
Probe probes/text-surrogate-positions (68 cells, excel-graph) shows LEN, MID, FIND, SEARCH,
REPLACE and the *B twins count UTF-16 code units (a SEARCH `?` matches one unit), while LEFT and
RIGHT count code points (`LEFT("a😀b",3)` is "a😀b"), so LEFT/RIGHT stay char-based. FIND also
accepts a start one past the end for "" (`FIND("","abc",4)` is 4); SEARCH does not. New helpers
`utf16_slice` and `one_char_per_utf16_unit` (functions/text/util.rs). Split pairs are dropped
whole (decision 0033; A6, A11, A12, A29, A38, B10 stay known failures). Eval: probes fixed 4
(concatenate-limit-spill E7, E13, G97; text-result-limit C24), 6 new-coverage deviation failures,
0 regressions. Docs: text.md (two entries), coercion.md. Follow-up: T-1296.

## Out of scope
-
