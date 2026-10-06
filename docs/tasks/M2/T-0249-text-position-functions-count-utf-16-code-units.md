---
id: T-0249
title: "Text position functions count UTF-16 code units (LEN, LEFT, MID, RIGHT, FIND, SEARCH, REPLACE)"
milestone: M2
lane: functions-text
status: todo
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
and E7/E11 (LEN of a 32,767-unit string with emoji is 32,767); see coercion.md "Text longer than
32,767 characters". `len_element` and the LEFT/RIGHT/MID helpers in
`functions/text/common.rs` count `chars()`. The 32,767 limit itself already counts UTF-16 units
(`cast::truncate_text`, REPT; T-0233). Open question for a probe first: what LEFT/MID/RIGHT
return when a position splits a surrogate pair (Excel keeps a lone surrogate, which a Rust
`String` cannot hold; E10 shows `UNICODE` of a lone high surrogate is `#VALUE!`).

## Acceptance criteria
- [ ] Probe (excel-probe) LEFT/MID/RIGHT/FIND/SEARCH/REPLACE positions around a surrogate pair; oracle values recorded.
- [ ] LEN and the position-based text functions count UTF-16 code units; probes/concatenate-limit-spill E7, E11, E13 and G97 pass.
- [ ] A split surrogate pair is handled one consistent way, documented as a known difference in docs/excel-semantics/text.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
