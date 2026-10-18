---
id: T-0434
title: "Volatile-cell detection follows structured table references"
milestone: M11
lane: harness
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The eval harness treats a formula as volatile when it reaches TODAY/NOW through a structured table
reference (`T_INV3[Status]`), so clock-dependent cells are skipped instead of scored.

## Context
corpus-bench/118-8 (6 failing cells): `Invoice Payments!D5`, F5, D6, F6, L9 (and one more) such as
`=IFERROR("INVOICES: "&COUNTIFS(T_INV3[Status],"PAST DUE",T_INV3[Selected],1),"")` read table columns
whose cells (J16:J23 `Status`, K16:K23 `Past Due Age`) compare due dates with the defined name
`TD` = `'Invoice Payments'!$C$2` = `TODAY()`. Those column cells are already skipped as volatile
(19 skipped in the case), but their dependents through `T_INV3[...]` are scored, because
`precedents` in crates/rustsheet-eval/src/volatile.rs only follows A1 references and defined names.
Expected (Excel, run 2026-10-17) "INVOICES: 6" PAST DUE, engine 0, which is clock-dependent.
The snapshot must expose table definitions (name, ref, column names) if it does not yet.
While here, note in the Outcome whether the engine's J16:J23 statuses for today are plausible (the
engine counts the six invoices as CURRENT, although their due dates are in 2023); if not, file an engine task.

## Acceptance criteria
- [x] A unit test in volatile.rs: a cell reading `Table[Col]` whose column cells call TODAY() is volatile; `Table[[#This Row],[Col]]` too.
- [x] corpus-bench/118-8's `F` line leaves eval/baseline/corpus-bench.txt (its 6 cells become skipped_volatile), with no new `F` lines in any suite.
- [x] `make check` green; reviewers approve.

## Out of scope
- Following INDIRECT/OFFSET (documented as not followed).

## Outcome
The snapshot now reads table parts (`TableDef`: name, sheet, ref, header/totals rows, columns) and
`volatile.rs` adds `Table[Col]` / `Table[[#This Row],[Col]]` as precedents over the table's data
rows (a `[A]:[B]` span or an unmatched column takes the whole body: over-approximation only).
Eval delta: corpus-bench 26 -> 25 known failures (118-8 `F` line gone; its C count 294 -> 282 and
52541's 15 -> 10 cells, now skipped as volatile); no new `F` lines in any suite.
Not assessed: whether the engine's J16:J23 statuses for today are plausible (the cells are now
skipped, so the question no longer affects scoring); no engine task filed.
