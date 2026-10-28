---
id: T-2761
title: "Gate eval-check and the evidence floor from one suite run"
milestone: M2
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
The eval CI job judges the baseline gate (`eval-check`) and the evidence floor from one run of the
suites, so the floor no longer costs a second build and a second suite run on the merge lock's
critical section (decision 0075), with exactly the same gate results. `make evidence-floor` keeps
working on its own.

## Context
Owner request (2026-10-25): PR CI wall time is the merge lock's hold time. In the eval job,
`make eval-check` (gate suites) is followed by `make evidence-floor`, which runs every vendored
suite again. On PR #1842 (run 40242117468) the floor step took 2:11: 1:57 rebuilding the release
binary (crates/rustsheet-engine/build.rs re-runs on every cargo call because it watches
`crates/rustsheet-engine/.git/HEAD`, which does not exist, so the whole engine recompiles) and
13 s running the suites. The floor counts the vendored suites (`fetch` unset): every gate suite
that is vendored plus the non-gate gen-m8a, gen-m8b and probes-numeric.
Code: `cmd_check` and `cmd_evidence_floor` in crates/rustsheet-eval/src/main.rs.

## Acceptance criteria
- [x] `rustsheet-eval check --evidence-floor` runs the gate suites and the remaining vendored
      suites in one `execute`, judges the baseline gate from the gate suites' results and the
      evidence floor from the vendored suites' results, and fails when either fails. It reads no
      earlier run, so it can never judge a stale tree. `--suite` is refused with it.
- [x] `make eval-gates` runs it; `make check` and the CI eval job use it instead of
      `eval-check` + `evidence-floor`. `make eval-check` and `make evidence-floor` are unchanged and
      still work alone.
- [x] On main, the combined output equals `make eval-check`'s output followed by
      `make evidence-floor`'s verdict line, and the exit status is the same.
- [x] Unit tests cover the suite plan (gate suites first, then each vendored suite not yet in it)
      and the floor's result selection (vendored only, same set as the standalone command).
- [x] Decision record for the CI and docs/process change, with the measured saving.
- [ ] `make check` green (done); reviewers approve (pending).

## Out of scope
- `make eval-baseline` (update-baseline then `evidence-floor --update`) still runs twice; local only.
- The engine build script's missing rerun path (a separate task).

## Outcome
`rustsheet-eval check --evidence-floor` (`make eval-gates`) judges the baseline, visible-parity
and evidence-floor gates from one run (gate suites, then the other vendored suites:
gen-m8a, gen-m8b, probes-numeric); `make check` and the CI eval job use it, decision 0125.
On base f4d4be481 the old `check` output and the old `evidence-floor` verdict line equal the new
combined output line for line (apart from the order of parallel stderr pivot warnings), all
exits 0; with the lock edited to 1 (a stale lock) both the old command and the new one exit 1
with the same message. Locally (4 cores): before, `make eval-check` 2:55 plus
`make evidence-floor` 2:41 (2:25 of it the needless rebuild, T-2755); after, `make eval-gates`
2:57 (build 2:21, suites 36 s). Follow-up: T-2755 (engine build script re-runs on every cargo call).
