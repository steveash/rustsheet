---
id: T-0431
title: "corpus-bench: add the FUSE subset of xlsx-corpus-bench"
milestone: M11
lane: harness
status: blocked
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

## Blocked because
Checked 2026-10-18 by an opus run (T-1779 session):
- zenodo.org is not reachable from the agent environment: `curl -I https://zenodo.org/records/581678`
  gets `403` from the egress proxy (CONNECT tunnel failed). So nothing can be fetched, pinned or verified
  here.
- The FUSE archive has no per-file access. xlsx-corpus-bench `harness/fetch_corpus.py` downloads
  `fuse.zip` (9.4 GB), which holds a split 7z (`FUSE.7z.001`...), which holds `fuse-cc-binaries.tar.gz`.
  Extraction needs ~25 GB free and a 7z decoder. The workbooks then come out of the tar via `fuse_extract.py`.
  A per-file fetch by sha256 is therefore impossible from this source. A seeded sample needs the whole
  archive plus a 7z reader in `rustsheet-eval fetch`, and that is a new dependency with supply-chain review.

Unblock when either (a) the environment's network policy allows zenodo.org and a spec (docs/specs/)
picks between a 7z dependency in the fetcher and a one-off, attributed re-host of a pinned sample
(e.g. a GitHub release tarball under 2 GB, fetched like SpreadsheetBench), or (b) another pinned source
of individual FUSE workbooks is found.
