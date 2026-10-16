---
id: T-1595
title: "cmp_text_excel orders the T-1590 probed characters (controls, dashes, symbols, non-Latin-1 letters)"
milestone: M2
lane: engine
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
`cmp_text_excel` ranks the characters probed in T-1590 as Excel does, so probes/text-collation-unprobed passes.

## Context
Split from T-1594 (its criteria half is done): the 89 failing rank cells of `probes/text-collation-unprobed`. Observed order, equalities (`ĳ`=`ij`) and caveats are in docs/excel-semantics/coercion.md "Collation equality in lookups and criteria; unprobed characters". Needs a fit, not a table copy: the control characters CHAR(1), CHAR(31), CHAR(127) sort above the empty string and below the en and em dash, and those below CHAR(160), while CHAR(9), CHAR(10) and CHAR(13) sort above CHAR(160) (so the control range is not one block, and the dashes behave like the hyphen only inside a longer text); the Latin-1 symbols, quotes and currency signs fall between the punctuation and the digits in a given order; `¼ ½ ¾ ¹ ² ³` sit between `0` and `9`; `ł đ ð þ ı œ` and Greek follow their base letters. Where the evidence is single-character only, record the choice made and keep it a minimal rule. Code: functions/util/collation.rs.

## Acceptance criteria
- [ ] All cells of probes/text-collation-unprobed pass; baseline shrinks (`make eval-baseline`).
- [ ] No regression in probes/text-collation-order and probes/text-order-nbsp-space.
- [ ] coercion.md fitted-key paragraph updated (UNVERIFIED notes removed where now evidenced).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Characters not in the T-1590 probes.
