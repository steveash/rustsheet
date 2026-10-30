---
id: T-3163
title: "Nightly fuzz: formula_eval never starts fuzzing, its 802,100-file seed corpus outlasts the 45-minute job"
milestone: M2
lane: harness
status: in-progress
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
The nightly `fuzz (formula_eval)` job finishes its initial corpus pass and fuzzes for its 20
minutes inside the job's 45-minute limit, so directive-5 coverage of formula evaluation resumes
and the grown corpus is saved again.

## Context
Nightly fuzz run 25 (2026-10-29 03:41 UTC, `.github/workflows/fuzz.yml`, main at 5b36426c) was
cancelled: `fuzz (formula_eval)` hit `timeout-minutes: 45` inside the "fuzz for 20 minutes" step.
The log shows no crash or slow input: libFuzzer loaded `802100 files found in corpus/formula_eval`
(31.8 MB) and was still in its initial pass at about 380-400 exec/s when the job was cancelled
(last pulse `#524288 ... corp: 13642/291Kb` at 04:11). `-max_total_time=1200` does not cut that
pass short, and a cancelled job neither saves the grown corpus nor uploads artifacts, so the
corpus never shrinks and every later night will fail the same way. `formula_parse` reads the same
seeds at a higher exec/s and passed. The seeds are one file per distinct formula in eval/suites
(`fuzz/seed_corpus.py`), plus the corpus cache grown on earlier nights.
Fix options for the implementer to weigh (decision 0035 governs the workflow): minimize the
restored corpus with `-merge=1` (or `cargo fuzz cmin`) before fuzzing; sample or cap the
suite seeds per target in `seed_corpus.py`; and/or give the initial pass its own budget. Changing
the workflow is a protected path: needs a decision record and explicit approval from
reviewer-architecture and reviewer-security. Filed by the 2026-10-29 18:47 UTC steward run.

## Acceptance criteria
- [ ] A `workflow_dispatch` run of the nightly fuzz workflow on the branch (or the first nightly
      after merge) finishes `fuzz (formula_eval)` green within the job limit, with fuzzing time
      after the initial pass.
- [ ] The corpus that is saved stays bounded (state the bound and why) so the job cannot grow
      back past the limit.
- [ ] `make check` green; reviewers approve.

## Out of scope
- New fuzz targets.
