# 0218 — Allow four GEOMEAN regressions from keeping the product's power of two out of the logarithm (T-3502)

- **Status:** accepted (2026-10-31, T-3502): the T-3502 PR's review panel (security, architecture, quality,
  Excel fidelity) approved this record. It applies CLAUDE.md directive 2's decision-record exception to four
  probes cells and weakens no gate.
- **Context:**
  - T-3502 re-validates GEOMEAN's 15-digit snap of the in-range root (T-1123,
    `FITTED(T-1123, 170 cells, ...)`) under decision 0188 R4. On the fresh log-uniform sweep
    probes-numeric/t3502-geomean-fresh (180 cells, excel-graph, 2026-10-31) the snapped
    `exp(ln(P)/n)` matches 58 cells and is up to 194 ulps off (79 cells beyond 15 digits): ln's rounding
    error at |ln P| up to 700 reaches the root.
  - Keeping P's power of two out of the logarithm, `exp(ln(m·2^r)/n)·2^q` with P = m·2^E and
    E = q·n + r (q toward zero), matches 169 of the 180 fresh cells (hold-out train 110/119, test 59/61)
    and is within 1 ulp on all of them. It is a structural model: no fitted constant, and it is
    `exp(ln(P)/n)` itself where |E| < n. On the in-range probes it matches 167 of 176 cells (the snap
    141). The FITTED tag goes (quality/fitted.json 81 to 80). numbers.md "GEOMEAN in-range root" has the
    scored models.
  - Against `main` it fixes 58 known failures (gen-m9 26, probes 32) and regresses none in gen-m9 or
    ironcalc-calc. It moves four probes cells one ulp off the bit: geomean-root-refinement Sheet1!A23
    `GEOMEAN(8.26,6.3,6.7,1.41,6.14,8.77,7.92,6.5,5.8,2.8)` (Excel 5.426261309079341, ours
    5.42626130907934), A33 (3.631815615869783, ours 3.6318156158697836), A34 (3.920282312333767, ours
    3.9202823123337676) and A44 `GEOMEAN(6.6,3.6,3.03,4.73,1.6)` (3.525770483125708, ours
    3.5257704831257075). None becomes visibly off; in-scope visibly-off cells fall from 2427 to 2424.
  - No model scored keeps all four and the fixes: Excel's root behaves like a library root of up to
    about 0.8 ulp (numbers.md, "GEOMEAN in-range root"). The correctly rounded root of P keeps them only
    by breaking ironcalc-calc GEOMEAN C23 and G27 and 14 other probes cells. Decision 0083's standing
    allowance does not cover this PR (it is not a campaign task).
- **Decision:** run `rustsheet-eval update-baseline --allow-regressions` once in the T-3502 PR for exactly
  these four probes cells. T-3526 owns them.
- **Consequences:** eval/baseline/probes.txt gains exactly these four lines and the baselines lose the 58
  fixed ones. Reviewers confirm the baseline diff adds no other line. Any later PR regressing a cell needs
  its own record.
