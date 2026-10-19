# T-1941 W4: G on [1,2), per-stage line analysis (state)

Status: structure partly pinned. Not bit-exact. Research only.

## Phase law (validated on synthetic Horner-in-y N/D, syn.c, W1 N3/D3)
- A product stage P = fl(M*y), with M a stored double, rounds at y = 1 + K*2^-52 with phase
  phase(K) = frac(u*K + v*eps*K^2 + w*eps^2*K^3), where:
  - eps = 2^-52;
  - u = M(1)/2^eR, v = M'(1)/2^eR, w = (M''(1)/2)/2^eR;
  - eR is the binade the product is rounded in.
- Far from 1, a run centred at y has local frequency F(y) = d/dt[t*m(t)] mod 1, with t = y-1 and m = M/2^eR.
- Same-binade "+ c" stages give no line. A sum that lands one binade up gives a triangle at u/2.
- Horner law between consecutive multiplicands: m_{j+1}(t) = 2^d*(1+t)*m_j(t) + const, i.e.
  b_k = 2^d*(a_k + a_{k-1}).

## Tools (gline_tools/)
- efit2.py: coherent low-n0 ladder fit with a wide initial b scan. The old efit window (|v| < 2.8) aliased.
  Not committed.
- vspec2.py / vspec3.py: incoherent (v,w) spectrum over the high runs, n0 in (3e11, 1.3e14).
  - Validated on synthetic data: (-4.382, -2.5) recovered against truth (-4.380, -2.67).
  - Second check: (5.875, 10.2) against truth (5.882, 9.67).
- yruns.py and track2.py: residuals of the G.txt long runs against mpmath, and frequency tracks F(y) across
  [1,2). dense.py builds the same json from the y101..y116 probe (extract_runs output).
- chain.py: pairwise Horner-link search. Not committed.

## Independent lines (f at 1+, v, w, m_k from tracks)
| line | f | v (m1) | w (m2) | track |
|---|---|---|---|---|
| L1 | 0.4094486825857 | +0.2494 (ladder +0.25012) | -0.30 | to 1.046 |
| L2 | 0.4230053820552 | +0.1870 | +0.05 | m = u+0.187t+0.05t^2+0.02t^3, to 1.44 |
| L3 | 0.1250589147893 | -0.1560 | +0.0085 | to 1.88 |
| a3048 | 0.3048029747 | +0.006 | ~0 | to 1.18 |
| a1099 | 0.1098510050 | -0.6113 | ~0 | to 1.18 |
| a4299 | 0.4298616996 | -0.225 | +7.5? | short |

Harmonics, not stages:
- 0.1811 = 2*L1; 0.2283 = 3*L1; 0.3622 = 4*L1; 0.0472 = 5*L1;
- 0.2197 = 2*a1099;
- 0.2859 = L1+a3048.

## Chain link found
a1099 (u = +1.10985) -> L1, with e_L1 = e_a + 1. Three predictions agree:
- L1 m1: predicted 0.2493, observed 0.2494.
- L1 w: predicted -0.305, observed -0.30.
- F_L1(1.236): predicted 0.4761, observed 0.4765.

The added constant is c = 2^e_a*(2*u1 - 1.10985), with u1 in {1.40945, -0.59055, -1.59055}.

## Ruled out here
- Cody SPECFUN GAMMA coefficients: 88,566 of 260,096 exact.
- Boost lanczos13m53 num/expG/den stage frequencies: the nearest is 2.6e-4 off, against about 1e-9 for a true
  stage.
- The form 1 + t*R: the w104 lines at t < 4e-12 need O(1) products by y.

## Needed
Dense oracle runs: 4096 consecutive doubles at y = 1 + j/64, to track every stage across [1,2) and pin the chain
and the binade crossings (1.365-1.370, 1.5438-1.545). The coordinator generated them as kmyst-gam-y101..y116.

## Update: structure solved (not yet bit-exact)
- G(y) = fl(N(y)/D(y)) on [1,2). N and D are independent raw-y Horner chains of degree (8,8), every op
  fl53(fl64).
- It is Cody-family: (xnum+xden)/xden from Cody's GAMMA, re-expanded in y, but with Excel's own coefficients.
- The forms 1+X/D and (X+D)/D are excluded, because both suppress D-stage errors near 1.
- See the coordinator README for the line to stage map and the pins.
- Fit: least squares of Gamma*D - N under the pins (null-space elimination plus QR at dps 60-100), then an
  exact-match coordinate search along the 6 remaining soft directions.
- Best candidate: G_candidate_fit4.txt (N ascending y^0..y^8, then D ascending). 29.9% exact on 260,096 G points,
  72% within 1 ulp. Not bit-exact: the remaining soft directions span 1e2 to 1e7 ulps per coefficient.

## Update 2: per-stage phase solve (W2 method), partial
- stagephase4.py demodulates one stage line per ladder run with the candidate's own phase law, comparing data
  with an exact simulation (simres.c). It unwraps the phase difference rung by rung.
  - Sawtooth: dphi = K*dM(y_K)/2^e.
  - Triangle: dphi = const + dS'(1)*K/2^(eS+1) + ...; the constant term is required.
  - Calibrated on a known perturbation: -5.03e-13 recovered against -4.94e-13 expected.
- Convergence:
  - D.c0 (L1) converges in 1-2 iterations: deltas a = -3e-14, b = 3e-10, c = -1e-6; rms 0.005 cycles to
    K = 3e12.
  - D.S1 (L3) and N.S0 (L2) also converge (rms 0.004-0.005).
- joint.py (not committed) is one linear system over all 18 coefficients, the three clean lines, soft Gamma-fit
  rows and a ridge.
  - It is consistent to K = 5e11 (rms 0.005), but its moves run along the near-null common-factor directions.
  - The ladder near y=1 pins only about 3 Taylor functionals per stage, so the exact score does not improve and
    a second relinearisation diverges.
- Blocker: the near-null directions need phase data far from y=1.
  - Answer: ladders at anchors 1.25, 1.5 and 1.75 (kmyst-gam-e701..e718).
  - Alternative: march outward over the dense y101..y116 runs.

## Playbook lessons (for docs/research/excel-kernel-playbook.md)
1. **Triangle lines.** A product P = fl(M*y) rounded into a sum S = fl(P + c) that lies one binade up carries the
   parity of P_r, whose integer part moves with M's own rounding.
   - So the line follows the smooth sum: F(y) = S'(y)/2^(eS+1) (mod 1, sign-folded), with
     F(1) = frac(S'(1)/2^(eS+1)).
   - It is NOT frac(u/2) and NOT (M + tM')/2^e. Read with the sawtooth law, triangle lines look like
     unassignable stages.
   - With the correct law every strong G line mapped onto one (8,8) N/D structure.
2. **Phase comparison against exact simulation.**
   - For a triangle line the data-vs-sim phase difference has a constant offset (from dS(1)) plus
     dS'(1)*K/2^(eS+1)+..., so the unwrap basis needs a constant column.
   - For a sawtooth line the difference is K*dM(y_K)/2^e, with no constant.
3. **Frequency-derived pins are biased for weak lines.** A line fitted with the wrong curvature alias shifts f by
   about dv*eps*2*n_typ, about 5e-6 for an alias jump of 13 in v. Pin with strong lines only, then switch to
   data-vs-simulation phases.
4. **Separate the curvature ambiguity before pinning.** The coherent low-n0 fit aliases |v| outside its scan
   window. An incoherent (v,w) spectrum over the highest runs, with local frequency
   f + 2v*eps*n0 + 3w*eps^2*n0^2, resolves it to about +-0.005.
5. **Long-range frequency tracks across the interval** (runs of >= 1024 consecutive doubles at many y) give whole
   stage curves. That is the strongest stage-assignment test: a stage must match the full curve, not just y=1.
6. **Doublets.** The Gamma residue at 0 forces n0 ~ d1 in a raw-y N/D, so N.c0 and D.c1 share a frequency to about
   2e-5. Treat that line as two stages.
