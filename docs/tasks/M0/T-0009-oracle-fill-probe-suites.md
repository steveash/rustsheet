---
id: T-0009
title: "Oracle-fill suites probes and probes-numeric (excel-graph)"
milestone: M0
lane: research
status: todo
priority: P0
size: S
depends_on: []
---

## Goal
Commit Excel's answers for the probe suites so the engine tasks waiting on them (T-0106, T-0107,
T-0108, T-0110) have oracle evidence to implement against.

## Context
The excel-graph oracle is live (docs/architecture/oracle.md). Split out of T-0004 so each fill
and each semantics flip has exactly one owner; T-0004 keeps `smoke`, calibration and the open
questions, T-0203 owns `gen-m2`, T-0109 owns `gen-m1`.

## Acceptance criteria
- [ ] `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes` and `--suite probes-numeric` write a `*.oracle.json` sidecar for every case; committed.
- [ ] `make eval` scores both suites (no longer "pending oracle"); new known failures recorded with `make eval-baseline` (new expectations, not regressions).
- [ ] docs/excel-semantics entries backed by these probes flipped from UNVERIFIED to VERIFIED or corrected, citing the case ids.
- [ ] `make check` green; reviewers approve.

## Out of scope
Engine fixes for the probed behavior (T-0106, T-0107, T-0108, T-0110); calibration (T-0004).
