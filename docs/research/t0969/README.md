# T-0969 attempt 6 data (2026-10-20)

`tcells.tsv`: every literal-argument T.DIST / T.DIST.RT / T.DIST.2T / TDIST cell in
`eval/runs/latest/report.json` on main a15f80ae, deduplicated. Columns: function, t, df, flag
(TRUE/FALSE/tails, empty when absent), Excel's value (excel-graph oracle), the engine's value.

The candidate formulas scored against it, and their results, are listed in the task file's
"Attempt 6" section (`docs/tasks/M9/T-0969-t-dist-t-dist-rt-t-dist-2t.md`). Each candidate was a
throwaway `#[cfg(test)]` module in `functions/statistical/` built from `gammaln::excel_gammaln`,
`gamma_x87::excel_gamma`, `x87_exp::excel_exp`, `x87_log10::excel_ln` and the `arithmetic.rs`
`extended_*` operations. The data is valid for main a15f80ae; rerun `make eval` and re-extract it
for a later engine.
