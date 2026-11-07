---
id: T-4180
title: "Excel goldens for round trips: R0 re-save and R1 edit-then-save through the excel-graph oracle"
milestone: M15
lane: harness
status: blocked
priority: P2
size: M
tier: opus
depends_on: [T-4167]
spec: docs/specs/SPEC-0033-xlsx-round-trip-contract.md
---

## Goal
The excel-graph oracle produces round-trip goldens. R0: Excel re-saves the uploaded workbook. R1: Excel applies the
same edit set through a range PATCH, then saves. Both are committed as sidecars or fetched goldens, and the suite
compares our output with Excel's under S. "Excel for the web refused to open it" is recorded as a verdict of its own.

## Context
Plan: docs/research/xlsx-round-trip-2026-11-04.md §4, "Excel as arbiter". The oracle already uploads, edits (typed input, T-0280), recalculates and downloads
Excel's save (docs/architecture/oracle.md). RQ1 (what Excel for the web drops that desktop keeps) must be answered
first, for the features the probes cover. Record it in docs/excel-semantics/.

Notes: T-4177 added the cell suite kind `export` (crates/rustsheet-eval/src/export_case.rs, eval/suites/pivot-export),
where a refusal is `CaseStatus::ExcelRefused` from a sidecar's `refused` (Excel's Graph error code and message). This
task decides how SPEC-0033's `excel-refused` maps onto the round-trip `Verdict` so the two kinds do not diverge
(docs/architecture/eval-harness.md "Engine-export suites").

## Acceptance criteria
- [ ] `rustsheet-oracle` has a round-trip mode producing R0 and R1 goldens for a suite.
- [ ] RQ1 findings are in docs/excel-semantics/.
- [ ] The round-trip suite scores against the goldens where present.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Desktop goldens (M14).

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275): M15 waits until Phase 1 parity exits. Unblock when
docs/status.md has the line `**Phase 1:** exited <date>`.
