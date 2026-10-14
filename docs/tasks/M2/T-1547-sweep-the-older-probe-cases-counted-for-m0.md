---
id: T-1547
title: "Sweep the older probe cases counted for M0-M2 for later-milestone subjects"
milestone: M2
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Every probe case and cell counted for M0-M2 has an M0-M2 subject, or a recorded reason under decision 0051's exception, so no later-milestone probe weighs on M2 exit.

## Context
T-1470 (decision 0058) mapped the post-0051 cases. A script sweep in its review (every probes case tagged M0/M1/M2 or untagged, not in probe-milestones.tsv or m1-probes.txt, whose called functions other than observers are all above M2 in eval/catalog/scope.json) still flags about 80 cases. Most are false hits from an incomplete observer list or are 0051 exceptions: clock-text-*, multiarea-* and reference operators are M2 subjects. Candidates to check: `multiarea-foreach` / `-more` / `-order-stats` (statistical functions over unions), `rich-errors-*` (IMAGE, STOCKHISTORY, FIELDVALUE), `ifs-shape-mismatch`, `ecma-ceiling*`, `text-general-ties-*`, `timevalue-*`, `union-walk-repeated-in-lambda`, `percentof-snap-overflow-union`, `if-index-ref-args-aggregates`, `subnormal-text-and-tail-companions`. Decide each by 0051 item 2 (subject, not observers; an argument builder is an observer, decision 0058); multi-subject cases get cell lines. eval/milestones/ is protected: amend by a decision record.

## Acceptance criteria
- [ ] A decision record lists each flagged case with its milestone or the 0051 exception that keeps its tag; probe-milestones.tsv matches; `make eval-check` passes.
- [ ] The record gives the M0-M2 probe known-failure count before and after (`make eval`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing any cell; editing case.json tags (decision 0043).
