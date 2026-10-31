# 0213 — Gate A's first R3 marks: the tiny-p T.INV and F.INV points of decision 0188's initial case

- **Status:** accepted (2026-10-31, T-3358). Applies decision 0188 R3 to committed Gate A points.
- **Context:** Decision 0188 R3 lists the tiny-p quantiles (T.INV, F.INV, BETA.INV and CHISQ.INV.RT at p < 1e-150,
  where Excel is grossly wrong) as an initial `MATH-OVER-EXCEL` case approved by the owner. It leaves the region's
  parameter bounds to be fixed from committed evidence (statistics.md, "Tiny-p quantiles"). T-3358's first tables for
  T.INV and F.INV (`eval/accuracy/tables/`, excel-graph, 50-digit mpmath) hold 8 points inside p < 1e-150 where Excel
  is more than G = 1e-4 off the math and the engine returns the math within ε. `eval/accuracy/math-over-excel.tsv` is a
  protected path (T-3379), so marking them needs this record.
- **Decision:**
  - The 8 points are R3-marked against the entry: accuracy/T.INV/s1 A89, A93 and A95, and accuracy/F.INV/s1 A112,
    A126, A146, A160 and A171.
  - The entry's region now names their p and df ranges (T.INV and F.INV only; BETA.INV and CHISQ.INV.RT are fixed by
    T-3529's tables).
  - The tables hold 17 points where Excel is more than G off the math. The other 9 are not marked:
    - T.INV A91 and A96, and F.INV A164 and A169, at p ≥ 1e-150, are outside the approved region (T-3544).
    - T.INV.2T A98, A99 and A103, and TINV A102, are functions outside the region at any p (T-3386).
    - F.INV A167 is inside the region, but its true quantile (3.0E-377) is below the double range. Excel returns
      3.85E-145 and the engine 0, so a mark would make 0 the target; that is left to the owner (T-3544).
  - They need an owner ruling first. They pass Gate A through its math side (A167 through the 1e-300 absolute floor),
    so nothing waits on them.
- **Consequences:**
  - The marked points are checked against the math only. Each keeps its evidence on the point: the check rejects a
    mark where Excel is not more than G off the math.
  - No eval cell is marked. R1(c) and Gate B honour a mark only with that per-cell evidence (T-3427).
  - Widening the region beyond p < 1e-150, or adding T.INV.2T and TINV, is a new R3 case for the owner.
