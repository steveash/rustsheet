# 0079 — make parity-report target for the generated parity page

- **Status:** accepted (2026-10-19, T-2041)
- **Context:** the owner wants a page on GitHub to link people to that shows RustSheet's
  per-function Excel parity (bit-exact, within 1 ulp, within 4 ulp, within 1e-9, off). It must be
  generated from the eval report, never hand-edited, and refreshable with one command. The
  Makefile is a protected path (scripts/check_governance.py), so the new target needs a record.
- **Decision:** the Makefile gains `make parity-report`, which runs the tools entry point
  `rustsheet-parity-report` on `eval/runs/latest/report.json` and `eval/catalog/scope.json` and
  writes `docs/parity.md`, `docs/parity/functions.md` and the SVG charts in `docs/parity/`. It
  only reads eval output and writes docs; no gate depends on it and it changes no existing target.
- **Consequences:** `make eval && make parity-report` refreshes the page; anyone (the steward
  included) may commit the regenerated files. The generated files are not hand-edited. The page is
  a snapshot that names its run date and commit, so it can lag `main` without being wrong. Nothing
  under docs/ is read by code, so decision 0066's CODE_READ_PATHS is unchanged. docs/parity.md and
  docs/parity/ are committed snapshots: CI does not check that they match the latest report, and
  no freshness gate may be added for them (a gate would fail every PR that moves the numbers,
  and the pages already name the run they show).
