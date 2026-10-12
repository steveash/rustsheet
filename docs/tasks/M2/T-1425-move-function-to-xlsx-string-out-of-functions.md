---
id: T-1425
title: "Move Function::to_xlsx_string out of functions/mod.rs so new functions fit under the oversized-file ratchet"
milestone: M2
lane: quality
status: todo
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
- [ ] `functions/mod.rs` is at least 100 lines shorter, and `quality/oversized-files.json` records the new value.
- [ ] `Function::to_xlsx_string` behaves the same (existing tests and the xlsx round-trip tests pass unchanged).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Adding the T-0411 names (#330) or BAHTTEXT (#504); those PRs merge `main` once this lands.
