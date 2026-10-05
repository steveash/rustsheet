---
id: T-0025
title: "Decision record: which Excel wins when desktop and web evidence disagree"
milestone: M0
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A decision record in `docs/decisions/` stating which evidence decides goldens when desktop Excel
(Excel-authored cached values, a future Windows backend) and Excel for the web (the excel-graph
oracle) disagree, and how such cells are scored.

## Context
Oracle calibration (docs/research/oracle-calibration.md, T-0004) found two kinds of divergence:
INDIRECT to a missing sheet (#REF! on desktop, an empty-cell reference on the web;
docs/excel-semantics/lookup.md) and last-digit floating point in ATAN2, IM*, ERFC, BESSEL,
MIRR, IRR and SIN (docs/excel-semantics/numbers.md). The docs currently prefer desktop where
the engine already follows it, without a recorded decision. Both are "Microsoft 365 Excel", so
the choice affects how eval is scored and what agents may cite.

## Acceptance criteria
- [x] Decision record with options (desktop wins / web wins / accept either for listed divergences), the choice and its consequences for sidecars, baselines and probes.
- [x] docs/architecture/oracle.md and the two excel-semantics entries point to it.
- [x] Reviewers approve.

## Out of scope
Building the Windows desktop backend.

## Outcome
Decision 0030: desktop en-US Microsoft 365 Excel wins when it and Excel for the web disagree;
web-only values are goldens except for the listed known divergences; non-en-US desktop workbooks
are no evidence for locale-dependent behavior; last-digit web goldens (≤ 4 ULPs) in the listed
functions stay known failures rather than fitting targets. oracle.md, excel-semantics README,
lookup.md and numbers.md point to it. No code or eval change (eval delta: none).
