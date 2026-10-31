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
  - Points of the same functions with p ≥ 1e-150 where Excel is also more than G off are not marked: T.INV A91 and
    A96, F.INV A164 and A169, T.INV.2T A98 and A103, and TINV A102. They are outside the approved region and need an
    owner ruling first (T-3386 has the two-tailed ones). They pass Gate A through its math side, so nothing waits on
    them.
- **Consequences:**
  - The marked points are checked against the math only. Each keeps its evidence on the point: the check rejects a
    mark where Excel is not more than G off the math.
  - No eval cell is marked. R1(c) and Gate B honour a mark only with that per-cell evidence (T-3427).
  - Widening the region beyond p < 1e-150, or adding T.INV.2T and TINV, is a new R3 case for the owner.
