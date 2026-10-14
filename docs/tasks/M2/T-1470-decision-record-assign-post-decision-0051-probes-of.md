---
id: T-1470
title: "Decision record: assign post-decision-0051 probes of M5-M10 subjects to their own milestone in probe-milestones.tsv"
milestone: M2
lane: research
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
Every `probes` case written since decision 0051 counts toward the milestone of the function or feature it
probes, as docs/milestones.md (M2 exit) and decision 0051 item 2 require. New probes stop arriving with an
`M1` tag that puts them on M2's exit.

## Context
On 2026-10-13, `make eval` assigns 196 known-failure cells to M0-M2 (decision 0051, `probe milestone M0-M2` line).
149 of them are in cases whose subject is a later-milestone function or feature. Their tasks were M1, so they
were tagged `M1` after the freeze, and no override line exists:
- **M9 financial functions (102 cells):** `date-system-1900-odd-accrint-bound` (17), `-1900-odd-threshold` (13),
  `-1904-odd-accrint-bound` (30), `-1904-odd-threshold` (19) (ODDF*/ODDL*, T-1447);
  `date-system-1900-financial-daycount` (9), `-1904-financial` (6), `-1904-financial-daycount` (5),
  `-1904-financial-shift` (2) (DISC/INTRATE/YIELD*/PRICE*/TBILL*/XIRR, T-1436);
  `date-system-1900-coupon-bound` (1) (DURATION, T-1440). The passing twins (`date-system-1904-coupon-bound`,
  `-1904-readers`) need the same call.
- **M10 number-format codes in TEXT (31 cells):** `text-format-length-limit-c` to `-j` (0051 already maps
  `text-format-length-limit` and `-b` to M10).
- **M8 external references (8 cells):** `external-ref-no-link-shapes` (0051 maps `external-ref-no-link` to M8).
- **M7 LAMBDA helpers (8 cells):** `lambda-helper-range-slices` (MAP/SCAN/REDUCE/BYCOL; tagged `M2`, T-1395).
Cell-level candidates (cell lines, T-1423), where the case keeps its tag under 0051's exception:
`volatile-invariant` RANDBETWEEN/RANDARRAY cells A2, A8, A12, A13, B9, B10, B18, F1:F9 (M5, T-1316/T-1318);
`index-lifted-edge-cases` H131:J131 (COUNTBLANK, M9, T-1437); `written-parens-reference-only` E40, E45 (ISREF of
LET, M7, T-1106).
Each failing cell already has an open task in its own milestone (T-1457, T-1374, T-0916, T-1455, T-1450, T-0936,
T-1454, T-1442, T-1316, T-1318, T-1437, T-1106). This task moves no failure out of sight. It only applies
0051's assignment rule. The 10 lone-surrogate cells (`text-surrogate-positions`, `concatenate-limit-spill`)
are M2 subjects and stay M2 (T-1030).

**Update (steward 2026-10-14):** the list above is out of date. On main 6ab245f0, `make eval`
assigns 518 known-failure cells to M0-M2 (it was 196). Cases added on 2026-10-13 also carry an `M1` tag
but probe later-milestone subjects. Include each of these too, after checking its subject:
`yield-solver-sweep` (66, YIELD), `date-system-1900-financial-readers` (17), `-1900-financial-readers-shifted` (18),
`-1904-financial-readers` (18), `date-system-1900-odd-years` (19), `-1904-odd-years` (7), `coupdays-feb-month-end` (4),
`text-format-unquoted-letters` (75, number-format codes), `text-format-length-limit-l` (4) and `-m` (1),
`textjoin-delimiter-array` (5), `eta-reduced-names` (3) and `countblank-array-values` (5).
`text-collation-order` (156) and `text-order-nbsp-space` (1) are M2 subjects (T-0220) and stay M2. The
"about 47" target in the acceptance criteria no longer holds: report the before/after counts from `make eval`
instead. T-1519 (harness) adds a gate that stops new wrongly tagged probe cases.

Reproduce the per-case list: `make eval`, then compare eval/runs/latest/report.json `cases[]` (suite
`probes`) against the rules in crates/rustsheet-eval/src/probe_milestones.rs. T-1471 adds this list to the report.
eval/milestones/ is protected (decision 0044), so the mapping edit needs this decision record (`make new-decision`).

## Acceptance criteria
- [x] A new decision record (amending 0051) lists each case and cell named above with its milestone, or with
      the reason it keeps its tag under 0051's exception.
- [x] eval/milestones/probe-milestones.tsv has the matching case and cell lines, and `make eval-check` passes.
- [x] The decision record states the M0-M2 known-failure count before and after, taken from `make eval`. (See the 2026-10-14 update in Context:
      the old 196 -> 47 estimate is stale.)
- [x] .claude/skills/excel-probe/SKILL.md states that a new probe case is tagged with the milestone of the function
      or feature it probes (decision 0051), never with the milestone of the task that wrote it.
- [x] `make check` green; reviewers approve.

## Out of scope
- Fixing any of the cells (their tasks are listed above).
- Editing case.json tags (oracle input hashes; decision 0043).

## Outcome
Decision 0058 applies decision 0051 item 2 to the probes written since 0051. It adds 42 case lines and
21 cell lines to `eval/milestones/probe-milestones.tsv`: M9 22, M10 12, M8 3, M7 2, M5 3. The cell
lines are on `volatile-invariant`, `index-lifted-edge-cases` and `written-parens-reference-only`. The
decision also lists the cases that keep their tag under 0051's exception, with the reason for each.
The excel-probe skill now says to tag a probe with its subject's milestone. `make eval` probe
milestone M0-M2 known failures went from 512 to 179 (361 cases / 13,249 cells before, 319 / 10,186
after). Every cell still counted for M0-M2 is an M2 subject. Eval delta: none (no cell changes
pass/fail).
