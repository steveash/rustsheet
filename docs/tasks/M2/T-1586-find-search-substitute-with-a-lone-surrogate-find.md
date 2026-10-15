---
id: T-1586
title: "FIND, SEARCH, SUBSTITUTE with a lone-surrogate find_text (they split pairs) and CODE/VALUE of lone surrogates match Excel"
milestone: M2
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
depends_on: [T-1580]
---

## Goal
FIND/SEARCH with a lone half as find_text and SUBSTITUTE with a lone-half old_text can split a pair; CODE is 63 and VALUE errors for lone surrogates.

## Context
probes/text-lone-surrogates A1-A2, A41-A60. Needs the exact `Text` (SPEC-0010).

## Acceptance criteria
- [ ] probes/text-lone-surrogates A1-A2 and A41-A60 pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
FIND, SEARCH and SUBSTITUTE keep a lone surrogate as its own unit (SEARCH folds case only in well-formed runs; SUBSTITUTE with any lone operand works over UTF-16 units, result length checked before building). `substitute_element` moved from common.rs to util.rs (common.rs was at the 1000-line limit). Eval: probes/text-lone-surrogates A43, A48-A51, A53, A55-A57 now pass (9 cells); The other cells (A1-A2, A58-A60: CODE, SEARCH, VALUE; A41-A42, A44-A47, A52, A54) already passed. With an `instance_num` the lone-surrogate SUBSTITUTE counts overlapping instances like the well-formed path (unprobed in both; text.md).
