---
id: T-0022
title: "Run routine reviewer agents on Sonnet; keep Excel-fidelity and performance on Opus"
milestone: M0
lane: harness
status: done
priority: P0
size: S
depends_on: []
---

## Goal
Each PR's review fan-out costs less without weakening review where judgment matters most.

## Context
Every PR runs 3-5 reviewer agents; on 2026-10-04 each used 20k-70k tokens. Per-MTok prices:
Sonnet 5.5 $2/$10, Opus 5.5 $4/$20. The backlog PR for T-0016..T-0023 was reviewed by
security, architecture and quality on Sonnet (~21k tokens each) and their findings were sound.

## Acceptance criteria
- [x] `model: sonnet` in the frontmatter of reviewer-security, reviewer-architecture and reviewer-quality; reviewer-excel-fidelity and reviewer-performance pinned to `model: opus` (so a Sonnet worker cannot downgrade them).
- [x] The decision record says why and sets a check: after a week, the steward's spot-audit compares misses by reviewer; any miss attributable to the cheaper model reverts that reviewer to Opus.
- [x] `make check` green; reviewers approve.

## Out of scope
Worker model tiers (T-0023).

## Outcome
Reviewer frontmatter: security, architecture, quality `model: sonnet`; Excel fidelity and performance
`model: opus` (pinned). Review skill notes it. Decision 0015 sets the one-week spot-audit check. Eval delta: none.
