---
id: T-0009
title: "Oracle-fill suites probes and probes-numeric (excel-graph)"
milestone: M0
lane: research
status: done
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
- [x] `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes` and `--suite probes-numeric` write a `*.oracle.json` sidecar for every case; committed.
- [x] `make eval` scores both suites (no longer "pending oracle"); new known failures recorded with `make eval-baseline` (new expectations, not regressions).
- [x] docs/excel-semantics entries backed by these probes flipped from UNVERIFIED to VERIFIED or corrected, citing the case ids.
- [x] `make check` green; reviewers approve.

## Out of scope
Engine fixes for the probed behavior (T-0106, T-0107, T-0108, T-0110); calibration (T-0004).

## Outcome
- Sidecars for all 3 `probes` and 4 `probes-numeric` cases (excel-graph, 2026-10-04). Values persist
  at full precision (`0.1+0.2` → 0.30000000000000004).
- Two inputs held `1E+308` literals, which make Excel reject the workbook: probe
  `precision-and-rounding` A13 now uses `10^308`; the generator's `1e308*10` became `1e307*100`
  (seed 1 regenerates only case 7b06c92d26d6101d → 4f413cc257ce15c4).
- Eval: probes 71.11% (13 new-coverage known failures recorded), probes-numeric 60.19% (not gated),
  0 regressions.
- excel-semantics: near-zero snapping corrected (SUM and `(a+b)-c` do snap); ROUND, number→text,
  comparison tolerance, 15-digit truncation, 1900-02-29, wildcards, D-function prefix VERIFIED;
  new entries for subnormal → 0, ≥1E+308 literals, DATE years < 1900, and `"="` vs "".
  `"<>text"` stays UNVERIFIED (no probe cell).
- Follow-ups: T-0010 (oracle client retries connection resets), T-0011 (remaining criteria probe gaps).
