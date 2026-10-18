# 0073 — Untrusted-cache suites keep the web number over a nearby cached value

- **Status:** accepted (2026-10-18, T-0433)
- **Context:**
  - Decision 0030 rule 1, as T-0142 implemented it in `rustsheet-oracle fill`
    (`_drop_desktop_conflicts`), overwrites a web number with the workbook's cached value. It does so
    when the two differ by less than 1e-9 and the workbook's AppVersion is 16.x.
  - In corpus-bench this produced 373 cells that fail only in the last digit, across seven
    workbooks: 49278, 54490, 53068, 52216, 49036, 38655 and 47766. Excel for the web computed
    exactly the engine's value in every one. The cached value had 15 significant digits. Two
    examples:
    - 47766 `Total (2)!I26` `=D26/C26` is 0.4782608695652174 on the web and 0.478260869565217
      in the cache.
    - 49036 `Trades!H2` is 4028.999999999998 on the web and 4029 in the cache.
  - Excel's own saves keep 17 digits (docs/architecture/oracle.md). A 15-digit cache therefore
    comes from a writer's cut, not from desktop Excel's arithmetic.
  - SpreadsheetBench recalculated its workbooks with LibreOffice or win32com, and writers other
    than Excel also claim AppVersion 16.0300. corpus-bench/suite.toml already sets
    `use_cached_values = false` for that reason.
- **Decision:**
  - Rule 1 of decision 0030 does not apply to a suite with `use_cached_values = false`. For such a
    suite the fill keeps the web number whenever the cached value is within 1e-9. Its caches are
    not desktop evidence, and rule 3 makes the web golden when no desktop evidence exists.
  - A material difference (more than 1e-9 relative, or a different type) is still left pending
    under decision 0019 in these suites. A cache that disagrees materially may still come from a
    real desktop recalculation (win32com) and expose a web-vs-desktop divergence. Leaving the cell
    pending only costs coverage. Scoring it against either value could teach the engine the wrong
    behavior.
  - `rustsheet-eval cases` reports each xlsx suite's `use_cached_values` in a
    `suite_uses_cached_values` field, so the tools never re-read suite.toml.
- **Consequences:**
  - The seven corpus-bench sidecars were refilled from excel-graph, using the new
    `rustsheet-oracle fill --case <id>` filter. They now hold the web numbers, and their 373
    last-digit cells pass.
  - poi (`use_cached_values = true`) and ironcalc-calc are unchanged: their 16.x caches still win
    within 1e-9.
  - The 1651 cells pending as material conflicts in corpus-bench (PENDING.md) stay pending.
  - Adding a suite with untrusted caches needs nothing beyond `use_cached_values = false`.
