# 0089 — A kept subnormal is saved to xlsx as 0; EXP_SIGN B44 is an M5 exit exception

- **Status:** accepted (2026-10-20, T-2204). It names one more exit exception under decision 0087 and
  weakens no gate: no baseline, ratchet or CI check changes.
- **Context:**
  - `ironcalc-calc/MATH_AND_TRIGONOMETRY/EXP_SIGN.xlsx` EXP_SIGN!B44 is `=EXP(A44)` with A44 = -745. Its
    cached value (Microsoft Excel Online) is 0. The engine gives 2^-1074 (x87 model V6, T-1931). The cell
    is one of the M5 row's "failing excl. exceptions".
  - Excel keeps that value in memory. probes/subnormal-saved-as-zero (excel-graph, 2026-10-20):
    `(EXP(-745))=0` is FALSE (B1), and `EXP(-745)*2^1000` is 5.2939559203393771E-23 = 2^-74 exactly
    (C1), so the kept value is exactly 2^-1074. `=A1` (A3) reads it from the cell (B3 FALSE, C3 2^-74).
  - The saved file holds 0. The excel-graph oracle reads values from the xlsx that Excel for the web
    saves (tools `oracle/cli.py` `_recalc_cached`). In that file (the git-ignored oracle cache
    `eval/.cache/oracle/excel-graph/609761bee2d9…d69.xlsx`, input of the probe above; not durable, rebuilt
    by `rustsheet-oracle recalc -b excel-graph` on the case) A1, A2 and A3 are
    `<v>0</v>`, while C1 is `<v>5.2939559203393771E-23</v>` and A4 (`EXP(-708)`, normal) is
    `<v>3.3075530036384078E-308</v>`. So "Graph reports a kept subnormal as 0.0" (decision 0042) is
    really "Excel writes a kept subnormal to the file as 0", and B44's cached 0 is the same artifact
    of Excel's own save, not a different value. The durable evidence is the committed sidecar (A1 = 0 with
    B1 FALSE and C1 = 2^-74) and B44's own raw `<f>EXP(A44)</f><v>0</v>` in the Excel Online workbook.
    Both are web saves; desktop Excel's save is not probed.
  - Binding measures: decision 0085 item 3 (a cell further than 4 ulp, or not numeric, blocks; 0 and
    2^-1074 are adjacent doubles); decision 0087 (named exceptions live in
    eval/milestones/exit-exceptions.tsv, added in the PR that takes the decision); decision 0052
    (DEVIATION entries are directive-5 limits only); prime directive 2.
- **Options considered:**
  - *(a) Name B44 as an M5 exit exception.* This extends the exceptions of decision 0085 (desktop-copied
    cells, item 2; time-boxed fits, item 3) with a third class, a cached value that is Excel's save artifact. The engine already holds Excel's in-memory value, and the
    file cannot show it, so no engine change can pass the cell. Chosen.
  - *(b) Extend decision 0042 to excel-cached expectations*, passing a cached 0 against an engine
    subnormal when an excel-graph probe of the same formula over the same inputs has a FALSE `(x)=0`
    companion. It would turn B44 into a pass, but it needs a cross-case evidence lookup in the
    harness for one cell today. Filed as T-2210 (P3), not required for the exit.
  - *(c) Leave B44 blocking.* It would block M5 on a value no engine can produce while matching
    Excel's arithmetic. Rejected.
  - *A DEVIATION entry.* Not allowed: this is not a directive-5 limit (decision 0052).
- **Decision:**
  1. `EXP_SIGN.xlsx!EXP_SIGN!B44` is a line of eval/milestones/exit-exceptions.tsv from M5, owned by
     T-2210. It stays an F line of eval/baseline/ironcalc-calc.txt.
  2. Later cells: an excel-cached (or excel-graph) 0 against an engine subnormal is this save artifact
     only when Excel evidence shows the value is kept, that is, an excel-graph probe of the same
     formula over the same inputs has a FALSE `(x)=0` companion. Such a cell may be named an exit
     exception citing this decision, in the PR that adds that evidence. Without the evidence it is an
     ordinary failure: where Excel flushes (`*`, `/`, PRODUCT and the rest of numbers.md "Subnormal
     results become 0"), the companion is TRUE and the engine is wrong.
- **Consequences:**
  - The M5 row's "failing excl. exceptions" drops by one. When T-2210 lands, B44 passes and its line
    is stale (report.md lists it) and is removed.
  - Decision 0042's rule is unchanged. Its context says Graph reports a kept subnormal as 0; this record
    corrects that to the saved file (numbers.md now says so), without editing decision 0042.
  - RustSheet's own xlsx export should write a kept subnormal as 0 as Excel does: T-2211.
