---
id: T-1181
title: "Probe 3-D span with both ends missing, CHOOSE-wrapped span in COUNTBLANK, deleted span end"
milestone: M1
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence for 3-D span forms T-1170 left unprobed, with the engine matching it.

## Context
T-1170 maps a span whose prefix has an unresolved end to `#NAME?` (`Model::wrong_sheet_error`) and follows only IF in `is_span_or_if_of_span`. Unprobed: `Nope1:Nope2!A1`, a span that lost a sheet when a sheet was deleted, `COUNTBLANK(CHOOSE(1,Data1:Data3!A1:B3))`.

## Acceptance criteria
- [ ] Probe the three forms with the excel-probe skill; the engine matches or the mismatch is fixed; references.md 3-D span entry updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
