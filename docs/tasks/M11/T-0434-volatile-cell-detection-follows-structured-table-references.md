---
id: T-0434
title: "Volatile-cell detection follows structured table references"
milestone: M11
lane: harness
status: in-progress
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
- [ ] A unit test in volatile.rs: a cell reading `Table[Col]` whose column cells call TODAY() is volatile; `Table[[#This Row],[Col]]` too.
- [ ] corpus-bench/118-8's `F` line leaves eval/baseline/corpus-bench.txt (its 6 cells become skipped_volatile), with no new `F` lines in any suite.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Following INDIRECT/OFFSET (documented as not followed).
