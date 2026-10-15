---
id: T-1626
title: "Probe the byte and East-Asian text functions SEARCHB, FINDB, REPLACEB, RIGHTB, LENB, ASC, JIS, PHONETIC, REGEXREPLACE, REGEXTEST to the evidence floor"
milestone: M5
lane: research
status: todo
priority: P1
size: S
tier: sonnet
depends_on: [T-0056]
---

## Goal
Bring each of these in-scope functions to at least 20 Excel-evidence cells (the evidence floor,
T-0056): SEARCHB (3), FINDB (7), REPLACEB (7), RIGHTB (8), LENB (11), ASC (10), JIS (11), PHONETIC (19), REGEXREPLACE (14), REGEXTEST (18) cells today (`make eval`, report "Below the evidence floor", 2026-10-15).

## Context
docs/decisions/0062 (evidence-floor ratchet); `excel-probe` skill. Write one or more spec probe
cases under eval/suites/probes/cases/, fill them with the oracle, and cover every parameter mode
of each function (catalog signatures in eval/catalog/functions.yaml) and the catalog's edge
pools (tools/src/rustsheet_tools/gen/generator.py "Edge-value pools"). Edge cases to include:
en-US byte functions on double-width text and surrogate pairs, start/num_bytes at 0, past the end and fractional, full-width/half-width katakana and ASCII for ASC/JIS, PHONETIC on plain cells and ranges, REGEX* case_sensitivity, occurrence (0, negative, past the last match), invalid patterns, capture-group references. Record every quirk found in docs/excel-semantics/ and file a fix task for each mismatch
(`make new-task`); fixing the engine is out of scope.

## Acceptance criteria
- [ ] Each listed function has ≥ 20 Excel-evidence cells in the report.
- [ ] `make eval-baseline` lowers quality/evidence-floor.json by the number of functions lifted
  over the floor; new failing cells are recorded as new-coverage failures with fix tasks.
- [ ] Quirks recorded in docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine fixes for the mismatches found (filed as tasks).
