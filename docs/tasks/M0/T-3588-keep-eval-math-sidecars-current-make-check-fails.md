---
id: T-3588
title: "Keep eval math sidecars current: make check fails when a probes or probes-numeric case has a Gate A one-call cell with no math value or a stale one"
milestone: M0
lane: harness
status: in-progress
priority: P2
size: S
tier: opus
depends_on: [T-3587]
---

## Goal
A case filed or edited without its `.math.json`, or with one written for other formulas, cannot land: `make check`
names the case and the command that regenerates its sidecar. R1(c)'s counts (decision 0188) then never drift from
what the committed cases support.

## Context
- Found by gap triage 2026-11-01: `rustsheet-accuracy math` (T-3380) was run once on probes-numeric and never on
  probes, so 17 cases had no sidecar (T-3587) and their in-envelope cells counted as visibly off, 233 of them in the M9
  probe row. Each later Gate A slice (T-3529, T-3535, T-3549, T-3553, T-3555) reruns the tool by hand. Nothing tells
  a probe author to run it.
- The check has to be cheap. The full mpmath run takes about 2 min 20 s over both suites, too slow for every
  `make check`. A candidate-only pass is cheap: the same candidate rule as `eval_math.py` (a scored one-call cell of a
  defined function whose arguments parse), with no math computed, compared with the sidecar's cells and their `f`
  texts. But today the tool drops a candidate silently when the generator gives up ("not converged or too slow", "no
  single value"), so such a cell looks missing. The design has to record those skips (for example a `skipped` map in
  the sidecar with its reason, under `MATH_FORMAT`), or limit the check to the reasons decidable without mpmath. Pick one
  and say why in the PR.
- Where it runs: next to Gate A's `accuracy` step in `eval-gates` (Makefile), or in the tools tests that `make check`
  runs. A stale `f` (a formula edited after the sidecar was written) is already dropped by the runner
  (`MathTable::attach`); the check makes it an error instead.
- Code: tools/src/rustsheet_tools/accuracy/eval_math.py, tables.py (`math_sidecar_bytes`), cli.py;
  crates/rustsheet-eval/src/accuracy/math.rs.

## Acceptance criteria
- [ ] `make check` fails on a case with a candidate cell that its sidecar neither holds nor lists as skipped, and on a
      sidecar entry whose `f` differs from the cell's formula, naming the case and the regenerate command.
      Tests show both failures and the pass after regenerating.
- [ ] On main after T-3587 the check passes. It adds at most 10 s to `make check` on the CI runner (quote the timing).
- [ ] The probe-authoring docs (the `excel-probe` skill's sidecar step, or docs/process/quality-gates.md) say to run
      `rustsheet-accuracy math --suite <suite>` after an oracle fill.
- [ ] `make check` green; reviewers approve.

## Out of scope
- New Gate A definitions. Sidecars for xlsx suites (tools never parse xlsx values).
