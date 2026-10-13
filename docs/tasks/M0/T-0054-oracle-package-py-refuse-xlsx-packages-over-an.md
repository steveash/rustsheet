---
id: T-0054
title: "oracle/package.py: refuse xlsx packages over an uncompressed-size cap before reading parts"
milestone: M0
lane: quality
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle tooling refuses an oversized xlsx before decompressing it.

## Context
Follow-up of T-0014 (the optional criterion). `package.read_parts` decompresses every entry into memory with no cap on entry count or total size (directive 5). Check sizes from `ZipInfo.file_size` before reading.

## Acceptance criteria
- [x] `read_parts` raises `PackageError` when the total uncompressed size or entry count exceeds a documented cap; a test covers it.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`package.py` gains `MAX_PARTS` (10000) and `MAX_UNCOMPRESSED_BYTES` (256 MiB); `read_parts` and `patch_parts` check the zip directory sizes before reading any part and raise `PackageError`. Test added. Eval delta none.
