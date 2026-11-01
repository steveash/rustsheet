---
id: T-3714
title: "The python CI job stops waiting on its own release rustsheet-eval build"
milestone: M0
lane: harness
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The python CI job's `make py-check` no longer spends about 2.5 minutes with its pytest workers
blocked on a `cargo build --release -p rustsheet_eval` that the tests trigger on demand.

## Context
Follow-up of T-3700 (decision 0233). In run 41294866508 (PR #2733), pytest with `-n auto` passed
46% of the 1,248 tests within 10 s, then stalled from 09:51:59 to 09:54:24 while
`rustsheet_tools.snapshot` built the release binary; the rest took about 30 s. `make py-check`
took 199 s, of which the tests themselves were about 40 s. The eval job builds the same binary
on another runner at the same time. Options to measure:
- start the build as its own step before pytest, overlapping ruff and mypy (a small win);
- make the tests that need the binary use a cheaper build;
- share the eval job's binary as an artifact, which serializes the jobs (likely worse);
- check which tests need a real binary at all and give the rest a stub.

## Acceptance criteria
- [ ] The python job's `make py-check` step is measured before and after in real PR runs, and
      the Outcome records which option won and why.
- [ ] Every test that runs today still runs; `make check` green; reviewers approve.

## Out of scope
- The eval job's own release build.
