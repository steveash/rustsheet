---
id: T-0902
title: "CHAR text mismatches"
milestone: M9
lane: functions
status: done
priority: P2
size: S
depends_on: []
---

## Goal
Fix the 29 CHAR cells that differ from Excel (likely code-page 1252 mapping for 128–159).

## Context
`make eval-show CASE=ironcalc-calc/TEXT/...` (find via report).

## Acceptance criteria
- [x] CHAR known failures → 0 (mapping table n/a: root cause was the harness, see Outcome).
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
The 29 mismatches were not a CHAR bug: Excel stores control characters in `<v>` as `_xHHHH_` and the
eval snapshot reader (expected side) did not decode them, while the xlsx importer did. The snapshot
reader now reuses `rustsheet_xlsx::import::decode_xlsx_escapes` (made `pub`) for shared, inline and
`t="str"` text. ironcalc-calc known failures 1909 -> 1880 (fixed 29, 0 regressions). `smoke/d-function-criteria`
was also recorded in the smoke baseline's coverage list by `make eval-baseline`.
