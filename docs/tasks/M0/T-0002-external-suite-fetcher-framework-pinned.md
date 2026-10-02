---
id: T-0002
title: "External suite fetcher framework (pinned downloads)"
milestone: M0
lane: harness
status: in-progress
priority: P1
size: M
depends_on: []
---

## Goal
Suites whose files are too large or not redistributable can be declared in suite.toml with a pinned source and fetched on demand into eval/.cache/suites/<name>/.

## Context
docs/research/test-suites-and-oracle.md §A.2 ranks POI, xlsx-corpus-bench (FUSE/SpreadsheetBench), LibreOffice and Gnumeric suites; several are CC-BY(-SA)/GPL and must not be vendored. `SuiteManifest` currently uses `deny_unknown_fields`.

## Acceptance criteria
- [ ] `suite.toml` accepts an optional `[fetch]` table: `url` (git or archive), `rev` or `sha256`, `subdir`, `include`.
- [ ] `rustsheet-eval fetch [--suite S]` (or a tools/ command, but only one implementation) downloads to `eval/.cache/suites/<name>` and verifies the pin; `root` of fetched suites resolves there.
- [ ] Unfetched suites are reported as `not fetched` by `rustsheet-eval suites` and skipped by `run`/`check` without failing.
- [ ] Unit tests for manifest parsing and path resolution; `make check` green.
- [ ] `make check` green; reviewers approve.

## Out of scope
Adding concrete external suites (T-0003).
