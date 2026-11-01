---
id: T-3309
title: "Generator: accented and combining-mark texts in the data and criteria pools, and an oracle-filled gen-m2 batch of criteria and comparison cells over them"
milestone: M2
lane: harness
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
gen-m2 holds oracle-backed cells where the *IF(S) criteria, MATCH/VLOOKUP and the comparison
operators meet accented, decomposed and expanding texts in random combinations with wildcards and
operators, so Excel's collation rules (T-3154, T-3181, T-3201) are checked beyond the hand-built
pair grids of the probes.

## Context
- The engine compares text with `cmp_text_pairwise` (operators, criteria) and `cmp_text` (sorts,
  lookups) in crates/rustsheet-engine/src/functions/util/collation.rs, fitted on probes only
  (probes/t3154-*, probes/t3201-*, probes/t3181-*, probes/t3133-lookup-criteria-case-fold). The
  generator's data `TEXT_POOL` (tools/src/rustsheet_tools/gen/generator.py) has one accented entry,
  `"ü"`, and `CRITERIA_POOL` none, so gen-m2's 21,480 cells barely touch these rules.
- Precedent for the change and the batch: T-2717 (criteria pool entries plus a unit test) and its
  batch `--focus --functions COUNTIF,COUNTIFS,SUMIF,SUMIFS --count 2 --seed 2717`. T-3064 is a
  separate batch for T-2717's entries; do not fold it in.
- Entries to add (Python strings; the xlsx writer stores them as typed):
  - `TEXT_POOL`: `"é"`, `"e"` + U+0301, `"É"`, `"æ"`, `"ae"`, `"ß"`, `"ss"`, `"ǎ"`, `"a"` + U+030C,
    `"co-op"`, `"coop"`, `"σx"`, `"ςx"`.
  - `CRITERIA_POOL`: `"é"`, `"=ae"`, `"<>ß"`, `"<é"`, `">=æ"`, `"é*"`, `"?s"`, `"ς*"`.
  A stored generated case never changes when the pools change (cases are committed with their
  formulas); only new seeds draw the new entries.

## Acceptance criteria
- [ ] `TEXT_POOL` and `CRITERIA_POOL` hold the entries above; a test in tools/tests/test_generator.py
      shows a seeded draw emits an accented data value and an accented criterion.
- [ ] A new seeded gen-m2 batch, `rustsheet-gen --milestone M2 --focus --functions
      COUNTIF,COUNTIFS,SUMIF,SUMIFS,MATCH,VLOOKUP` (seed recorded in the Outcome), is filled with
      excel-graph and committed; its sidecars hold at least one nonzero COUNTIF/SUMIF result whose
      criterion or matched data value is accented (cite the cell in the Outcome).
- [ ] The new cells pass, or the failing cells are known failures recorded by `make eval-baseline`
      (0 regressions) and listed in a filed engine task (case stem and A1 names in one bullet,
      decision 0159).
- [ ] `cd tools && uv run ruff check && uv run mypy` clean; `make check` green; reviewers approve.

## Out of scope
- Changing the collation code (the follow-up task, if any cell fails).
- New generator policies; other scripts, width and kana (T-3275, T-3276).
