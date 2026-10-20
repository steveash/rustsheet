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
- Engine shipping of the current best models was T-2123, split into T-2128 (GAMMALN) and T-2129 (GAMMA); `gamma_kernels.rs` holds the coefficient arrays to swap.

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
> - when a kernel goes bit-exact (≥ 99.9% with only documented x87 near-ties), update `gamma_kernels.rs` (a new engine task) and
>   `docs/excel-semantics/statistics.md`.

## Acceptance criteria
- [ ] Each of K1, K2, K3 and (1.5,2] has either a bit-exact model (≥ 99.9% of all probed points, misses
      only at documented x87 near-ties) committed with its verifier, or an updated best model plus a measured
      ceiling and the remaining hypotheses in HANDOFF2.md.
- [ ] Every new probe case is filled and committed. Coefficient files and scripts are under `docs/research/t1941/`.
- [ ] Quirks recorded in `docs/excel-semantics/statistics.md` ("GAMMA, GAMMALN and LN kernels"). Lessons added
      to `docs/research/excel-kernel-playbook.md`. An engine task is filed to swap `gamma_kernels.rs`'s arrays.
- [ ] `make check` green; reviewers approve. Land notes, coefficient files and verifiers on main in a slim PR, as the T-1941
      close-out did. Bulk probe data (`kmyst-gam-*`, gigabytes) stays on the research branch.

## Out of scope
- Engine implementation (T-2123, T-2128, T-2129 and follow-ups).
- Incomplete gamma / beta kernels (T-0971, T-0970).
