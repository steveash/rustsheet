---
id: T-0031
title: "Salvage findings from stale draft PRs 11, 13 and 38 before closing them"
milestone: M0
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Nothing of value is lost when the stale draft PRs #11, #13 and #38 are closed.

## Context
Three docs-only draft PRs from other sessions (2026-10-02..04) were blocked on the then-missing oracle and never merged. #11 (T-0904) is superseded by T-0913 but also filed T-0905..T-0907, which never reached main; its arrays.md theory was later disproven (arrays.md). #13 (T-0901) holds financial fit findings; #38 (T-0925) holds AS241 findings. Owner asked to clean them up.

## Acceptance criteria
- [x] T-0905 restored; T-0907 restored for IM* only (CONVERT is T-0922); T-0906 dropped (covered by T-0914 done, T-0915, T-0917, T-0939).
- [x] #13's findings in docs/excel-semantics/financial.md and T-0901; #38's in T-0925; both tasks stay `todo` (oracle exists, decision 0013).
- [ ] PRs #11, #13, #38 closed with a pointer to this PR (done right after this PR merges).
- [x] `make check` green; reviewers approve.

## Out of scope
- Implementing any of the salvaged tasks.

## Outcome
Salvaged as above in one docs PR; the three draft PRs are closed once it merges.
