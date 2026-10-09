---
id: T-1213
title: "Implement DBCS and PHONETIC as the web oracle evaluates them"
milestone: M5
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
DBCS and PHONETIC are implemented as excel-graph evaluates them: DBCS of ASCII text returns it unchanged, PHONETIC of a reference is `#N/A`.

## Context
From T-1087's probes: probes/text-modes-dbcs C3 (`DBCS("abc")` = "abc") and probes/text-modes-phonetic, -range, -number C1 (`#N/A` for a cell, a range and a number cell) are `#NAME?` in the engine. A literal argument (`PHONETIC("abc")`) makes excel-graph refuse the workbook, so it is unobservable (docs/excel-semantics/text.md "Text function argument modes"). Check DBCS for full-width conversion of katakana/ASCII before implementing more than the ASCII case (excel-probe).

## Acceptance criteria
- [ ] The four probe cells pass; any DBCS conversion rule beyond them comes with a probe.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
