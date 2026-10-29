---
id: T-1752
title: "SKEW.P of fewer than three numbers is #DIV/0!, not 0"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
SKEW.P of a data set with fewer than three numbers is #DIV/0!, not 0.

## Context
Found by T-1623: probes/stats-evidence-floor-t1623 S!Q16 (`SKEW.P(A1:A2)`) and S!Q19 (`SKEW.P(C1:C5)`, two numbers after text/bool/blank are skipped): Excel gives #DIV/0!, ours 0. SKEW.P(A1) and SKEW.P(N1:N3) already match.

## Acceptance criteria
- [ ] The failing cell(s) in probes/stats-evidence-floor-t1623 pass and are removed from eval/baseline via `make eval-baseline`.
- [ ] The rule is recorded in docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Already fixed: T-1404 (done) made SKEW.P of fewer than three numbers `#DIV/0!`; `S!Q16` and `S!Q19` of probes/stats-evidence-floor-t1623
pass on `main` and are not in `eval/baseline/probes.txt` (only Q36 of that case is a known failure). No code change; the rule is in
statistics.md "SKEW.P of two numbers is #DIV/0!" and "SKEW, SKEW.P, KURT, HARMEAN, ... (T-1623)". Closed as a duplicate of T-1404.
