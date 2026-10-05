---
id: T-0029
title: "Cap decompressed part size in the eval xlsx snapshot reader"
milestone: M0
lane: harness
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`rustsheet-eval snapshot`/`run` refuse an xlsx part whose decompressed size exceeds a fixed limit instead of reading it into memory (zip-bomb guard, CLAUDE.md directive 5).

## Context
`crates/rustsheet-eval/src/xlsx_snapshot.rs::read_part` calls `read_to_string` on the zip entry with no cap, so a small malicious workbook can expand to gigabytes. Found by reviewer-security on the T-0203 PR. Check how `crates/rustsheet-xlsx` guards its own reads and reuse that limit (one definition) rather than inventing a second constant.

## Acceptance criteria
- [x] `read_part` stops at the limit (e.g. `Read::take(limit + 1)`) and returns `SnapshotError::Malformed` naming the part when exceeded.
- [x] A unit test builds an in-memory zip whose part decompresses past the limit and asserts the error.
- [x] Eval results unchanged.
- [x] `make check` green; reviewers approve.

## Out of scope
- Limits in the engine's importer (separate if missing).

## Outcome
`read_part` now reads through `Read::take(limit + 1)` and returns `SnapshotError::Malformed` naming the part when it exceeds `MAX_PART_BYTES` (256 MiB). `rustsheet-xlsx` has no existing limit to reuse, so the constant lives in the snapshot reader; the importer limit stays out of scope. Unit test `part_over_the_size_limit_is_malformed` covers it. Eval results unchanged.
