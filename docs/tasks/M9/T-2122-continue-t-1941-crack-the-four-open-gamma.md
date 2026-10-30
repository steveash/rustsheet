---
id: T-2122
title: "Continue T-1941: crack the four open gamma kernels to the bit"
milestone: M9
lane: research
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-1941]
---

## Goal
Bit-exact models (structure, op order, x87 rounding, coefficient doubles) for the four kernels T-1941 left open:
K1 = GAMMA's G(y) on [1,2), K2 = GAMMALN on (2,3], K3 = GAMMALN near 1 on [0.7,1.5), and GAMMALN on (1.5,2].
Every other piece of GAMMA, GAMMALN and LN is already bit-exact or reduces exactly to these four.

## Context
- Restart file: `docs/research/t1941/HANDOFF2.md` (read §12 first: final worker states and pickup steps) on branch `agent/research/T-1941-crack-gammaln-gamma-ln`
  (self-contained: solved pieces, best models, the method that took K2 from 0.74 to 0.95, traps, dead ends,
  ranked hypotheses, data inventory). The chronological log is `HANDOFF.md` (sections 1-12.20) and `README.md`.
- Best models at handoff (exact / within 1 ulp vs Excel):
  - K2: `k2_ref.py`, 0.950 / 0.978. Reviewer 3's stored cand9 is 0.993 near 2 (HANDOFF2 §12.1). Structure: stored
    is favoured by the near-2 ladders; the far-only transfer control is still to run.
  - K1: `w4/k1r/plain1.txt`, 0.776 / 0.878. The [1.8,2) dip is 0.50 and looks like a coefficient problem.
  - K3: `w5/p5` / `w5/p4/ss_fc.txt`, 0.33. Ladder probes `kmyst-gam-k3l*` are filling, with an unwrap tool ready.
  - (1.5,2]: W6's chain in `w6/data/`, 42.7% exact on R.
- Engine shipping of the current best models is T-2123 (independent of this task).

## How to run (restart prompt)
Paste this to a fresh coordinator session (Opus, auto mode), on the research branch above:

> You coordinate T-2122 on branch `agent/research/T-1941-crack-gammaln-gamma-ln` (research only; never
> force-push; commits `T-2122: ...` with the Co-Authored-By / Claude-Session trailers; never print oracle
> credentials). First read `docs/research/t1941/HANDOFF2.md` completely, then skim `HANDOFF.md` §12.11-12.20.
> Spawn five sessions with `create_session` on the same branch:
> - W-K2 (Opus): HANDOFF2 §3.2 / §7.1 / §12.1. Reviewer 3 argues K2 is plain stored and the ~5% residual is a
>   per-run offset c3 at N.s3. Have the coordinator fill the §12.1 micro-ladders. Unwrap c3(x) and identify
>   its law. Run the far-only-refit transfer control. Then refit (near-2 constraint, cluster bit-match plus
>   bias rms) so that one set beats both cand9 near 2 and k2_ref far from 2.
> - W-K3 (Opus): §3.4 / §7.2. Unwrap the k3l ladders with `w5/p5/ladder.py`, solve per-stage slopes for
>   coefficient moves (c·g head-move shape), peel, refit. Request 6 more anchors if the N slopes reach ~1e6 cycles/x.
> - W-K1 (Opus): §3.3 / §7.3. Dip-weighted null-lattice search from plain1 and the dip-only set. Then the D P2
>   line unwrap in [1.8,2); specify `kmyst-gam-k1l*` ladder probes if the dense runs don't unwrap.
> - W-15 (Opus): §3.5 / §7.4. Run the noise fingerprint (`w5/fp/`) on (1.5,2]. Then alias scans at N.P3 and D.P1
>   with the correct periods, then per-stage phase laws and the c·g head-move recipe.
> - Reviewer (Fable/strongest): critique each worker's claims against §5 (traps). Demand planted controls and
>   holdouts. Propose structural forms when a worker stalls.
> Rules for everyone: planted control before any real-data claim. Hold out (alternate clusters, long runs,
> near-2 ladders). Never treat a mask gain as structure evidence. Never normalise by a non-power-of-two D0.
> Compensate head moves at the next constant only (late chain fixed). Select oracle columns by formula.
> Coordinator duties:
> - fill probe requests with the oracle (`cd tools && uv run rustsheet-oracle fill -b excel-graph --suite
>   probes-numeric --case probes-numeric/<id>.case.json ...`, ≤ 16,384 rows per case, commit landed oracles per
>   batch);
> - commit worker scripts and coefficient files under `docs/research/t1941/`;
> - append dated sections to HANDOFF2.md;
> - route results between workers;
> - verify every claimed model yourself on the 14 long runs / dense clusters before committing it;
> - keep a ~40-minute self check-in;
> - when a kernel goes bit-exact (≥ 99.9% with only documented x87 near-ties), update T-2123's model list and
>   `docs/excel-semantics/statistics.md`.
- Run 3 (2026-10-22, one opus worker, ~30 min, K3 only): HANDOFF2 section 15, `JOINTALL=1` in `t2122/k3_n6first.py`.
  The joint N6+N7 fit with alias freedom on every row passes the 12-anchor planted control (N7 -1.344e9 against truth
  -1.344e9) and fits all 12 real anchors with one head move, N7 +4.007e8, N8 -4.611e10, N9 +3.503e12 ulps (10/14 rows,
  rms 2.65e4 cycles/x): the a-f move. Run 2's g-l conflict was the missing N6 alias freedom. Scored after a full
  function refit (15.3), that move beats the zero-move refit control by 3.9 points (0.298 against 0.260) but not the base
  (0.322). The refit itself costs 6 points, so next is a bit-preserving conditional solve of the lower chain (15.3).
- Run 4 (2026-10-23, one opus worker, K3): HANDOFF2 section 16, `t2122/k3_lower.py`. With the head fixed, a joint
  N3+N5 solve recovers the control's N4-N6 to 0.2%. On real data it keeps all 10 N3 rows: N4 +3.9e7, N5 -1.6e9,
  N6 +7.4e8, which agrees with ladder.py's joint consensus. The D chain is still unidentified below D6. The 15.3 refit
  score swings up to 10 points on a 1-ulp change of N6, so run 3's +3.9 points for the head move is noise. Next: a
  score that is invariant to the lower chain's low bits (HANDOFF2 16.2).
- Run 5 (2026-10-28, one opus worker, K3 step 16.2.1): HANDOFF2 section 17, `t2122/k3_reread.py`, `k3_null.py`,
  `k3_tol.py`. Re-reading the ladders with a moved set as base gives a score that the lower chain's low bits do not
  affect. It is calibrated on the control, and its reads are the next correction: Newton steps converge to the
  ladder's floor. Real data: run 4's move A is right, and two Newton steps give set `k3_run5_C.txt`, with 21/29 rows
  at zero (all 11 N3 rows within ±150 cycles/x). C's total move matches run 1's independent consensus. Remaining 1σ
  errors: N9 4.5e6, N8 3.2e3, N5 1.7e3 and N4/N6/N7 40-200 ulps, against exact-rate tolerances of about ±400, ±10
  and ±1. Then (17.5) scan2's N.S7 box scan from C (3σ box, 9.6 M points): the planted control finds its truth at
  rank 0 (949 against a base bias of 644), but on real data nothing in the box beats the base ridge (702). Either
  the top is more than 3σ out, or Excel keeps S7 in 80-bit; the second is the live hypothesis. Next: the
  unstored-top control and the ladder re-read under it (17.6). The unstored-top control (S7 and S8 in 80-bit, 17.7)
  reproduces the real scan: no peak, and a base ridge of 658 against real 702. Next: the ladder re-read with an
  unstored top (17.6 step 2), then KN = 8. The re-read turned out blind to the top's stores (17.8), so the next test is
  a J=6 (N.S6) box scan from C, on the KN = 7 control first.
- Run 7 (2026-10-29, one opus worker, K3 18.5 step 1): HANDOFF2 section 19, scan3.py `BIASFILE`/`BIASKN`,
  `t2122/k3_j6bias.sh`. Subtracting the base's own 80-bit-top residual before the J=6 scan makes the KN = 7 control rank its
  planted S_6 first with no b cut (1468 against the base's 1432). On real data the same subtraction leaves the base ridge and
  the mirror sidelobes, with no isolated top, so the real store structure is not "S_7/S_8 80-bit over C". Next: use BIASKN
  as a store-structure meter (19.3).
- Run 8 (2026-10-30, one opus worker, K3 19.3 step 1): HANDOFF2 section 20, `t2122/k3_meter.sh`. The bias subtraction is a
  store-structure meter: on the real data it steps once, between BIASKN 7 and 6 (base 1379 -> flat, sd 175 -> 23; 99 and 8
  change nothing, 5 equals 6). A KN = 6 control scanned with the KN = 7 bias reproduces the real KN-7 map point for point
  (base, ridge, mirror sidelobes) and is flat with the right one. So Excel keeps S_6 (and above) in 80-bit: KN <= 6, and
  the J=6 store scans of runs 6-7 cannot find a top. Next: the J=5 scan (quartic term) on a KN = 6 control, then real (20.3).

## Acceptance criteria
- [ ] Each of K1, K2, K3 and (1.5,2] has either a bit-exact model (≥ 99.9% of all probed points, misses
      only at documented x87 near-ties) committed with its verifier, or an updated best model plus a measured
      ceiling and the remaining hypotheses in HANDOFF2.md.
- [ ] Every new probe case is filled and committed. Coefficient files and scripts are under `docs/research/t1941/`.
- [ ] Quirks recorded in `docs/excel-semantics/statistics.md` ("GAMMA, GAMMALN and LN kernels"). Lessons added
      to `docs/research/excel-kernel-playbook.md`. T-2123's model list updated.
- [ ] `make check` green; reviewers approve (merged to main by the research branch's PR).

## Out of scope
- Engine implementation (T-2123).
- Incomplete gamma / beta kernels (T-0971, T-0970).
