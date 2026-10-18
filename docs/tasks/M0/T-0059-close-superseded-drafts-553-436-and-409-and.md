---
id: T-0059
title: "Close superseded drafts #553, #436 and #409 and record their tasks' state"
milestone: M0
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Nothing of value is lost when the stale draft PRs #553 (T-1284), #436 (T-1186) and #409 (T-1155) are
closed, and each task's state on main says what is true now.

## Context
Three draft PRs from other sessions (2026-10-08 to 10-11) each marked their task blocked on the branch
and never merged. A task blocked on its pushed branch stays claimed by that branch (decision 0048),
and the orphan rule (decision 0050) does not cover drafts or blocked tasks, so the steward cannot merge
them. Main has since moved past all three:
- #553: GEOMEAN(5,7) (probes/subtotal-reference-operands-d H28) is still a known failure
  (eval/baseline/probes.txt), and its conflict with GEOMEAN(1,3) is the in-range class numbers.md
  "GEOMEAN in-range root" already records (T-0288). T-1108 covered only the over/underflow path.
- #436: T-1778's research plan (docs/research/t1778-erf-kernel-research-plan.md, updates 4 and 7) found
  the exact ERFC tail for 4 < x <= 26.55. It confirms the branch's rounded-square finding and rules out
  the continued fraction the branch kept. The branch's code change alone fixes no gated cell. Rebuilt
  on main 0b5ce7a4, it leaves every gated suite unchanged (A5 stays one ulp off), gains 4
  probes-numeric cells and loses 3 (m9-norm-s-inv). Its T-1187 file had already reached main through
  #434 and was finished by #539.
- #409: T-1583 (#813) and T-0916 (#914) fitted XIRR. The cells T-1155 names pass, and financial.md
  "XIRR is a bracketing bisection, not Newton" replaces the branch's partial model.
This follows T-0031's salvage pattern.

## Acceptance criteria
- [x] T-1284 `blocked` (time-boxed, decision 0056, with a reopen condition); H28 added to numbers.md's
  remaining in-range misses.
- [x] T-1186 stays `todo`, depends on the new engine task T-1916 (the T-1778 tail), and its Context
  points at the tail model. T-1187's false "now in `erfc_cf`" claim is corrected. numbers.md and
  statistics.md no longer call T-1186 blocked.
- [x] T-1155 `done` (its zero-first-value guesses moved to T-1591); financial.md's "open (T-1155)" fixed.
- [x] T-0941 `done` (its cells pass, eval/baseline/ironcalc-calc.txt:164).
- [ ] PRs #553, #436 and #409 closed with a pointer to this PR (right after this PR merges).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing anything (T-1916 does the ERFC tail).
- docs/research/t1778-erf-kernel-research-plan.md, which the T-1778 session owns. Its "T-1780" engine
  reference is wrong (T-1780 is HSTACK/VSTACK); T-1916's Context records it.

## Outcome
Docs only, eval delta none.
- #553 (T-1284): superseded. T-1284 is `blocked`, time-boxed (decision 0056). numbers.md lists H28 and
  records that r's distance from the exact root does not decide the outcome (70 in-range probe cells).
- #436 (T-1186): superseded by T-1778. A new engine task, T-1916, implements the tail and first lands
  the coefficients from `origin/agent/research/T-1778-kernel-map` on main. T-1186 depends on it.
- #409 (T-1155): superseded by T-1583 and T-0916. T-1155 is `done`; T-1591 gains the guesses 0.2 and 1.
- Side finding: T-0941's two GEOMEAN.xlsx cells already passed (since T-1113), so it is `done`.
