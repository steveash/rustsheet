---
id: T-1568
title: "probe-milestones.tsv loader rejects a cell line whose cell is not a checked cell of its case"
milestone: M2
lane: harness
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A cell line in eval/milestones/probe-milestones.tsv whose `Sheet!A1` is not a checked cell of its case (a key of the case's `.oracle.json`, or of its `expected`) fails the eval run, naming the line. Today such a line is silently ignored.

## Context
The T-1562 review (decision 0060, 1,086 new cell lines) found that `Mapping::check_known` (crates/rustsheet-eval/src/probe_milestones.rs) checks only that a line's case exists. A typo in a cell would leave that cell counted for its case's milestone, and nobody would see it. Extend the check to cells, using the scored cells the run already has, and add a unit test beside `an_unknown_case_names_its_line`.

## Acceptance criteria
- [ ] A cell line naming an unchecked cell makes `rustsheet-eval run`/`check` fail with the line number; a unit test covers it.
- [ ] The committed mapping still passes (`the_committed_mapping_parses` and `make eval-check`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing any line of the mapping.
