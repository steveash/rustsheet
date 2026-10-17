---
id: T-0431
title: "corpus-bench: add the FUSE subset of xlsx-corpus-bench"
milestone: M11
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
corpus-bench also covers a pinned FUSE subset (the other half of witanlabs' xlsx-corpus-bench), crash-only like the SpreadsheetBench part.

## Context
T-0426 shipped only the SpreadsheetBench half (eval/suites/corpus-bench). FUSE (CC-BY 4.0) is a 9.4 GB zip on Zenodo (record 581678; URL in xlsx-corpus-bench harness/fetch_corpus.py), listed in xlsx-corpus-bench results-fuse/manifest.jsonl (10,702 unique workbooks). The archive fetcher caps archives at 2 GB (`MAX_ARCHIVE_BYTES`, fetch.rs), so this needs a design choice: a second suite with per-file fetches by sha256 from the manifest, or a raised cap with a seeded sample extracted. Pick a subset that a local run finishes in minutes and record the sampling rule. Keep `use_cached_values = false`: FUSE caches are not en-US Excel evidence either.

## Acceptance criteria
- [ ] `rustsheet-eval fetch` pins and verifies the FUSE subset; license and attribution recorded; crash-only run's failures in the baseline as new coverage and filed as tasks.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
