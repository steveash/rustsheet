---
id: T-2805
title: "SPEC-0024 step 3: charge the other whole-array readers a LAMBDA can repeat"
milestone: M7
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-2803, T-2804]
---

## Goal
Every function that reads a whole array argument charges the cells it reads to the formula work budget (SPEC-0024). No reader
is left that a LAMBDA can repeat over a LET-bound array without bound.

## Context
SPEC-0024 rollout step 3. Audit the whole-array readers and charge each the cells it reads through `Model::spend_work`. The
list to check: MATCH/XMATCH/XLOOKUP/LOOKUP/VLOOKUP/HLOOKUP linear search; SORT/SORTBY; UNIQUE; FILTER; TEXTJOIN/CONCAT;
TOCOL/TOROW; SUMPRODUCT; MMULT inputs; the *IF(S) over arrays; INDEX over arrays (O(1), so no charge; record that). A reader also charges the text elements it reads by their length (⌈bytes / `TEXT_BYTES_PER_UNIT`⌉, SPEC-0024 section 1). Prefer a
shared helper at the point where an array argument is read whole. Do not add a per-function copy (directive 3). List the
covered readers in the SPEC-0024 DEVIATION entry in arrays.md.

## Acceptance criteria
- [ ] For each charged reader, `LET(s,SEQUENCE(100000),REDUCE(0,SEQUENCE(100000),LAMBDA(a,b,a+<reader over s>)))` is
      `#VALUE!` within the harness timeout (engine tests, counters, no timing).
- [ ] The DEVIATION entry lists the covered readers.
- [ ] No baseline gains a line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The aggregate walker (T-2804).
