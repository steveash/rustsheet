---
id: T-0021
title: "Fuzz the formula parser and xlsx importer (cargo-fuzz, nightly)"
milestone: M0
lane: quality
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
Untrusted formulas and workbooks are fuzzed continuously, so panics, stack overflows and
unbounded allocation are found by machines, not by luck (CLAUDE.md directive 5).

## Context
T-0931 (stack overflow on deeply nested parentheses) was found by hand. Fuzzing was planned for
M12; the 2026-10-04 review moved it forward.

## Acceptance criteria
- [ ] `fuzz/` with cargo-fuzz targets: formula parse, parse+evaluate on a small model, xlsx import (with zip-bomb limits on).
- [ ] Seed corpora from eval/suites and crates/rustsheet-xlsx/tests; dictionary of formula tokens.
- [ ] Nightly workflow runs each target for a fixed time budget; a crash uploads the input and fails the job; the steward files a task per distinct crash.
- [ ] The `fuzz/` crate depends only on public engine/xlsx APIs; nothing in the engine depends on it.
- [ ] Any crash found while building this is fixed or filed.
- [ ] The workflow uses least-privilege `permissions:`, never `pull_request_target`, and uploads only the crashing input (no secrets or environment).
- [ ] Decision record for the protected-path change (CI); `make check` green; reviewers approve.
