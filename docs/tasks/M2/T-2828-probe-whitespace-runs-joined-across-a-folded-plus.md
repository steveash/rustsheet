---
id: T-2828
title: "Probe whitespace runs joined across a folded plus or a sign when they hold line feeds"
milestone: M2
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Know what Excel prints when two typed whitespace runs meet around a token that leaves the tree (a folded plus, a
sign) and either run holds line feeds, so `Run::then` matches it.

## Context
- From T-2782 review: `Run::then` (crates/rustsheet-engine/src/expressions/parser/spacing.rs) joins the run before a
  folded `+` with the run after it, adding the line feeds and keeping the earlier spaces only when the later run has
  no line feed. That rule is inferred, not probed. Within one run, spaces before 256 line feeds print nothing
  (probes/t2782-whitespace-run-wrap A9), but `then` keeps them when the later run's count has wrapped to 0.
- docs/excel-semantics/syntax.md ("Ours" under typed whitespace) marks the join as unprobed.

## Acceptance criteria
- [ ] A probe case (excel-graph) with FORMULATEXT and LEN for: `=  +` LF `5`, `=` LF `+  5`, `=1+ ` LF ` +` LF ` 5`,
      `=- ` LF ` 5`, `=1+   +` + 256 LF + `5`; findings recorded in docs/excel-semantics/syntax.md.
- [ ] If any answer differs from what ours prints, an engine task filed with `make new-task`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Runs inside one gap (T-2782, done).

## Outcome
Probed all five shapes on excel-graph (probes/t2828-whitespace-join-folded-plus); our FORMULATEXT, LEN and values match in all 30 cells, so no engine task. Recorded in docs/excel-semantics/syntax.md. Eval delta: +30 probe cells passing, no regressions.
