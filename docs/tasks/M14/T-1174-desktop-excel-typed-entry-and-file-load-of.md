---
id: T-1174
title: "Desktop Excel: typed entry and file load of non-reference SUBTOTAL/AREAS arguments (follow-up of T-1163)"
milestone: M14
lane: research
status: blocked
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Know whether desktop Excel accepts (typed or loaded from a file) the SUBTOTAL/AREAS forms that Excel for the web refuses, and narrow the parse-time refusal if it does.

## Context
T-1163 (decision 0045) refuses at parse time, on excel-graph evidence only, a SUBTOTAL argument after the function number that is not a reference operand (`{1,2}`, `A1:A2*1`, `1`, `SUM(..)`, `SUBTOTAL(9,A1,)`, a nested `SUBTOTAL(9,SUBTOTAL(9,A1:A2))`) and `AREAS(1)`. See docs/excel-semantics/references.md "SUBTOTAL and AREAS take references only". Needs the excel-desktop oracle.

## Acceptance criteria
- [ ] Desktop typed-entry and file-load results recorded for the refused forms listed in references.md (at least `SUBTOTAL(9,{1,2})`, `SUBTOTAL(9,1)`, `SUBTOTAL(9,A1,)`, `SUBTOTAL(9,SUBTOTAL(9,A1:A2))`, `AREAS(1)`).
- [ ] If desktop loads any form with a value, the engine follows it (decision 0045 consequences) with a probe case.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Blocked because
Needs the excel-desktop oracle (`rustsheet-oracle status`: excel-desktop n/a). Unblock when it is available.
