---
id: T-1679
title: "references.md: narrow the stale UNVERIFIED external-cache sentence and the reversed-span claim (T-1609 audit)"
milestone: M8
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
docs/excel-semantics/references.md states exactly what the T-1609 evidence verified about cached external workbooks, and nothing more.

## Context
Found by the 2026-10-16 steward spot-audit of PR #871 (T-1609). The opus architecture and quality reviewers each flagged it as MINOR.
- The T-1459 paragraph (around line 228) still lists "a book, sheet or name the cache does not hold" as unprobed and UNVERIFIED. fixtures/external-link-cached-lookups Refs!B18-B20 now show `[1]!NoSuchName` and `[1]Nope!$A$1` are `#BLOCKED!`.
- The T-1609 bullet says the reversed span `[1]Defines:Uses!` gives "the same". Only `SUM([1]Defines:Uses!$B$2)` = 142 was probed (fixtures/external-link-cached-3d-span Refs!B9).

## Acceptance criteria
- [x] The UNVERIFIED sentence lists only what is still unprobed: the `[other.xlsx]` file-name book form and `refreshError="1"` sheets. It points to the T-1609 cells for uncached names and sheets.
- [x] The reversed-span sentence names only the probed SUM cell.
- [x] `make check` green; reviewers approve.

## Out of scope
- New probes; implementation of 3-D external spans (T-1654).

## Outcome
references.md: the UNVERIFIED sentence now lists only the `[other.xlsx]` file-name book (and the refreshError sheets, already in the same sentence) and points to Refs!B18-B20 for uncached names and sheets; the reversed-span claim names only the probed SUM cell (Refs!B9). Docs only; no eval change.
