# 0130 — Replace BASE.xlsx with upstream BASE_DECIMAL.xlsx (T-2819)

- **Status:** accepted (2026-10-26, T-2819)
- **Context:** IronCalc b716e550 replaced its Excel-authored calc test `BASE.xlsx` with `BASE_DECIMAL.xlsx`,
  a superset (Microsoft Excel 16.0300, calcChain, 17-digit cached values; 358 formula cells covering BASE and
  DECIMAL against 61 before). Taking it deletes `BASE.xlsx`, which `scripts/check_governance.py` treats as a
  deleted test file.
- **Decision:** delete `crates/rustsheet-xlsx/tests/calc_tests/MATH_AND_TRIGONOMETRY/BASE.xlsx` and add
  `BASE_DECIMAL.xlsx` in the same change. No coverage is lost: the new workbook scores 358 cells, all passing,
  and the baseline case line is renamed by `make eval-baseline`.
- **Consequences:** the ironcalc-calc suite scores more Excel-computed cells, known failures stay at 638 and no
  gate is weakened. Later upstream workbook renames of this kind cite this record.
