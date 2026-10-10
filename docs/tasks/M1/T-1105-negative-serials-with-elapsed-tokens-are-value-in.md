---
id: T-1105
title: "Negative serials with elapsed tokens are #VALUE! in the 1900 system; 1904 workbooks keep the minus"
milestone: M1
lane: functions-date
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-1267, T-1268]
spec: docs/specs/SPEC-0005-workbook-date-system.md
---

## Goal
TEXT and cell display of a negative serial under a format with an elapsed token match Excel: #VALUE! in the 1900 date system, the leading minus kept in a 1904 workbook.

## Context
probes/text-negative-serial-elapsed-mixed A1, A3, A5, A6 expect #VALUE! (dates.md, T-1079); the engine exempts elapsed sections in `ParsePart::Date` (formatter/format.rs) because poi ElapsedFormatTests A40-A47 is a 1904 workbook. The engine has no date-system awareness, so the fix needs the importer's date1904 flag plumbed to the formatter (design question, hence opus). Cell display (#####) under a date format is still unprobed: the case format cannot set a cell number format, so that needs a harness extension first.

Design: SPEC-0005 (rollout step 3), split out by T-1266.

## Acceptance criteria
- [ ] probes/text-negative-serial-elapsed-mixed passes without regressing poi ElapsedFormatTests A40-A47.
- [ ] Cell display of a negative under a date format is probed (xlsx case with a number format) and recorded in dates.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

Note (T-1079 review): poi A40-A47 only evidence elapsed/time formats in 1904, never `d`/`mmm`/`yyyy` tokens; probe a 1904 mixed-token format before generalizing, and add `TEXT(-1E-10,"[h]:mm")` and `TEXT(-0,"[h]:mm")` to the 1900 probes.
