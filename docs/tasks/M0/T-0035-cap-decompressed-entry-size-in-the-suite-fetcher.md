---
id: T-0035
title: "Cap decompressed entry size in the suite fetcher extract_zip"
milestone: M0
lane: harness
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`extract_zip` in `crates/rustsheet-eval/src/fetch.rs` refuses an archive entry whose decompressed size exceeds the limit instead of copying it to disk unbounded (zip-bomb guard, CLAUDE.md directive 5).

## Context
`io::copy(&mut entry, ...)` has no cap. Found by reviewer-security on the T-0029 PR. Reuse `rustsheet::LoadOptions::default().max_uncompressed_bytes` as T-0029 does; do not add a second constant.

## Acceptance criteria
- [ ] `extract_zip` stops at the limit (`Read::take(limit + 1)`) and errors naming the entry when exceeded.
- [ ] A unit test builds a zip with an entry past the limit and asserts the error.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Total-size limits across entries.
