---
id: T-3026
title: "Probe IMSUB/IMDIV with an omission passed through two LET or LAMBDA bindings (T-2770 follow-up)"
milestone: M9
lane: functions-engineering
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
IMSUB and IMDIV match Excel when an omitted LAMBDA parameter or empty LET value is passed on through a second binding, or the mismatch is recorded.

## Context
T-2770 made `complex_pair_of` (complex/mod.rs) raise `#N/A` for a variable bound to an omitted value (`is_omitted_variable`, `bind_argument` hands `Binding::Omitted` on). Unprobed: `LET(e,,LET(f,e,IMSUB("1",f)))` and a LAMBDA parameter forwarded to an inner LAMBDA (T-2770 fidelity review). Probe them (excel-probe skill), match, and update engineering.md "T-1625 probes".

## Acceptance criteria
- [ ] A probe case with oracle values for the shapes above; the engine matches them or each mismatch is recorded.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
