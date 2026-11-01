---
id: T-3588
title: "Keep eval math sidecars current: make check fails when a probes or probes-numeric case has a Gate A one-call cell with no math value or a stale one"
milestone: M0
lane: harness
status: done
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
- [x] `make check` fails on a case with a candidate cell that its sidecar neither holds nor lists as skipped, and on a
      sidecar entry whose `f` differs from the cell's formula, naming the case and the regenerate command.
      Tests show both failures and the pass after regenerating.
- [x] On main after T-3587 the check passes. It adds at most 10 s to `make check` on the CI runner (quote the timing).
- [x] The probe-authoring docs (the `excel-probe` skill's sidecar step, or docs/process/quality-gates.md) say to run
      `rustsheet-accuracy math --suite <suite>` after an oracle fill.
- [x] `make check` green; reviewers approve.

## Out of scope
- New Gate A definitions. Sidecars for xlsx suites (tools never parse xlsx values).

## Outcome
- **Design (skips in the sidecar):** a generator skip ("no single value", "not converged or too
  slow") can only be decided by mpmath. So `rustsheet-accuracy math` now writes those candidates
  to the sidecar as an optional `skipped` map (sheet → A1 → `f`, `why`; `rustsheet-math/v1`, which
  the runner reads past). The cheap pass can then tell a dropped cell from a missing one. Limiting
  the check to the mpmath-free reasons would have left those cells looking missing.
- **The check:** `rustsheet-accuracy math --check --suite probes --suite probes-numeric`
  (`make eval-math-check`, part of `make check`; CI's eval job runs it after `make eval-gates`)
  reports two things, each line naming the case, with the regenerate command last:
  - a candidate cell that the sidecar neither holds nor lists as skipped;
  - a sidecar entry whose `f` is not its cell's formula.
- **One definition:** `eval_math.candidates` is the single candidate rule, and both `math` and
  `math --check` use it. The check memoizes each case's arithmetic-over-literals cells and runs
  one case per process.
- **Timing:** 5.1 s for both suites on a 4-CPU container, the size of the CI runner. It was 14 s
  single-process.
- **Decision 0243** (protected paths: Makefile, ci.yml). The tests in
  `tools/tests/test_accuracy_eval_math.py` show a missing cell, a stale entry, a dropped cell
  listed as skipped, and the pass after regenerating.
- **Main was not current:** the first run listed 1,427 problems. Fresh sidecars for 10 cases had
  no sidecar at all (amordegrc-a76-bisect-a/b/d/h, the date-system daycount pair, t3352,
  t3443-basym-a/hold/sweep). Five probes sidecars held stale or missing cells
  (fin-argument-modes, solver-iteration-traces, stat-argument-modes, t2435-dist-extremes,
  t3349-t-inv-2t-near-one). The other regenerated sidecars only gained `skipped`. After
  regenerating, the check passes. `make eval-gates` is unchanged (no regressions); the sidecars
  move only R1(c)'s report counts.
- Eval delta: gate none.
