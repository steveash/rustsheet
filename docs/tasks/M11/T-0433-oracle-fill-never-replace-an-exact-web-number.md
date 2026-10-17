---
id: T-0433
title: "Oracle fill: never replace an exact web number with a 15-digit cached value (decision 0030 rule 1)"
milestone: M11
lane: harness
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
corpus-bench sidecars hold the numbers Excel for the web computed, not cached values that a
writer cut to 15 significant digits, so the 373 last-digit "failures" they cause disappear.

## Context
Found by the corpus-bench gap triage after T-0427. Every one of the 373 `numeric_ulp`/`numeric_close`
cells in corpus-bench is an oracle artifact, not an engine gap: the web result file in
`eval/.cache/oracle/excel-graph/<input sha256>.xlsx` holds exactly the engine's value, but the
sidecar holds the workbook's cached value, which has 15 significant digits. Example:
`corpus-bench/47766` `Total (2)!I26` `=D26/C26` (11/23): web 0.4782608695652174 = engine,
cached and sidecar 0.478260869565217. Likewise `49036 Trades!H2` web 4028.999999999998, sidecar 4029.

Cause: `_drop_desktop_conflicts` in tools/src/rustsheet_tools/oracle/cli.py applies decision 0030
rule 1 (T-0142): a web number within 1e-9 of a value cached by an AppVersion 16.x workbook is
overwritten with the cached value. AppVersion is 16.0300 for every Excel since 2016 and for files
re-saved by other writers claiming it (docs/research/test-suites-and-oracle.md, "Old calc engine"),
and these caches carry 15 digits: Excel's own saves keep 17 (docs/architecture/oracle.md, open
question 1), so a 15-digit cache is a writer's cut, not desktop evidence. corpus-bench/suite.toml
already says its caches are not Excel evidence (`use_cached_values = false`).

Affected cases (failing cells): 49278 (229), 54490 (61), 53068 (31), 52216 (29), 49036 (12),
38655 (6), 47766 (5). Inspect with `./target/release/rustsheet-eval show corpus-bench/<id>/1_<id>_input.xlsx`.

Options to weigh (a decision record if rule 1 changes, docs/decisions/0030): skip rule 1 when the
cached number text has fewer than 16 significant digits (needs the raw `<v>` text, which the snapshot
does not expose today), or skip it for suites whose caches are not trusted (`use_cached_values = false`).
The material-conflict drop (1651 pending cells in eval/suites/corpus-bench/PENDING.md) rests on the
same caches: decide whether it applies to an untrusted-cache suite too.

## Acceptance criteria
- [ ] Re-filled sidecars (from the cached web results, no new Graph session needed) of the seven cases above hold the web values; these 7 `F` lines (49278, 54490, 53068, 52216, 49036, 38655, 47766) leave eval/baseline/corpus-bench.txt with no new ones.
- [ ] A tools test covers the rule: a 15-digit cached number never overwrites a web number.
- [ ] Decision 0030 (or a new decision) states the changed rule; `make check` green; reviewers approve.

## Out of scope
- Engine changes: the engine already matches the web values.
