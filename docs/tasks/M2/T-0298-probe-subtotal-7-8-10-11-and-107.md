---
id: T-0298
title: "Probe SUBTOTAL 7/8/10/11 and 107-111 one-pass vs two-pass variance and overflow"
milestone: M2
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's SUBTOTAL variance and standard-deviation codes are shown to use (or not) the same one-pass/two-pass switch and overflow `#NUM!` as VAR/AGGREGATE/DVAR, which the engine now assumes.

## Context
Steward spot-audit 2026-10-09 of PR #448 (T-1185, 6851c3e), reviewer-excel-fidelity on opus. T-1185 moved
SUBTOTAL 7/8/10/11 and 107/108/110/111 onto the shared `variance_result`/`excel_variance`
(`functions/statistical/variance.rs`). The only SUBTOTAL oracle data (probes/subtotal-hidden-filtered,
a few gen-m5 cells) uses small, well-spread values, so it exercises the one-pass branch only (Hide!C10
matches to the ulp). Nothing shows SUBTOTAL switches to two-pass on offset/cancelling data, or returns
`#NUM!` for `SUBTOTAL(11,{1E+200,-1E+200})`-style overflow. DVAR got this probe in T-1054
(probes/dvar-one-vs-two-pass); copy its data for SUBTOTAL. Record the result in
docs/excel-semantics/statistics.md next to the database-variants paragraph (one-pass VERIFIED,
two-pass and overflow UNVERIFIED until this lands) and fix numbers.md if it overstates.

## Acceptance criteria
- [ ] An oracle-filled probe runs SUBTOTAL 7/8/10/11 and 107/108/110/111 over offset, small-magnitude and 1E+200 data.
- [ ] statistics.md records the SUBTOTAL rule; any engine mismatch has a filed task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- AGGREGATE (already probed).

## Outcome
Added probes/subtotal-variance-one-vs-two-pass (72 cells; SUBTOTAL 7/8/10/11 and 107/108/110/111 over offset,
small-magnitude and 1E+200 data), oracle-filled on excel-graph. The engine already matched all 72 cells
(offset and small-variance sets take the two-pass form, a well-spread set the one-pass form, and
`{1E+200,-1E+200}` is `#NUM!`), so no engine task was needed. Recorded in
docs/excel-semantics/statistics.md. Eval delta: +72 passing probe cells (new case recorded in eval/baseline/probes.txt), no regressions.
docs/excel-semantics/numbers.md was checked: it only claims SUBTOTAL 1, 6, 9 overflow and says 101/106/109 are unprobed, so it needed no change.
