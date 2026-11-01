---
id: T-3623
title: "exit_owners: check colon cell spans per list block, not per line"
milestone: M0
lane: harness
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`scripts/exit_owners.py` counts a colon cell span such as `A153:A166` as owned when it sits on a continuation line of
a list item that names the case, as it already does for single A1 names, so wrapped cell lists stop showing as
unowned.

## Context
- The gap triage of 2026-11-01 found this. `names_in_span` (scripts/exit_owners.py) checks spans one line at a time:
  a span only counts when the case name is on the same physical line. `names_beside_case` reads whole blocks, meaning
  a line and its continuation lines (`blocks()`, decision 0159).
- The result: `make exit-owners MILESTONE=M9` reports about 304 t3354 F.DIST cells as unowned, though T-3479 lists
  them in spans on the wrapped lines of its cell list. A triager is then likely to file duplicate tasks for them.
- Keep the reason the check is per line, not per file: a task that quotes a formula range (`Data!B6:C10`) must not
  own those cells. Per block keeps that property.

## Acceptance criteria
- [ ] `names_in_span` checks each block from `blocks()` that names the case, not each line.
- [ ] A unit test covers both sides: a span on the continuation line of a list item naming the case is owned, and a
      span in a different block (for example a quoted formula range in another paragraph) is not.
- [ ] `make exit-owners MILESTONE=M9` no longer reports the t3354-f-dist cells that T-3479 lists. Record the before and
      after counts in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any other ownership rule.
