---
id: T-1523
title: "Fit lab: score candidate kernel algorithms against the oracle sweeps, bit-exact and within 4 ulp"
milestone: M9
lane: harness
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
A reusable fit lab: give it a sweep (oracle-filled probe case) and candidate implementations of a kernel, and it reports per candidate how many cells are bit-exact and within 4 ulp, with the worst cells, so a fit is a ranked comparison instead of guesswork.

## Context
Owner request 2026-10-14 (decision 0056); the near-miss figures and their source are in T-1522. The oracle sweeps for every major kernel exist (T-1040 to T-1046, T-1004 to T-1009, `eval/suites/probes-numeric/cases/m9-*`). `docs/research/t1187-erfc-candidates.py` is a one-off version of this for erfc. The fit-without-sweep rule (decision 0026) and the time-box (decision 0056) both depend on it.

## Acceptance criteria
- [x] `tools/src/rustsheet_tools/fitlab/` (uv project, ruff and mypy strict): reads sweep inputs and Excel values through the existing readers (`tools/src/rustsheet_tools/snapshot.py`, the oracle package's sidecar reading; never re-parsing xlsx), runs candidate functions, prints a ranked table and writes JSON.
- [x] Exact arithmetic helpers: correctly rounded results from `fractions`, rounding to a p-bit significand (53 and 64), so candidates can model x87 double rounding.
- [x] One worked example: the engine's current kernel and at least two candidates for one kernel, run on its sweep.
- [x] It replaces `docs/research/t1187-erfc-candidates.py`: those candidates (T-1187) are ported into the lab and the script is removed. The lab only reads: it never edits `eval/baseline/` or case files.
- [x] Documented in `docs/architecture/` (how to add a candidate).
- [x] `make check` green; reviewers approve.

## Out of scope
- Porting a winning candidate into the engine (the kernel's own fit task does that).

## Outcome
`rustsheet-fitlab` (tools/src/rustsheet_tools/fitlab/, docs/architecture/fit-lab.md) scores candidates on
an oracle sweep. It reads the case and sidecar JSON through `rustsheet-eval cases` and the engine's values
through `rustsheet-eval show --all`. `suite_cases` moved from the oracle CLI to `snapshot.py`, so both
tools share one discovery path. The lab prints a ranked table (bit-exact, within 4 ulp, within 1e-9, worst
ulp), per-family unions and the worst cells, and writes JSON. `fitlab/exact.py` has `round_to_bits`
(p = 53 and 64), the x87 model (`double_round`, `X87`, `Ext`), and correctly rounded sqrt, exp, ln and pow.

Worked example (ERFC, T-1040 sweeps, excel-graph):

| fit | candidate | exact | within 4 ulp |
|---|---|---|---|
| `erfc` (530 cells) | engine today | 198 (37.4%) | 526 (99.2%) |
| | `math.erfc` (glibc) | 174 (32.8%) | 518 (97.7%) |
| | `1 - math.erf` | 88 (16.6%) | 130 (24.5%) |
| `erfc-tail` (205 cells, x >= 1) | best T-1187 candidate (`bu_even` n >= 100, `(1/big)*(k*ISQPI)`) | 58 (28.3%) | 201 |
| | engine today | 44 (21.5%) | 203 |
| | `math.erfc` | 37 | - |

All 5,794 `erfc-tail` candidates together get 177/205 cells exactly (174 without the engine and libm).
docs/research/t1187-erfc-candidates.py moved into the lab as the `erfc-tail` fit
(fitlab/kernels/erfc.py) and was deleted. The fit reproduces the script's counts exactly: best 57, 58,
38 and 4, union 174.

