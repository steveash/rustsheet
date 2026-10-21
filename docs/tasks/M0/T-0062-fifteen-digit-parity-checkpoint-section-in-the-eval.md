---
id: T-0062
title: "Fifteen-digit parity checkpoint section in the eval report: per-function Excel cells, generated cells and visibly-off count against decision 0094"
milestone: M0
lane: harness
status: done
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
- [x] The catalog takes an optional `v15: exempt` key with a reason. The PI, TRUE, FALSE and NA
      entries, the volatile functions and INFO get it. The harness rejects an exempt entry that
      has no reason.
- [x] report.json and report.md gain a "Fifteen-digit parity checkpoint" section:
  - per function: Excel cells, generated outer-call cells, exemption, visibly-off cells, and
    whether it meets the bar;
  - the functions below the bar, worst first;
  - an overall line: "Fifteen-digit parity: met | not met (coverage N/506, visibly-off cells M)".
- [x] The visibly-off total counts every suite that ran (vendored and fetched) and says which
      suites those were.
- [x] Unit tests cover counting a generated cell only for its outer call, the exemption path,
      and the overall verdict.
- [x] `.claude/skills/steward/SKILL.md` asks the steward to copy the overall line into the
      docs/status.md scoreboard each run, and to state whether the previous run also met it.
- [x] `make check` green; reviewers approve.

## Out of scope
- Raising coverage (T-2252 to T-2258).
- Making the checkpoint a gate.

## Visible-parity stage
This task is the measuring stick for decision 0094's checkpoint.

## Outcome
- Catalog: optional `v15: exempt` with a required `v15_reason` (functions.yaml, scope.json view,
  `CatalogFunction`); `Catalog::parse` rejects an exemption without a reason
  (`CatalogError::ExemptWithoutReason`). Exempt: PI, TRUE, FALSE, NA (no arguments), NOW, TODAY,
  RAND, RANDBETWEEN, RANDARRAY (volatile) and INFO (closed set of type texts).
- `v15_checkpoint.rs`: per in-scope catalog function its Excel cells, generated outer-call cells
  (`formula_calls::outer_call`: the formula is one call of the function), exemption, visibly-off
  cells, covered/meets; the overall `Fifteen-digit parity: met | not met (coverage N/M,
  visibly-off cells K)` line, with the suites counted. report.json `v15_checkpoint`, report.md
  section after "Visibly off".
- `formula_calls.rs` takes `functions_in`, `calls_only_in_scope` and the new `outer_call` out of
  report.rs, which was at the 1000-line limit.
- Steward skill: copy the line into the docs/status.md scoreboard and say whether the previous
  run met it.
- Eval delta: none (report only; no baseline line moves).
