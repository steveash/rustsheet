---
id: T-2883
title: "Probe name/INDEX/IF forms of a text or boolean cell as data of TRIMMEAN, MODE, paired stats"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Name, INDEX, OFFSET, IF and CHOOSE results holding a text or boolean cell are read like a cell reference in every statistical function that matches `Node::ReferenceKind` by hand.

## Context
T-2344 fixed QUARTILE/PERCENTILE/PERCENTRANK by reading the argument once with `evaluate_picked_reference_arg` (percentile.rs). These still hand-roll `matches!(*x.ungrouped_at(row), Node::ReferenceKind{..})`: trimmean.rs, mode_functions.rs, paired.rs, chisq.rs, z_test.rs, regression.rs, util/aggregate_args.rs, and others (`rg 'Node::ReferenceKind' crates/rustsheet-engine/src/functions`). Probe each with a name/INDEX/IF cell, then share one definition.

## Acceptance criteria
- [ ] Oracle-backed probe case per function family; engine matches; statistics.md entries updated; no second definition of the concept.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
