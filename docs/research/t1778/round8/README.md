# T-1778 round 8: Excel EXP model V6

`exp_v6.py` is the reference model (numbers.md, "EXP is an x87 2^(a*log2 e) evaluation"). Scores:
- 2,139,591 of 2,139,684 EXP cells across all kmyst-erfc-* probes (reported by the cloud worker; the 93 misses are
  F2XM1 near-ties);
- 60,000 of 60,000 on an independent random sample of EXP(-(x*x)) cells (coordinator check).

A hardware test for a non-Intel F2XM1 (the cloud worker's hwtest.c) is described in the round-8 notes of the
research plan. The 3,709 "decided" cells, where the floor/ceil choice of u changes the result, can be rebuilt by
scoring V6 with u = floor64 and with u = ceil64 of 2^|f| - 1, keeping the cells where exactly one matches Excel.
