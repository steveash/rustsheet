---
id: T-3556
title: "SPEC: gate the fetched suites (poi, calipers, corpus-bench) so a regression in them fails a PR, and make eval runs them when they are not fetched"
milestone: M2
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
An accepted spec in `docs/specs/` that decides how the three fetched external suites (poi, calipers,
corpus-bench) are checked before a PR merges, and how `make eval` treats them when their cache is
missing, so that a regression in them fails a PR instead of being found by the next steward run, and
every `make eval` measures what Phase 1's exit reads.

## Context
- The fetched suites are not checked by CI. `.github/workflows/ci.yml`'s eval job runs `make eval-gates` in a fresh
  checkout with no `rustsheet-eval fetch`, and `select` in crates/rustsheet-eval/src/main.rs skips every suite whose
  `fetch_state()` is `NotFetched` ("skipping poi: not fetched ..."). poi and corpus-bench say `gate = true` in their
  suite.toml, but that gate applies only on a machine that has fetched them. calipers says `gate = false`.
  Decision 0067 already notes that "`make check` and CI do not run" poi. `.github/workflows/nightly.yml` still says
  the fetched suites are "downloaded here once their fetchers exist (T-0006)". The fetcher exists (T-0002) and the
  nightly job still does not call it.
- A regression got through this way. T-3120 (#2389) moved calipers `roundtrip/formulas_financial` Sheet1!E132 (IRR)
  2 ulps off Excel. CI was green. The regression surfaced only the next day, when T-3230's
  `update-baseline --suite calipers` refused it (T-3246, docs/tasks/M11/T-3246-*.md). While it stood, 2,862 fixed
  calipers cells could not be recorded.
- Phase 1 reads these suites. Gate B (decision 0188 R1(b), docs/milestones.md "Phase order") counts calipers,
  corpus-bench and poi, and M8's exit reads the poi M8 subset. On a clean checkout today (b39e1eae),
  `make eval` skipped all three and `make phase1-exit` printed `run not met ... skipped suites: calipers, corpus-bench,
  poi` and `function rate n/a Gate B not measured`. The steward fetches them by hand and runs `make eval` a second
  time on every run (docs/status.md, 2026-10-31 18:43 UTC snapshot).
- Size: calipers has 4,375,835 scored cells (docs/progress.md), far more than every vendored suite together. The
  spec has to measure the wall time and cache cost of each suite on the CI runner before it picks a design.
- Related machinery: decision 0207 (ejected PRs with baseline-only conflicts; "Fetched suites (poi, corpus-bench)"),
  SPEC-0027 (merge train: its batch tests run the same eval job), `quality/evidence-floor.json` and the visible-parity
  lock (decision 0088), which count vendored suites only (docs/process/quality-gates.md lines 17 and 62), and T-3557
  (`run --fetch-missing`, an independent CLI flag this spec may wire in).

## Acceptance criteria
- [ ] `docs/specs/SPEC-NNNN-fetched-suite-gate.md` (next free number, from the template) exists with status
      `accepted` and decides each of these, with the measured numbers behind each choice:
  - [ ] which fetched suites gate a PR, and where (the required `eval` job, a separate required job, a
        path-filtered job, or the merge train's batch run). It states the measured fetch time, eval time and cache
        size of each suite on `ubuntu-24.04`, and the total CI time budget it accepts.
  - [ ] how the cache is keyed (the pinned `rev`/`sha256` in suite.toml) and what happens when the fetch fails
        (network): the job fails, or it skips with a visible warning. If it can skip, the spec says how a skip is
        kept from hiding a regression.
  - [ ] what `make eval` does locally when a suite is not fetched: fetch first (T-3557's flag), or skip as today.
        It also says what `make phase1-exit` and the progress dashboard do with a run that skipped a suite.
  - [ ] whether calipers stays `gate = false` and, if it starts gating, how its baseline (eval/baseline/calipers.txt)
        and `cached-overrides.tsv` are handled. It also says whether the visible-parity lock and the evidence floor
        start counting fetched suites (decisions 0062, 0088), or why not.
  - [ ] a rollout of S/M tasks, each naming the protected paths it changes (`.github/`, `Makefile`, `scripts/`,
        `docs/process/`) and the decision record each needs.
- [ ] The rollout's implementation tasks are filed with `make new-task`, depend on this task and link the spec.
- [ ] `nightly.yml`'s stale T-0006 comment is listed as part of a rollout task, not fixed here (protected path).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing `.github/`, `Makefile`, `scripts/` or any baseline: the rollout tasks do that.
- Fixing any cell of the fetched suites.
- The `run --fetch-missing` flag itself (T-3557).
