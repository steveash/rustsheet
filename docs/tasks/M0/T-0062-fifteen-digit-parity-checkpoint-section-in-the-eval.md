---
id: T-0062
title: "Fifteen-digit parity checkpoint section in the eval report: per-function Excel cells, generated cells and visibly-off count against decision 0094"
milestone: M0
lane: harness
status: in-progress
priority: P1
size: M
tier: opus
campaign: visible
depends_on: []
---

## Goal
`make eval` says whether RustSheet has fifteen-digit parity as decision 0094 defines it. For each
in-scope function it shows how far that function is from the bar, so the owner and the steward
can watch one line move.

## Context
- Decision 0094 sets the checkpoint: every in-scope function has ≥ 400 Excel cells (target 500),
  ≥ 200 of them generated with `--focus`, or an exemption; no visibly-off cell in any vendored
  suite (probes-numeric included), nor in poi or corpus-bench when fetched; and both hold on two
  consecutive steward runs.
- Per-function Excel evidence (`report.json` `evidence`) and visibly-off counts (`functions[].visible`,
  T-0061, `crates/rustsheet-eval/src/closeness.rs`) already exist. Missing:
  - generated-suite cells per function, counted only where the function is the formula's outer
    call;
  - exemptions;
  - the combined verdict.
- On 2026-10-21, 358 of 506 functions were below 400 Excel cells and 5,804 cells were visibly
  off.

## Acceptance criteria
- [ ] The catalog takes an optional `v15: exempt` key with a reason. The PI, TRUE, FALSE and NA
      entries, the volatile functions and INFO get it. The harness rejects an exempt entry that
      has no reason.
- [ ] report.json and report.md gain a "Fifteen-digit parity checkpoint" section:
  - per function: Excel cells, generated outer-call cells, exemption, visibly-off cells, and
    whether it meets the bar;
  - the functions below the bar, worst first;
  - an overall line: "Fifteen-digit parity: met | not met (coverage N/506, visibly-off cells M)".
- [ ] The visibly-off total counts every suite that ran (vendored and fetched) and says which
      suites those were.
- [ ] Unit tests cover counting a generated cell only for its outer call, the exemption path,
      and the overall verdict.
- [ ] `.claude/skills/steward/SKILL.md` asks the steward to copy the overall line into the
      docs/status.md scoreboard each run, and to state whether the previous run also met it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Raising coverage (T-2252 to T-2258).
- Making the checkpoint a gate.

## Visible-parity stage
This task is the measuring stick for decision 0094's checkpoint.
