---
id: T-0189
title: "Probe subnormal results of the % operator and of functions (EXP, SUM)"
milestone: M1
lane: research
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for whether `=1E-306%`, `=EXP(-745)` and a SUM of tiny products flush a subnormal result to 0, as operators do (numbers.md "Subnormal results become 0").

## Context
Follow-up to T-0179 review. The engine flushes only `+ - * / ^` and POWER (`arithmetic::finite_or_num`); the postfix `%` (model.rs `OpUnary::Percentage`) and function results are not. Probe each with `&""` companions via the excel-probe skill; file an engine task for every cell the engine gets wrong.

## Acceptance criteria
- [ ] A probe case with oracle sidecar covers `%`, EXP, SUM, PRODUCT and one more function.
- [ ] numbers.md records the rule VERIFIED per function group; engine follow-ups are filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
