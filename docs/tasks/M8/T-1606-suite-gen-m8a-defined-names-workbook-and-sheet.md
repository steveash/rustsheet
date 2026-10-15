---
id: T-1606
title: "Suite gen-m8a: defined names (workbook and sheet scope, LAMBDA names) and 3-D references, oracle-filled"
milestone: M8
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
A committed, seeded suite `gen-m8a` with excel-graph values scores defined names and 3-D
references, so M8's exit criterion ("oracle suites for each feature at 100%") can be measured
for those features.

## Context
Owner request (2026-10-15): finish Phase 1's test suite before the Phase 2 re-architecture
(decision 0027). Building test infrastructure is allowed during Phase 1, and this is one of the
suites the milestone exit criteria name that does not exist yet.
- M8 has no generated or oracle suite yet. The catalog has no M8 functions because names and 3-D
  references are syntax, so this needs a generator policy, the way `--policy operators-refs`
  built gen-m1 (tools/src/rustsheet_tools/gen/cli.py).
- spec2xlsx already writes defined names (`wb.define_name`). Check how sheet-scoped names and
  LAMBDA-bodied names are written, and extend the builder if it can't write them.
- Existing evidence and quirks: docs/excel-semantics/references.md and the 3-D aggregate work
  (T-0122). Open probe task T-1431 covers non-ASCII names; don't duplicate it.
- Model the suite on gen-m7 (T-1275): suite.toml with `gate = false`, `milestone = ["M8"]`,
  sidecars from `rustsheet-oracle fill -b excel-graph`.

## Acceptance criteria
- [ ] A generator policy (for example `--policy names-3d`) produces cases covering:
  - workbook- and sheet-scoped names, the same name defined in both scopes, and names referring
    to constants, ranges, formulas and LAMBDAs;
  - names used inside other names, and relative and absolute name references;
  - 3-D ranges (`Sheet1:Sheet3!A1`, `SUM(Sheet1:Sheet3!A1:B2)`) across 2–5 sheets, including
    blank sheets and spans that include the formula's own sheet;
  - errors: undefined names, a 3-D span into a deleted-looking range, `#REF!`.
- [ ] `eval/suites/gen-m8a` is committed with ≥ 3,000 oracle-filled cells, canary-verified, and a
  `scripts/tasks.py`-filed task (or an existing one, cited) for every failing group.
- [ ] Every new quirk is recorded in docs/excel-semantics/ (references.md or a new names.md).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing engine failures (file them). Tables and structured references (T-1607). Sheet edits
  (T-1608).
