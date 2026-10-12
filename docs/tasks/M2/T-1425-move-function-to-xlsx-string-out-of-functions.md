---
id: T-1425
title: "Move Function::to_xlsx_string out of functions/mod.rs so new functions fit under the oversized-file ratchet"
milestone: M2
lane: quality
status: done
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
`crates/rustsheet-engine/src/functions/mod.rs` shrinks enough that a PR adding a function (an enum
variant, a name-table line, a language field and a dispatch arm: about 4 lines) passes the
oversized-file ratchet again.

## Context
`scripts/check_file_size.py` lets an inherited oversized file only shrink. On 2026-10-12, `main`
(c7e1d618) has `functions/mod.rs` at its ratchet value of 2,363 lines, so any PR that registers a
function or an `_xlfn.` name fails `make check` at `hygiene`. The steward found two stale PRs blocked
this way: #330 (T-0411, +5 `to_xlsx_string` arms, "grew 2363 -> 2368") and #504 (T-1214 BAHTTEXT,
+4 registration lines, "grew 2363 -> 2367"). Move `Function::to_xlsx_string` (the `_xlfn.` prefix
table, about 150 lines) into its own module, e.g. `functions/xlsx_names.rs`, as an
`impl Function` block. Do it as a pure move (no behaviour change), then record the drop with
`scripts/check_file_size.py --update`. `quality/` is protected, so the PR follows the
governance rule in the `steward` skill.

## Acceptance criteria
- [x] `functions/mod.rs` is at least 100 lines shorter. (Amended: `quality/oversized-files.json` keeps its old
      value, see the Outcome.)
- [x] `Function::to_xlsx_string` behaves the same (existing tests and the xlsx round-trip tests pass unchanged).
- [x] `make check` green; reviewers approve.

## Out of scope
- Adding the T-0411 names (#330) or BAHTTEXT (#504); those PRs merge `main` once this lands.

## Outcome
`Function::to_xlsx_string` moved verbatim (checked line for line against `main`) to `functions/xlsx_names.rs`;
`functions/mod.rs` is 2,363 -> 2,179 lines. `tools/src/rustsheet_tools/spec2xlsx.py` reads the `_xlfn.` arms
from the new file, and a new test checks that every name it reads is a real Excel function (a comment shaped
like an arm would otherwise add a fake prefix). The ratchet lock is deliberately not updated: the script fails
only when a file grows past its lock, so leaving 2,363 gives the 184 lines of headroom the Goal asks for
(BAHTTEXT, #504, adds 4). Recording 2,179 here would block #504 again by 4 lines. The next ratchet paydown
that runs `check_file_size.py --update` records the value.
