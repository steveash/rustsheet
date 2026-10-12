---
id: T-1415
title: "CI: skip the Rust, eval and Python jobs on docs-only PRs"
milestone: M1
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Docs-only PRs (task files, status, decisions, skills, top-level Markdown) skip the rust, eval and
python CI jobs; every other PR and every push to `main` runs the full CI, as before.

## Context
Owner request, 2026-10-12. Decision 0049 records the rule and why `docs/excel-semantics/` is
excluded (the eval harness reads its DEVIATION anchors).

## Acceptance criteria
- [x] A `changes` job in `.github/workflows/ci.yml` classifies the PR diff; `rust`, `eval` and
      `python` skip only when it succeeds with `code=false`. Hygiene always runs.
- [x] Pushes to `main`, `workflow_dispatch`, an empty diff and a failed `changes` job run everything.
- [x] Decision 0049 records the rule; `make hygiene` green; security and architecture reviewers
      approve (protected CI path).

## Out of scope
- Path filters for other job splits (for example python-only changes skipping rust).

## Outcome
Implemented as described; this PR itself changes `.github/` so it runs the full CI, and the next
docs-only PR is the first live check of the skip.
