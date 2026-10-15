---
id: T-1586
title: "FIND, SEARCH, SUBSTITUTE with a lone-surrogate find_text (they split pairs) and CODE/VALUE of lone surrogates match Excel"
milestone: M2
lane: functions-text
status: todo
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
