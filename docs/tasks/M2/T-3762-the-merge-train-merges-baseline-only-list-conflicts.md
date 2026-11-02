---
id: T-3762
title: "The merge train merges baseline-only list conflicts as sets instead of ejecting"
milestone: M2
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A PR whose only conflict with main is in the eval-gated baseline lists is merged by the train,
with the lists combined as sets, instead of being ejected and waiting for `make catch-up`.

## Context
Owner discussion, 2026-11-01. The lists (`eval/baseline/<suite>.txt`, the visible-parity lock)
are sorted line sets: `C` lines per case, `F`/`TF` lines per known failure. Two PRs touching
neighbouring lines conflict in git although their changes are independent. Decision 0207's
`make catch-up` clears this, but only after an ejection, a catch-up and another CI run; the
janitor (T-3630) automates that after 1 h.
In the train's build step the merge would instead be: main's lines, minus the lines the PR
removed, plus the lines it added, refusing a new `F`/`TF` line for a case the PR did not add (the
0207 regression guard). The train's CI then checks the lists against the engine, and a wrong
merge goes red and bisects as usual.
Evidence of the payoff is modest: of the 5 "conflict with main" ejections open at 07:05 UTC on
2026-11-01, one (#2638) was baseline-only; two were one-off fallout from decision 0219's lock
format change, one a real code conflict, and one already merged clean.

## Acceptance criteria
- [ ] The build step merges baseline-only conflicts as sets (a git merge driver or the train's
      own resolution), with the regression guard, and ejects anything else as before.
- [ ] Tests: neighbouring-line conflicts, a refused regression line, a conflict elsewhere.
- [ ] SPEC-0027 and decision 0207 are amended by a decision record.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Conflicts outside the gated lists.
