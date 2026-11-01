---
id: T-2981
title: "Own and route the incomplete-beta sweep cells no kernel task names (70 probes-numeric cases, about 88,500 visibly-off cells)"
milestone: M9
lane: functions-stat
status: done
priority: P1
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
Every visibly-off cell of the incomplete-beta research sweeps that no kernel task names has one owner that will fix
it: each sweep case below is assigned to the ported-kernel task whose branch computes it, and that task's text names
the case. Cells no planned branch covers get a new `campaign:` task. After this, the BETA.DIST, F.DIST, BINOM.DIST
and NEGBINOM.DIST gaps are fully owned by tasks the queue picks before the milestone order.

## Context
- Phase 1 triage (T-0090, eval of 2026-10-27 at `main` 7f6b373a; calipers, poi and corpus-bench not fetched, which
  hold none of these cells): 113 in-scope functions are below the Phase 1 rate. BETA.DIST (52,755 visibly-off of
  207,001 attributed cells, 74.5% within 15 digits), F.DIST (38,102 of 72,627, 47.5%), BINOM.DIST (3,456) and
  NEGBINOM.DIST (1,648) hold about 94% of the visibly-off cells of all 113.
- 88,502 of those cells are in the 70 probes-numeric sweep cases below, and no open, non-deferred task
  names them (T-2821 named some whole cases in its Context; it is now `defer: phase-1b`). Under the exit-owner rule
  (scripts/exit_owners.py `owned`), a task line that names a case stem and no cell owns the whole case, so this task
  owns all of these cases until each is handed on. The per-function totals: BETA.DIST 46,122, BINOM.DIST 2,919, F.DIST 37,771, F.DIST.RT 167, NEGBINOM.DIST 1,523.
- The fix is the ported incomplete-beta chain (T-2464's E1-E8): T-2506 and T-2507 are in flight, then T-2508,
  T-2509, T-2510 and T-2511 (all P1 after the triage), with T-2847 moving F, BINOM and NEGBINOM onto the bratio
  branches, T-2790 for the F density and T-2585 for the t density. The routing below is a first guess from each
  sweep's name and its research note; verify it before handing a case on.
- The single visibly-off gen-m9 cells of these functions (about 25: F.DIST, F.DIST.RT, FDIST, BINOMDIST,
  NEGBINOMDIST) are left to the T-0087 sweep.

Cases: the 70 sweep cases of the triage, plus the two T-0087 cells below, are all routed (see Outcome).

## Acceptance criteria
- [x] For each case above, `make eval-show CASE=probes-numeric/<stem>` (or the sweep's research note) shows which
      kernel branch computes its visibly-off cells, and the owning task's text names the case stem, with the
      visibly-off count, in a "Cases owned (T-2981)" list. The case is then removed from this task's list.
- [x] Cells that no open task's planned branch covers (for example, the min(a,b) <= 1 cells T-2503's M2 leaves
      visibly off, now that T-2821 is parked) get a new task with `campaign: accuracy` or `visible` and the
      `## Visible-parity stage` section, filed with `make new-task`.
- [x] Afterwards `python3 scripts/exit_owners.py --all-off` (T-0086), or the T-0090 method until it lands, shows no
      unowned visibly-off cell in these cases.
- [x] No engine code changes; `make check` green; reviewers approve.

## Out of scope
- Fixing the kernels (the chain tasks above).
- Bit-level residue of these sweeps (Phase 1b tasks).

## Visible-parity stage
Target (decision 0114): every visibly-off cell named here has an owning campaign task the queue picks before the
milestone order. This task changes no value; its stage only makes sure it is picked first in M9.

## Cells added by the T-0087 sweep (2026-10-28)
Visibly-off Excel cells no open task owned on the eval of 2026-10-28 at `main` c2b6143a (`python3 scripts/exit_owners.py --all-off`; calipers, poi and corpus-bench not fetched): F.DIST.RT 1, GAMMALN 1. This task now owns them:
- probes-numeric/m9-f-dist-rt: Calc!A19
- probes-numeric/t2464-r2-bpser-small-gl: Calc!A3188
Routing for these two (first guess, verify as for the cases above): the F.DIST.RT cell goes with the F upper tail on the ported kernel (T-2510 bgrat / T-2977); the GAMMALN cell is a GAMMALN kernel value inside the bpser sweep, for a GAMMALN follow-up of T-2123 (finished) to file.

## Outcome
Routed on a fresh `make eval` (2026-10-28, `main` 9586958b; calipers, poi and corpus-bench not fetched). With this
task's file removed, `python3 scripts/exit_owners.py --all-off --tasks <copy>` listed 87,709 visibly-off cells in
these 72 cases with no other open owner (the two bpser-small-tiny cases' 446 cells were already T-2933's). Each cell
was classified from its formula and the oracle's argument values: function, cdf or density, and for a cdf the
bratio branch `bratio::dispatch` takes (a scratch re-implementation of `dispatch` on (a, b, x, y) as SPEC-0025's
caller table forms them). Each owner's file now has a "Cases owned (T-2981)" list with the per-case counts:

| owner | what | cells | cases |
|---|---|---|---|
| T-2507 | BETA.DIST density, min(a,b) > 1 (Loader) | 43,401 | 22 |
| T-3034 (new) | F.DIST density: port T-2586's frozen M-final | 37,629 | 34 |
| T-2146 | BINOM.DIST and NEGBINOM.DIST interior pmf | 4,415 | 4 |
| T-2510 | cdf on the bgrat branches (bgrat, bup+bgrat, L140 bup+bgrat, L140 bup+bup+bgrat) | 1,879 | 13 |
| T-3035 (new) | BETA.DIST density min(a,b) <= 1 (algdiv for max > 8; shape 1 against a large shape) and BINOM.DIST(0,n,p) | 193 | 9 |
| T-2585 | T.DIST density cells of the T density probes | 73 | 3 |
| T-2508 | cdf on bfrac | 64 | 6 |
| T-3007 | cdf on already-ported branches (mostly a shape above 8, its TOMS betaln item) or at a shape of 1 | 54 | 10 |
| T-3036 (new) | GAMMALN near 1.7, the T-0087 cell | 1 | 1 |
| T-2933 (already) | bpser-small-tiny, -tiny2: fpser and apser cells | 446 | 2 |

Corrections to the first guesses above:
- The 23 t2586-r4/r5 cases are F.DIST *density* sweeps (T-2586's F density research), not F cdf cells, so they are
  not T-2847's, T-2508's or T-2510's. No task planned to ship the density model, hence T-3034.
- t2464-r2-log1p-hix is all BETA.DIST density cells with min(a,b) > 1 (T-2507), not a bpser or bup branch.
- The m9-beta-dist and w2f cases split: their density cells go to T-2507 (or T-3035 for min(a,b) <= 1), their cdf
  cells by branch to T-2508, T-2510 and T-3007 (w2f-absweep 985 and w2f-xruns-2 782 cells are L140 bup+bgrat).
- The T-0087 F.DIST.RT cell (m9-f-dist-rt A19) is on the ported L140 bup+bpser branch: T-3007, not T-2510.
- T-2847 is done, so nothing went to it.

`python3 scripts/exit_owners.py --all-off` after the routing: no visibly-off cell of these cases is unowned (the only
unowned group left in the report is ODDFYIELD's 6 probes cells, unrelated). No engine code changed.
