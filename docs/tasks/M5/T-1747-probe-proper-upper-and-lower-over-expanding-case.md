---
id: T-1747
title: "Probe PROPER, UPPER and LOWER over expanding case mappings and final sigma variants"
milestone: M5
lane: functions-text
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
PROPER, UPPER and LOWER match Excel on characters whose full Unicode case mapping expands and on final-sigma variants.

## Context
T-1307 (probes/clean-proper-chars) found Excel uses one-to-one mappings (ß, ŉ, ǰ, ﬁ unchanged; İ to I; ΐ to Ϊ) and
a final ς for a word-final capital Σ in PROPER (docs/excel-semantics/text.md "CLEAN character set and PROPER ..."). Unprobed:
ᾳ, ẖ, ẘ, ﬀ, ﬃ, ﬆ, և; lower-case σ at a word end ("ασ"); Σ before a combining mark or an apostrophe; UPPER and LOWER of all
of these. Code: `simple_case` in functions/text/string_format.rs, UPPER/LOWER in functions/text/common.rs.

## Acceptance criteria
- [ ] A probe case (excel-probe skill) with those cells; the engine matches each or each mismatch has its own task.
- [ ] text.md updated, scoping the T-1307 rule to the probed characters.
- [ ] `make check` green; reviewers approve.

## Out of scope
- CLEAN (T-1307 covered it).
