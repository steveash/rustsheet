---
id: T-0300
title: "Pay down mechanical lint debt (doc_markdown, uninlined_format_args, semicolon_if_nothing_returned)"
milestone: M0
lane: quality
status: in-progress
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
- [ ] Entries removed from crate Cargo.toml lint tables; `scripts/check_lint_debt.py --update` committed; `make check` green.
- [ ] `make check` green; reviewers approve.

## Out of scope
Semantic changes.
