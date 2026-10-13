---
id: T-0053
title: "spec2xlsx: use oracle/package.py for its zip read/patch/rewrite (_patch_parts, entered_cells)"
milestone: M0
lane: quality
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`oracle/package.py` is the only zip read/patch/rewrite in tools/.

## Context
Follow-up of T-0014. `spec2xlsx._patch_parts` is another unzip/patch/rezip loop (it keeps ZipInfo and rejects missing parts) and `entered_cells` opens the zip directly. `write_parts` would need to keep ZipInfo metadata. spec2xlsx stays a leaf module that `backends` imports, so move the helper somewhere both can import without a cycle.

## Acceptance criteria
- [x] No `zipfile` use in tools/src outside the shared package helper.
- [x] spec2xlsx output is byte-identical for the existing tests.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`spec2xlsx._patch_parts` moved to `oracle/package.patch_parts` (keeps ZipInfo, rejects missing parts); `entered_cells` uses `read_parts`. No `zipfile` use in tools/src outside `oracle/package.py`. Tools tests pass unchanged; eval delta none.
