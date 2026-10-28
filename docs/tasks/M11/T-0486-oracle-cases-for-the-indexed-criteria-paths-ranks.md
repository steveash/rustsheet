---
id: T-0486
title: "Oracle cases for the indexed criteria paths: ranks and per-row SUMIFS over 100+ rows"
milestone: M11
lane: harness
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel-filled eval cases that go through T-0465's value indexes and prefix blocks. The indexed
COUNTIFS and SUMIFS paths would then be checked against Excel directly, not only against the walk.

## Context
T-0465 serves COUNTIFS and SUMIFS from a sorted or hashed index of a cached criteria range. It
also reads a block with the same columns (`I$11:I11`) from the cached range. Both only start on a
range of 64 or more cells, from its second distinct criterion. So the small eval cases rarely reach
them, and the unit tests (test_ifs_cache.rs) compare them with the walk only. The excel-fidelity
review of T-0465 asked for oracle cases.

## Acceptance criteria
- [ ] A probes case, filled by the excel-graph oracle, over 100 to 200 rows of mixed values:
      - numbers differing past 15 digits, -0, dates;
      - text numbers, text with case and accents, booleans, errors and blanks.
- [ ] Its formulas, each over the full block, a growing prefix and an inner block:
      - ranks `COUNTIFS(all,">"&x)+1+COUNTIFS(prefix,x)`;
      - per-row SUMIFS with number and text equality criteria;
      - each comparison operator.
- [ ] `make eval-check` passes the new case, or its failures are filed as engine tasks.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes, unless the case finds a difference.
