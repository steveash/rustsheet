---
id: T-0015
title: "spec2xlsx: deterministic output so oracle input_sha256 and the cache are reproducible"
milestone: M0
lane: harness
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`build_xlsx(spec)` returns identical bytes for an identical spec across runs and machines.

## Context
Found reviewing the T-0004 smoke fill: XlsxWriter embeds the creation time in
`docProps/core.xml`, so the same unchanged case hashes differently in a later run. Consequences:
the `source.input_sha256` recorded in `*.oracle.json` cannot detect a stale sidecar after a case
edit, and `eval/.cache/oracle/` (keyed by that sha) misses on every new run, re-uploading every
workbook to Excel. Likely fix: pass a fixed `created` date via `Workbook.set_properties` (and check
for any other run-dependent bytes, e.g. zip timestamps).

## Acceptance criteria
- [ ] A test builds the same spec twice (with a time change in between, e.g. a patched clock) and gets identical bytes.
- [ ] Existing sidecars' `input_sha256` either match the new deterministic output or are documented as legacy.
- [ ] `make check` green; reviewers approve.

## Out of scope
Changing the case format or re-filling sidecars (values do not depend on the hash).
