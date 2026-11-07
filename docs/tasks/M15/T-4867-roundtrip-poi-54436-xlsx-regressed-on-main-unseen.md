---
id: T-4867
title: "roundtrip-poi/54436.xlsx regressed on main unseen: PR CI does not fetch poi, so the round-trip gate T-4629 added only runs locally and nightly"
milestone: M15
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
A future regression in a fetched-cache round-trip suite is caught before its PR merges, not after. (Fixing
roundtrip-poi/54436.xlsx itself is T-4825's, the one owner of that regression.)

## Context
Found while running `make check` for T-4830 on main at c2917a626 (2026-11-07), with the poi cache
fetched at the pinned revision (`.rustsheet-fetched`: apache/poi `9958a2a2`):

```
[roundtrip-poi] 350 cases (14 not scored), 6819353 items, 99.99% kept, 36 failing
  REGRESSION roundtrip-poi/54436.xlsx	* 296 2cd19df138a87ec5
```

`eval/baseline/roundtrip-poi.txt` lists the case (`C roundtrip-poi/54436.xlsx 995`) with no `F`
line. Main's own code gives the same result as T-4830's branch (`--filter 54436`: 78.79% kept,
losses of weight 211, mostly in the theme part's `fmtScheme`), so main regressed. Main's CI is
green because PR CI does not fetch the poi cache: the round-trip suites gated by T-4629
(`cases_of = "poi"`, "skipped while not fetched") run only where the cache exists, which is local
`make check` and the nightly fetch job (`.github/workflows/nightly.yml`). Candidates are the xlsx
import and export changes since T-4629 (#3658, 2026-11-06), for example T-4561 and T-4560 (patched
packages), T-4176, T-4148 and T-4355. Bisect with
`rustsheet-eval roundtrip --suite roundtrip-poi --filter 54436`.

## Acceptance criteria
- [ ] A change in PR CI, or an equivalent, keeps a fetched-cache round-trip regression from merging
      unseen. Either PR CI fetches the poi cache for the round-trip gate, or the gap is written
      down with its owner.
- [ ] `make check` green.

## Out of scope
- The 54436 fix: T-4825 (P0). It was bisected to T-4176 (#3800, cd0c355ad), and #3892 (a2bae4dec) since recorded it as
  `F	roundtrip-poi/54436.xlsx	* 296 2cd19df138a87ec5`, so `roundtrip --check` passes on main today and is no longer the
  symptom to clear (2026-11-07 steward follow-up).
- Other round-trip losses already in the baseline.

## Outcome
Duplicate of T-4726 (2026-11-07). T-4726 already plans this goal under SPEC-0035 D1-D3: the CI `eval` job gates poi
and corpus-bench, cells and `roundtrip-*` suites, from a cached pinned fetch that fails the job when it fails. It
depends on T-4725 (`--require-fetched`). Measured here for its budget: on 4 cores, `roundtrip --suite roundtrip-poi`
takes about 28 s and `roundtrip-corpus-bench` about 38 s. The caches are 22 MB and 33 MB.

Gating in PR CI would have caught T-4176 (#3800). It would not have stopped what hid the regression afterwards:
#3892's re-record changed the case's checked count, so the ratchet took R0's regression as R1's new coverage. That
mechanism is filed as T-4961. The 54436 fix stays T-4825's.

