---
id: T-3379
title: "Gate A slice 4: scope and governance of the reference tables (exempt list, missing-table count, append-only batches, known-failure rules)"
milestone: M0
lane: harness
status: todo
priority: P0
size: S
tier: opus
depends_on: [T-3356]
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
Gate A's tables cannot be weakened without a decision record, and the check reports which in-scope numeric functions
still lack a table (SPEC-0029, "Gate A holds" and "Known failures").

## Context
- SPEC-0029 (T-3355): `eval/accuracy/exempt.tsv`, `math-over-excel.tsv`, `known-failures.tsv` and the immutable
  batch files under `eval/accuracy/tables/`. Decision 0188 R2 and R3 (a new R3 case needs an owner ruling).
- Code: `scripts/check_governance.py` (`PROTECTED`, `is_protected`), `scripts/ci_changes.py` (routing),
  `scripts/tasks.py` (task lookup), the check of T-3356 (`rustsheet-eval accuracy`).
- Protected paths need a decision record (`make new-decision`); this task changes scripts/ itself, so it needs one.

## Acceptance criteria
- [ ] `exempt.tsv` (`<FUNCTION>\t<class>\t<reason>`, classes `exact` and `composite`) exists with the first exempt
      functions, and the check prints the missing-table count: catalog functions with `scope: in` and
      `returns: number` that have neither a table directory nor an exempt line.
- [ ] Governance: changing or deleting a committed file under `eval/accuracy/tables/`, and any change to `exempt.tsv`
      or `math-over-excel.tsv`, needs a decision record (adding a new batch's files does not).
- [ ] Hygiene: every `known-failures.tsv` line names an existing task that is not `done`, and a line added in a diff
      names a point whose batch is added in the same diff.
- [ ] `scripts/ci_changes.py` routes `eval/accuracy/**` and `tools/src/rustsheet_tools/accuracy/**` to the eval job.
- [ ] Tests for each rule (SPEC-0029 test plan); `make check` green; reviewers approve.

## Out of scope
- The tables themselves (T-3358 and its follow-ups).
