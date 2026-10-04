# 0019 — Fetched suites keep sidecars in the repo; web results that contradict desktop Excel stay pending

- **Status:** accepted (2026-10-04, T-0019)
- **Context:**
  - Fetched suites (`poi`) live in the git-ignored `eval/.cache/suites/`, so a sidecar written next
    to the case would never be committed.
  - The oracle is Excel for the web. Filling POI showed that it sometimes contradicts the values
    desktop Excel cached in the same workbook: lowercase-`e` scientific formats (`#VALUE!` on the
    web, 238 cells), elapsed tokens after a plain `s` (10 cells), and `CELL("width")`, which the
    web does not implement (9 cells). Scoring those cells against the web would have counted the
    engine's matching `#VALUE!` as parity.
- **Decision:**
  - A suite may set `oracle_dir` (relative, inside the suite directory); sidecars then live there,
    mirroring the case path. Validated at load time.
  - `rustsheet-eval cases <suite>` is the one definition of case discovery, sidecar location and
    cached-value trust (`SuiteManifest::trusts_cached_values`) for tools.
  - `rustsheet-oracle fill` drops every cell where the web value materially differs from a value
    cached by a Microsoft Excel application (numbers beyond 1e-9 relative; other types exactly)
    and prints it as `CONFLICT`. Those cells stay pending; the semantics entry gets
    `Status: CONFLICTING` with both values; nothing is implemented until an excel-desktop probe on
    current Microsoft 365 decides.
  - Formulas the web does not implement are refused for the whole workbook, like SHEETS/INFO.
- **Consequences:** fewer scored cells, but no parity claim rests on web-only behavior that desktop
  Excel contradicts. A general policy for web-vs-desktop disagreements without desktop cached values
  (e.g. last-digit float differences) is out of scope here.
