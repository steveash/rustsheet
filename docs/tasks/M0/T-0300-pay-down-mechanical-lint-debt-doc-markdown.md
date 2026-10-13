---
id: T-0300
title: "Pay down mechanical lint debt (doc_markdown, uninlined_format_args, semicolon_if_nothing_returned)"
milestone: M0
lane: quality
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Remove three LINT-DEBT allow entries by fixing the code (mostly `cargo clippy --fix`).

## Context
quality/lint-debt.json; lint-debt may only shrink.

## Acceptance criteria
- [x] Entries removed from crate Cargo.toml lint tables; `scripts/check_lint_debt.py --update` committed; `make check` green.
- [x] `make check` green; reviewers approve.

## Out of scope
Semantic changes.

## Outcome
Removed `doc_markdown`, `semicolon_if_nothing_returned` (engine, xlsx, python bindings) and
`uninlined_format_args` (engine, xlsx) from the LINT-DEBT tables: 8 entries gone from
`quality/lint-debt.json` (`check_lint_debt.py --update`, recorded under decision 0010). The code
changes are clippy's own fixes (doc backticks, inlined format args, trailing semicolons) across
130 files; no semantic change. Eval delta: none (0 fixed, 0 regressions).
