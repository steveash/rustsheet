---
id: T-0410
title: "Defined-name lookup by key instead of a scan of every name"
milestone: M3
lane: perf
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Resolving a defined name during evaluation costs one map lookup and no clone of the parsed definition, not a scan of every name.

## Context
Raised by the performance reviewer on T-0223. `Model::get_parsed_defined_name` (model.rs) walks all of
`parsed_defined_names`, upper-casing every key, and clones the `ParsedDefinedName` it finds (a LAMBDA
definition clones its whole body AST). Since T-0223, `evaluate_aggregate_arg` sends every defined-name
argument through `evaluate_node_with_reference`, so `SUM(Name)`/`MAX(Name)` pay this on each call.
`fn_isref` (functions/information/mod.rs) already does `parsed_defined_names.get(&(scope, name.to_lowercase()))`,
which suggests the keys are stored lower-cased. Confirm where the map is filled, then look names up by key and
return a borrow (or clone only the cheap variants). Keep the case-insensitive match identical, including
non-ASCII names (to_uppercase vs to_lowercase can differ, e.g. "ß"); add a unit test with a mixed-case and a
non-ASCII name.

## Acceptance criteria
- [ ] `get_parsed_defined_name` no longer iterates `parsed_defined_names`; a unit test covers mixed-case and non-ASCII names, sheet-scoped and global.
- [ ] 0 eval regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
