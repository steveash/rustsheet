---
id: T-2867
title: "Calipers anonymisation artifacts: score pv26_020/pv26_2_020 Summary row 17 and pv26_010 Cover!C16 against Excel's values for the shipped workbook (51 visibly-off cells; MIN, MAX, AVERAGE Phase 1)"
milestone: M2
lane: research
status: todo
priority: P1
size: S
tier: opus
campaign: semantics
depends_on: []
---

## Goal
The calipers cells whose cached value the corpus anonymisation broke are scored against what Excel computes for
the workbook as shipped, so they stop counting as visibly-off cells of MIN, MAX, AVERAGE (M2), MEDIAN and
INDIRECT, and nothing else in those workbooks changes.

## Context
Gap triage 2026-10-27 (`make eval` on 7bd9090e). Corpus background: T-0446 (the calipers suite) and T-2660. These 51 visibly-off cells have no open owner; T-2660 (done)
found the cause and left them "not actionable", but they still count against Phase 1 (decision 0114):
MIN, MAX and AVERAGE are M2 functions whose only visibly-off attributed cells are these (6 each, 1,932 / 2,836 /
13,585 Phase 1 cells), so the current milestone's functions miss the 99.9% bar because of a corpus artifact.
- `calipers/pv_goldens/pv26_020/golden.xlsx` and its twin `pv26_2_020`, sheet Summary (25 cells each):
  C17:O17 `=INDIRECT($B17&"!TargetName")` and the like (cached `Company2277`, 41239, 41303, "USD", ...; ours
  `#REF!`), and M23:O26 `=MAX/AVERAGE/MEDIAN/MIN(M7:M21)` over them (cached numbers; ours `#REF!`).
  The anonymisation renamed the text in `Summary!B17` to `Company0691` but left the sheet it names as `ConAgra`
  (whose sheet-scoped `TargetName`, D24, holds `Company2277`). The shipped workbook has no `Company0691`
  sheet, so `#REF!` is what Excel gives for it (docs/excel-semantics/lookup.md "INDIRECT of `Sheet!Name` text",
  calipers note; "INDIRECT to a sheet that does not exist is #REF!").
- `calipers/pv_goldens/pv26_010/golden.xlsx` `Cover!C16`: stored as `<f ca="1">45678</f>` with no `<v>`, so
  the expectation is empty while Excel computes 45678 (ours 45678). The anonymisation replaced a volatile
  formula (`ca="1"`) by a literal and dropped its value.

Mechanism: decision 0032's per-cell `cached_overrides` (eval/suites/calipers/cached-overrides.tsv, read by
crates/rustsheet-eval/src/overrides.rs) replaces a cached expectation with a cell of a committed excel-graph
sidecar computing the same formula on the same inputs. Decision 0032 limits overrides to decision 0030 rule 2
evidence "or a later decision that says so" (decision 0090 added kept subnormals), so this needs a decision
record first (`make new-decision`): cached values the corpus anonymisation made inconsistent with the shipped
workbook are not Excel evidence for it. Never hand-write a value, never add these cells to eval/baseline by hand.

Evidence to collect (excel-probe skill, excel-graph): either recalculate the two shipped workbooks with the
oracle (`rustsheet-eval compare` / `rustsheet-oracle recalc`, if their size allows) and commit a sidecar, or write
a probe that reproduces the inputs: a `Summary` sheet whose B17 names a missing sheet with the same formulas, the
M7:M21 columns holding Excel's cached values for the other rows, and `=45678` for Cover!C16.

## Acceptance criteria
- [ ] A decision record (`make new-decision`) extends decision 0032 to cached values broken by the corpus
      anonymisation, citing these cells; the architecture reviewer approves it.
- [ ] eval/suites/calipers/cached-overrides.tsv gains one line per cell above (51), each pointing at a committed
      excel-graph sidecar cell with the same formula on the same inputs.
- [ ] After `make eval`, these 51 cells pass; MIN, MAX and AVERAGE have no visibly-off attributed cell in the
      "Fifteen-digit parity checkpoint" table; the 51 known-failure lines leave eval/baseline/calipers.txt
      (`make eval-baseline`) with no new lines in any baseline.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0114 `semantics`: every cell has the wrong type or error against its cached value; the target is that
each is scored against Excel's value for the shipped workbook and passes.

## Out of scope
- Other calipers cells (desktop-only rules: T-2663, T-2704; PV model last digits: T-0454, T-0458, T-0463).
- Any engine change: ours is already Excel's answer for the shipped workbook.
