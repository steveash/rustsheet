---
id: T-0426
title: "Suite corpus-bench: pinned SpreadsheetBench and FUSE subset from the xlsx-corpus-bench manifest, crash-only first"
milestone: M11
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
A pinned, fetched-at-test-time suite `corpus-bench` of real-world workbooks from witanlabs'
xlsx-corpus-bench manifest (SpreadsheetBench and a FUSE subset) runs crash-only in `make eval`.
This is the denominator for M11's exit ("formula-cell match on xlsx-corpus-bench ≥ 99.8%").

## Context
Owner request (2026-10-15): finish Phase 1's test suite before the Phase 2 re-architecture
(decision 0027). Building test infrastructure is allowed during Phase 1, and this is one of the
suites the milestone exit criteria name that does not exist yet.
- docs/research/test-suites-and-oracle.md §9–11:
  - xlsx-corpus-bench (Apache-2.0) publishes sha256 manifests and per-file counts, but not
    Excel's values.
  - SpreadsheetBench is CC BY-SA 4.0 and FUSE is CC-BY 4.0, so both are fetched at test time and
    never vendored.
- The pinned suite fetcher already exists (`rustsheet-eval fetch`, T-0002), and `poi` is a
  fetched suite.
- Start with a subset that runs within CI's eval budget (for example 500 SpreadsheetBench
  workbooks, chosen by a seeded sample of the manifest). Record the sampling rule so the
  subset can grow later.

## Acceptance criteria
- [ ] Fetch config pins the sources and sha256s. `rustsheet-eval fetch --suite corpus-bench`
  downloads and verifies the subset, and license and attribution are recorded in the suite.
- [ ] The suite runs crash-only, with no expectations yet. Any panic, timeout or import error is
  filed as a task, and `make eval` wall time stays within the CI budget (or the suite is excluded
  from CI and run by the steward).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Oracle values (T-0427).
