---
id: T-3813
title: "Incomplete beta: bfrac's last unmodelled rounding per term past 10 terms"
milestone: M9
lane: research
status: todo
priority: P2
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
The last unmodelled rounding in Excel's bfrac terms is found, so the T-3571 model stays bit-exact past 10 terms, or the
negatives are recorded with their counts.

## Context
- T-3571 (docs/research/t2464/r5-bfrac/RESULTS.md section 4): the frozen Lentz model is bit-exact on 168/170 fresh
  points under 10 terms but 435/690 at 10-19, 69/232 at 20-29 and 22/108 beyond; misses are ±1-3 ulp. One-ulp nudges
  of a single β_n (three times as often as α_n) fix 409 of 468 tune misses (`nudge.py`), so one rounding per term in β
  (or α) is still Excel's and not the model's.
- Leads: β's sub-expressions in other orders and groupings ((a+n)·X/(a+2n+1) with X's sum order, fl(n·x·(b-n))/s
  against n·x/s·(b-n)); α's v/s/s against v/(s·s) at large s; the (100, 120) cells (a = 100) are the weakest, so
  terms built from a/(a+1), 1/a or n/a first. Also probe the swapped call (x above the mean): λ's form there is not
  probed (T-3812 keeps the engine's).
- Work in docs/research/t2464/ in a new `r6-*` directory, importing r5-bfrac's `model.py`, `runs.py` and `lentz.py`.
  Score on the many-term runs of t3571-r5-bfrac-tune; hold out t3571-r5-bfrac-hold and -fresh.
- Rules: positive controls first; a real hold-out; record negatives (decision 0056); truncate formula literals to 15
  digits as Excel does.

## Acceptance criteria
- [ ] A RESULTS.md records the amended model with exact, within-4-ulp and within-15-digit counts on a held-out probe
      by term-count bin, or the negatives tried with their counts.
- [ ] New oracle probes (if any) are committed under eval/suites/probes-numeric/cases/ with their sidecars.
- [ ] statistics.md's "Incomplete beta bfrac is TOMS's fraction in a Lentz loop" entry is updated.
- [ ] T-3812 (the engine port) is amended, or a follow-up is filed, for any change.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (T-3812); the density (T-3244).
