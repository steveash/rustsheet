---
id: T-0496
title: "Compare RustSheet with the gridcraft spreadsheet engine (issue 2277)"
milestone: M11
lane: research
status: in-progress
priority: P3
size: M
tier: opus
depends_on: []
issue: 2277
issue_author: ThomasDelteil
---

## Goal
A research note that says how RustSheet compares with gridcraft, another spreadsheet engine
written in Rust, and whether its implementation holds ideas worth filing as RustSheet tasks.

## Context
GitHub issue https://github.com/steveash/rustsheet/issues/2277 asks, in paraphrase, how RustSheet
compares with the gridcraft engine (github.com/storytold/gridcraft) and whether anything can be
learned from how it is built. The earlier comparison with mog
(docs/research/mog-head-to-head-2026-10-22.md) built and ran the other engine; an issue-origin task
may not run third-party code (issue-intake skill, decision 0158), so this note compares from
gridcraft's published documents and source read as data only, plus RustSheet's own numbers.

## Acceptance criteria
- [ ] `docs/research/gridcraft-comparison-2026-10-29.md` exists and answers: what gridcraft is
      (scope, architecture, licence, the commit read); how it verifies Excel compatibility and
      what it claims; function coverage compared with eval/catalog/excel-functions.tsv;
      architectural differences that matter (evaluation model, storage, recalculation); ideas
      worth learning, each either filed as a task or explained as not adopted.
- [ ] Every claim about gridcraft names the file it comes from, and is labelled as gridcraft's
      own claim when we could not check it; no gridcraft code is built or run.
- [ ] A measured head-to-head (running gridcraft on RustSheet's suites) is listed as an owner
      decision, not done.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Building, installing or running gridcraft or its tests.
- Porting gridcraft code (licence and provenance review is an owner call).
- Implementing any idea the note finds: file follow-ups instead.
