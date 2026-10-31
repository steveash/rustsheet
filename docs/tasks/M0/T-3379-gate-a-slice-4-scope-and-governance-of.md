---
id: T-3379
title: "Gate A slice 4: scope and governance of the reference tables (exempt list, missing-table count, append-only batches, known-failure rules)"
milestone: M0
lane: harness
status: done
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
- [x] `exempt.tsv` (`<FUNCTION>\t<class>\t<reason>`, classes `exact` and `composite`) exists with the first exempt
      functions, and the check prints the missing-table count: catalog functions with `scope: in` and
      `returns: number` that have neither a table directory nor an exempt line.
- [x] Governance: changing or deleting a committed file under `eval/accuracy/tables/`, and any change to `exempt.tsv`
      or `math-over-excel.tsv`, needs a decision record (adding a new batch's files does not).
- [x] Hygiene: every `known-failures.tsv` line names an existing task that is not `done`, and a line added in a diff
      names a point whose batch is added in the same diff.
- [x] `scripts/ci_changes.py` routes `eval/accuracy/**` and `tools/src/rustsheet_tools/accuracy/**` to the eval job.
- [x] Tests for each rule (SPEC-0029 test plan); `make check` green; reviewers approve.

## Out of scope
- The tables themselves (T-3358 and its follow-ups).

## Outcome
- `eval/accuracy/exempt.tsv` lands with 41 `exact` functions (counts, positions, codes, integer date parts, bit
  operations). `rustsheet-eval accuracy` reads it (`accuracy/lists.rs`), prints `missing tables <n>` (323 today: no
  table has landed yet, T-3358), fails on a stale exemption, and writes `holds` and `missing_tables` to its JSON. The
  catalog view `eval/catalog/scope.json` now carries `returns`, which the count reads (`Catalog::in_scope_numeric`).
- Governance (`scripts/accuracy_governance.py`, wired into `check_governance.py`, decision 0199): a change, delete or
  rename of a committed `eval/accuracy/tables/` file and any change to `exempt.tsv` or `math-over-excel.tsv` needs a
  decision record. A `known-failures.tsv` line must name an existing task that is not done, and a new line must name a
  point whose batch the same diff adds.
- `eval/accuracy/**` and `tools/src/rustsheet_tools/accuracy/**` are already code for `ci_changes.py`, so they run the
  eval job. A test now pins that.
- Tests: `scripts/test_accuracy_governance.py` (12 tests), plus the stale-exemption, missing-table and `holds` cases
  in `accuracy/tests.rs`, `accuracy/lists.rs` and `tests/accuracy_gate.rs` (the fixture now has its own catalog view
  and `exempt.tsv`).
- Eval delta: none (Gate A has no tables yet; the eval suites are untouched).
