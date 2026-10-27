---
id: T-2892
title: "Gate gen-m8a and gen-m8b: gate = true with recorded baselines, so a names, 3-D or table regression fails make check"
milestone: M8
lane: harness
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`make eval-check` (and so `make check` and CI) fails when an engine change breaks a cell of gen-m8a (defined names and
3-D references) or gen-m8b (tables and structured references), as it already does for every other generated suite.

## Context
- eval/suites/gen-m8a/suite.toml and eval/suites/gen-m8b/suite.toml have `gate = false`, and eval/baseline/ has no
  gen-m8a.txt or gen-m8b.txt. They were created ungated on purpose while their gaps were open (T-1606, T-1607: "Model
  the suite on gen-m7: `gate = false`"); T-1675's Outcome notes "gen-m8b is not a gated suite, so no baseline file
  changed". gen-m7 was gated the same way once it was filled (T-2225).
- Planner run of 2026-10-27 (`make eval` on 6087413bc): gen-m8a 12 cases, 3,577 cells, 100% bit-exact, 0 pending;
  gen-m8b 9 cases, 3,368 cells, 100%, 0 pending. So gating them records baselines with `C` lines and no `F` line.
- Every other generated suite (gen-m1, gen-m2, gen-m5, gen-m6, gen-m7, gen-m9, gen-m10) is `gate = true` with
  `baseline = "cells"`. The gate reads `manifest.gate` in crates/rustsheet-eval/src/main.rs and
  crates/rustsheet-eval/src/evidence_floor.rs; no code change should be needed.

## Acceptance criteria
- [ ] Both suite.toml files say `gate = true`; nothing else in them changes.
- [ ] eval/baseline/gen-m8a.txt and eval/baseline/gen-m8b.txt exist, written by `make eval-baseline` (never by hand),
      with a `C` line per case and 0 `F` lines (or, if a cell fails on the PR's main, its F line and the owning task
      named in the Outcome). If `make eval-baseline` also ratchets quality/evidence-floor.json, that is its own output.
- [ ] `make eval-check` passes, and the Outcome shows it failing on a deliberately broken gen-m8b expectation in a
      scratch copy (not committed).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The M8 exit row and what M8's "oracle suites for each feature" means (T-2889).
- Growing either suite or new suites for external references or sheet operations.
