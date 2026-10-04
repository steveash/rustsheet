---
id: T-0013
title: "Oracle: canary-verify every recalculated workbook; force Excel to save the session"
milestone: M0
lane: research
status: done
priority: P0
size: M
depends_on: []
---

## Goal
No oracle result can be the cached values we uploaded: every workbook sent to an oracle backend
proves, inside itself, that Excel recalculated it.

## Context
Microsoft documents the Graph workbook endpoints as delegated-only, and uploading an xlsx does
not recalculate it, so an app-only oracle that silently stopped calculating would return
plausible-looking cached values. The owner asked for a canary embedded in every workbook (not a
separate probe run), so that per-workbook differences cannot slip through.

## Acceptance criteria
- [x] Every workbook recalculated by `rustsheet-oracle recalc`/`fill` carries a canary sheet with
      poisoned cached values; results are refused unless all canary cells hold Excel's answers;
      the canary sheet never reaches a sidecar; cached results are re-verified on read.
- [x] Workbooks whose answers the extra sheet could change (`SHEETS()`, `INFO()`) are refused.
- [x] Any defect the canary exposes in the live Graph flow is fixed and demonstrated live.
- [x] docs/architecture/oracle.md records the trust model, the evidence and the pitfalls.
- [x] `make check` green; reviewers approve.

## Out of scope
Filling any suite (T-0004 smoke, T-0009 probes, T-0203 gen-m2); the Windows desktop backend.

## Outcome
The canary immediately caught a real defect: for workbooks last saved by current Excel (every
ironcalc and most POI workbooks), Excel for the web computes in-session but `closeSession` saves
nothing, and the old flow downloaded the upload after a silent 60 s timeout. The Graph backend
now makes a no-trace edit (add + delete a scratch sheet) after `calculate`, and raises if the
eTag never moves. Verified live on 2026-10-04: ironcalc `arithmetic.xlsx` and `DAVERAGE.xlsx`
and smoke spec cases pass the canary (~10 s each); upload-only and no-edit controls fail it.
Offline: the canary embeds into 568 of 591 suite workbooks with case values unchanged (refusals:
2 `SHEETS`/`INFO` workbooks, 21 corrupt fuzzer/backslash-path packages). Eval delta: none.
