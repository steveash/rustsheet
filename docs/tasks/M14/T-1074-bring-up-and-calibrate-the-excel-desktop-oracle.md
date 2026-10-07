---
id: T-1074
title: "Bring up and calibrate the excel-desktop oracle backend"
milestone: M14
lane: harness
status: blocked
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The `excel-desktop` backend produces canary-verified sidecars on the owner's Windows machine, and a
calibration report says how far its results agree with excel-graph and with the values desktop
Excel cached in ironcalc-calc. This is the first Phase 4 task (decision 0039). Every other M14 task
depends on it.

## Context
`ExcelDesktopBackend` (tools/src/rustsheet_tools/oracle/backends.py) opens, recalculates
(`CalculateFullRebuild`) and saves a workbook over COM. It has never run: `rustsheet-oracle status`
shows `excel-desktop  n/a` in every session. The owner sets up the machine as described in
docs/architecture/oracle.md, "Owner setup — Windows desktop backend" (Microsoft 365 Apps,
auto-logon, a self-hosted runner labelled `excel-oracle`, a Trusted Location). T-0004 did the same
bring-up and calibration for excel-graph (docs/research/oracle-calibration.md). Reuse
`rustsheet-oracle calibrate`; do not write a second comparison.

## Acceptance criteria
- [ ] `cd tools && uv run rustsheet-oracle status` shows `excel-desktop OK` on the oracle machine,
      and the canary passes on desktop for one `fill -b excel-desktop --suite smoke` run. If a canary
      formula behaves differently on desktop, update canary.py and docs/architecture/oracle.md.
- [ ] `uv run rustsheet-oracle calibrate -b excel-desktop --suite ironcalc-calc` has been run. Its
      agreement table is added to docs/research/oracle-calibration.md, next to the excel-graph one.
- [ ] docs/architecture/oracle.md "Known divergences" says, for each listed divergence, which value
      desktop gave.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Replacing web goldens with desktop values (decision 0030 "Sidecars, to do"; one PR per change,
  with the evidence per cell).
- COM operations other than open, recalculate and save: typed input (T-1016), conditional-format
  evaluation (T-1051), fill handle (T-0171) and structural edits (T-0198). Each of those tasks
  adds the step it needs.

## Blocked because
Phase 4 (decision 0039). It needs the owner's Windows machine with Microsoft 365 Apps; the owner
does not plan to set it up soon. This is not a "Needs owner" item. Unblock when
`cd tools && uv run rustsheet-oracle status` shows `excel-desktop OK`.
