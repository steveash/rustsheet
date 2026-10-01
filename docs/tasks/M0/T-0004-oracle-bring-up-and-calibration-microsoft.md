---
id: T-0004
title: "Oracle bring-up and calibration (Microsoft Graph)"
milestone: M0
lane: research
status: blocked
priority: P0
size: M
depends_on: []
---

## Goal
Prove the Excel oracle end to end and answer the open questions in docs/architecture/oracle.md.

## Context
Requires owner-provided credentials (RUSTSHEET_ORACLE_* environment variables, work/school tenant).

## Acceptance criteria
- [ ] `rustsheet-oracle fill -b excel-graph` produces sidecars for suites `probes`, `smoke`, `gen-m2`; committed.
- [ ] Calibration: recalc every ironcalc-calc workbook through Graph; report agreement with cached values (cells, by function) in docs/research/oracle-calibration.md.
- [ ] Open questions answered (precision of persisted values, persistence lag, app-only support).
- [ ] docs/excel-semantics entries for every probe flipped from UNVERIFIED to VERIFIED/corrected.
- [ ] `make check` green; reviewers approve.

## Out of scope
Windows desktop backend.

## Blocked because
Needs owner-provided Microsoft 365 work/school tenant credentials (docs/architecture/oracle.md, docs/status.md 'Needs owner').
