---
id: T-1515
title: "Distribution functions: boolean/empty coercion, tail overflow and HYPGEOM.DIST/Z.TEST/T.TEST/F.TEST/CHISQ.TEST edges from the T-1049 sweep"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: sonnet
depends_on: []
---

## Goal
The engine matches Excel on the coercion and edge rules recorded from the T-1049 sweep.

## Context
See docs/excel-semantics/statistics.md, section "Modern distribution functions: coercion, error edges and domain rules (T-1049)", and the failing cells of `probes-numeric/t1049-*` (`rustsheet-eval show probes-numeric/t1049-<name>.case.json`). Gaps: boolean arguments (F.DIST, F.DIST.RT, F.INV.RT, WEIBULL.DIST, PHI, Z.TEST) give #VALUE! where Excel coerces; PHI/WEIBULL.DIST/F.DIST tail overflow cases; CHISQ.INV(.RT) with df 1E10 and p near 1; HYPGEOM.DIST edge domains and populations up to 1E6 (the grid also feeds T-0977); Z.TEST sigma/one-element/empty-range rules; T.TEST/F.TEST/CHISQ.TEST scalar and shape-mismatch errors. Split by function if the diff exceeds size M.

## Acceptance criteria
- [ ] The non-ulp mismatches of the listed t1049 cases pass and the baseline shrinks (`make eval-baseline`).
- [ ] Each rule cites its case id in a code comment.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Last-digit (ulp) differences of the incomplete gamma/beta kernels (T-1044, T-0977).
- The F.INV hang (T-1514).
