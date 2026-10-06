---
id: T-0994
title: "Probe error precedence for text elements and coercion errors next to an error argument (WEEKDAY, MID)"
milestone: M1
lane: research
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: [T-0129]
---

## Goal
Oracle evidence settles two error-precedence sub-rules that T-0129 implemented without a probe.

## Context
T-0129 review (excel-fidelity, minor). docs/excel-semantics/arrays.md "Errors broadcast through &,
comparisons, POWER and the element-wise functions" marks both as UNVERIFIED:
1. `fn_weekday` (functions/date_and_time.rs): with an error return type and an array of dates, a
   non-numeric text element gives `#VALUE!` (the engine's choice), not the return type's error.
   Probe `WEEKDAY({"x",1,TRUE,""},NA())` and a range with text, booleans and blanks.
2. `broadcast_text` (functions/text/common.rs): with all-scalar arguments the first error argument is
   returned before coercing the others, so `MID("abc","x",NA())` is `#N/A`, while the array form
   `MID({"abc"},"x",NA())` is `#VALUE!`. Probe both, plus LEFT/RIGHT/REPT/FIND equivalents.

## Acceptance criteria
- [ ] A probe case under eval/suites/probes/cases, oracle-filled (excel-probe skill).
- [ ] arrays.md updated to VERIFIED with the rule Excel shows; a follow-up task filed if the engine differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing a fix (file it separately).
