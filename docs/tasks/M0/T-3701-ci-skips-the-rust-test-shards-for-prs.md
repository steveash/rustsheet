---
id: T-3701
title: "CI skips the Rust test shards for PRs that change no Rust input"
milestone: M0
lane: harness
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A PR whose diff cannot change what the Rust tests, clippy or the Python bindings see (for example
only eval suite data, eval baselines or `docs/excel-semantics/`) skips the Rust test shards and
the bindings build, and still runs the eval gate and hygiene. Anything the classifier cannot
place still runs everything.

## Context
Owner request (2026-11-01, CI-speed investigation). The `changes` job (decision 0049, rules in
`scripts/ci_changes.py`, decision 0066) gives one answer, docs-only or not. Of the last 52 commits
on main, 38 ran the heavy jobs, but only 26 touched `crates/` or `bindings/`; 7 touched only eval
data or `docs/excel-semantics/` and paid for about 9.5 min of Rust tests anyway. Some Rust tests
and build scripts read files outside `crates/` (for example
`crates/rustsheet-xlsx/build.rs` reads `eval/suites/ironcalc-calc/cached-overrides.tsv`; the eval
crate's tests may read `eval/` and `docs/excel-semantics/` anchors), so the classifier needs an
exact, tested list of such inputs, like `CODE_READ_PATHS` and `scripts/check_docs_readers.py`.

## Acceptance criteria
- [ ] `ci_changes.py` emits separate outputs (at least `rust` and `eval`/`python`) and CI keys each
      job on its own output; the fail-safe of 0049 holds per output (unknown, empty or failed
      classification runs everything; pushes to main and the train branches run everything).
- [ ] Every file a Rust test, build script or the bindings read outside `crates/` and `bindings/`
      is in a tested list, and a hygiene check fails when a new reader is added without it.
- [ ] Unit tests in `scripts/test_ci_changes.py` cover each new class and the fail-safes.
- [ ] Required checks still report (skipped jobs pass via the summary job, as in 0122).
- [ ] A decision record amends 0049/0066; `make check` green; reviewers approve.

## Out of scope
- Changing which tests run inside a job.
