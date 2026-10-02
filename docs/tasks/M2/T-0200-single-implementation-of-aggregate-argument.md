---
id: T-0200
title: "Single implementation of aggregate-argument iteration"
milestone: M2
lane: functions
status: done
priority: P1
size: M
depends_on: []
---

## Goal
Replace the copies of 'iterate numeric args/ranges skipping text/bools in ranges' across financial, math, statistical modules with one shared iterator.

## Context
`python3 scripts/check_duplication.py --report 10` shows a 15x duplicated block (financial/mod.rs:821, mathematical.rs:289/372/489, count_and_average.rs:479, devsq.rs:31, …).

## Acceptance criteria
- [x] One helper (documented with Excel's coercion rules for direct args vs references) used by all call sites.
- [x] Duplicated lines reduced (update quality/duplication.json); eval unchanged or improved.
- [x] `make check` green; reviewers approve.

## Out of scope
Changing semantics (pure refactor unless a failing eval case proves a bug).

## Outcome
Added `Model::for_each_number_in_range` (functions/util.rs) and used it at 11 call sites (NPV, SUMSQ,
PRODUCT, DEVSQ, VAR/STDEV families, AVERAGE-family `for_each_value`). Duplicated lines 9153 → 8829;
eval unchanged. The `*A` variants (VARA/STDEVA, GCD/LCM) coerce text/booleans in ranges and
legitimately keep their own walks. Follow-up: a coercing counterpart helper for those.
