---
id: T-4944
title: "Ratchet engine-export cases' rowItems/colItems verdicts in the baseline"
milestone: M16
lane: harness
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
An engine-export case whose engine `rowItems`/`colItems` are not Excel's fails in the baseline ratchet, as its cells do, instead of only being listed in report.md's "Pivot layout" section.

## Context
T-4177 compares the lines the engine saves (`pivot_scoring::saved_lines`) with the sidecar's, verdict `LinesDiffer`, but only reports it: the cells-mode baseline holds `Sheet!A1` items, and a line item would need a key that `exit_owners`, `new_case_owners` and the visible-parity lock read too. Today t4177-api-spill-refresh (Blocked) and t4177-spike (K, S) differ. Design the item key with T-4180, which extends this suite kind to the general excel-graph round-trip goldens (SPEC-0033 X3).

## Acceptance criteria
- [ ] A lines mismatch of an export case is a baseline item, a new one a regression, with tests.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
