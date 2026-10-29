# 0163 — Allow one F.DIST density regression where M-final replaces the beta density

- **Status:** accepted (2026-10-29, T-3034)
- **Context:**
  - T-3034 replaces the F density (the beta density times y(1 - y)/x, with a Loader fallback) with Excel's own
    chain as T-2586 froze it, M-final (`docs/research/t2464/r5-f-final/fdensity_excel.py`; statistics.md "F density
    is not the beta density, R's df or a GAMMALN formula (T-2513)"). The port equals the model bit for bit on all
    64,883 positive research rows. In probes-numeric, F.DIST's visibly-off cells fall from 38,027 to 18,722 and its
    bit-exact cells rise from 5,595 to 16,268; in the gated suites 5 cells are fixed (4 gen-m9, 1 ironcalc-calc).
  - One gated cell fails now that matched before: `gen-m9/8fec7b207f4dd753` Calc!A65,
    `F.DIST(1,9,Data!C2,...)` with C2 = "1", that is F.DIST(1,9,1,FALSE). Excel 0.22913073333963166, now
    0.22913073333963163 (1 ulp; the exact density is 0.22913073333963160824...). The model gives the same value: it
    is M-final's known small-df residue (one df of 1 with a larger partner, r5-f-final RESULTS.md section 4), which
    the old form happened to land on.
- **Options considered:**
  - *Keep the old form for small df, or for df2 = 1.* By the 36 regions of r5-f-final's map the old form loses
    everywhere (df2 = 1: 2,246 of 3,437 research cells within 15 digits, against M-final's 2,902), so it would give up
    hundreds of cells to keep one.
  - *Special-case the cell or the shape (9, 1).* That fits to individual cells, which decision 0056 forbids.
- **Decision:** run `rustsheet-eval update-baseline --allow-regressions` once in the T-3034 PR, for exactly this cell.
  It stays with T-2586's open small-df constant residue.
- **Consequences:** `eval/baseline/gen-m9.txt` gains this one line and loses 4; `eval/baseline/ironcalc-calc.txt`
  loses 1. Reviewers confirm the baseline diff adds no other line.
  Ungated probes-numeric cells the old form had bit-exact, such as t2586-r5-f-bigconst-a B889, can be lost too; they
  are in the T-3034 Outcome's per-cell count (2,591 out against 21,404 in) and stay with T-2586's open residue.
