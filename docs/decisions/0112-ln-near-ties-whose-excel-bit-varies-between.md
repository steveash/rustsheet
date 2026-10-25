# 0112 — LN near-ties whose Excel bit varies between fills stay named known failures

- **Status:** accepted (2026-10-25, T-2144). It changes no scoring, baseline, ratchet, exit measure or CI check.
- **Context:**
  - probes/ln-fyl2x-kernel (tag M9; LN is catalog M5) has 27 known failures. Each is one ulp at an FYL2X near-tie, where the
    exact product `RN64(ln 2) * log2 x` lies about half a 64-bit ulp from a 53-bit midpoint, so the 64-bit rounding
    decides the last bit (numbers.md "LN is FLDLN2 then FYL2X").
  - T-2143 calculated one 380-point workbook three times within 35 s (probes-numeric/t2143-ln-near-ties-fill1..3). The
    fills disagree at 23 points, always between two adjacent doubles. So Excel's bit there is not a function of x. A
    difference between servers' CPUs is a plausible cause, but it is unprobed.
  - T-2144 fitted guard-bit FYL2X models: a k-bit internal product, truncated or rounded, before RN64 and RN53, for k =
    66..72 (docs/research/t1941/ln_guard_bits.py; numbers.md has every count). The best, k = 67 rounded, does at least
    as well as the shipped model on every set it was scored on:
    - fill-stable points: 354 of 357 (shipped 345), losing none the shipped model matches
    - sampled fixture points: all 4,000
    - fixture near-ties: 18 of 30 (shipped 0)
    - probes/ln-fyl2x-kernel: 242 of 253 cells (shipped 226).
    It still misses 3 fill-stable points, so T-2144's criterion for changing `excel_ln` (every fill-stable point) is not met.
    The fixture sample holds few near-ties, so it cannot show the model is safe where it differs.
- **Options considered:**
  - *(a) Accept either neighbour at these cells* (a scoring rule like decision 0042's pair). The evidence that Excel's bit
    varies exists only at the 23 T-2143 points, not per probe cell. Not chosen.
  - *(b) Ship k = 67 rounded now.* It dominates the shipped model on all the evidence scored here. Its risk lies in the
    near-ties of the full T-1941 set (2.89M cells), which T-2144 did not score. Deferred to T-2641, which ships it if it
    loses no point there.
  - *(c) Keep the 27 cells as named known failures of probes/ln-fyl2x-kernel until T-2641 decides.* Chosen.
- **Decision:**
  1. The shipped model stays (`excel_ln`, `RN53(RN64(RN64(ln 2) * log2 x))`) until T-2641 decides on k = 67 rounded. The 27
     cells stay F lines of eval/baseline/probes.txt and are owned by T-2641: probes/ln-fyl2x-kernel Calc!B10, B16, B26,
     B33, B48, B67, B78, B83, B95, B99, B107, B113, B114, B117, B129, B132, B140, B163, B164, B179, B190, B197, B208, B233,
     B238, B246 and B249, each one ulp off.
  2. This decision grants no milestone-exit exemption. The cells count like any other known failure of their probe
     row. A later exemption needs its own decision, under the probes criteria of docs/milestones.md.
- **Consequences:**
  - No engine change now. T-2641 may fix up to 16 of the 27 cells. A cell whose Excel bit varies between fills cannot be
    matched by any deterministic model, so whatever T-2641 decides, some near-tie cells may remain.
