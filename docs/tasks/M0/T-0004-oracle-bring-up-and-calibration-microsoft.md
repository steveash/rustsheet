---
id: T-0004
title: "Oracle bring-up and calibration (Microsoft Graph)"
milestone: M0
lane: research
status: todo
priority: P0
size: M
depends_on: []
---

## Goal
Prove the Excel oracle end to end and answer the open questions in docs/architecture/oracle.md.

## Context
Credentials are configured (app-only, `Sites.Selected` on a dedicated SharePoint site). A live
`rustsheet-oracle recalc -b excel-graph` round trip succeeded on 2026-10-04 (~11 s per workbook).

## Acceptance criteria
- [x] `rustsheet-oracle fill -b excel-graph` produces sidecars for suite `smoke`; committed (`probes`/`probes-numeric` are T-0009, `gen-m2` is T-0203).
- [ ] Calibration: recalc every ironcalc-calc workbook through Graph; report agreement with cached values (cells, by function) in docs/research/oracle-calibration.md.
- [x] Open questions answered (precision of persisted values, persistence lag); app-only support already confirmed 2026-10-04.
- [ ] `make check` green; reviewers approve.

## Out of scope
Windows desktop backend. Probe-suite fills and their excel-semantics flips (T-0009).

## Progress
- 2026-10-04 (T-0013, PR #47): open questions 1–2 answered in docs/architecture/oracle.md; every
  oracle result is canary-verified.
- 2026-10-04: smoke filled (5 canary-verified sidecars). 37/38 hand values matched Excel; the one
  that did not (d-function-criteria J2: D-function criteria are not trimmed) was corrected in the
  case, the engine and docs/excel-semantics/criteria.md. Also fixed eval scoring hand and oracle
  values side by side (unquoted hand keys) instead of letting the oracle win.
  Remaining: calibration.
