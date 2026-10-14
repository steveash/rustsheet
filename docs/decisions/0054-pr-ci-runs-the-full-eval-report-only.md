# 0054 — PR CI runs the full eval report only when the gate fails or on main

- **Status:** accepted (2026-10-14, owner request to cut CI time, T-1520). It narrows when an
  informational step runs and changes no gate.
- **Context:** The merge queue (decision 0053) is not available for a personal-account repository,
  so decision 0028's "test against current `main`" rule still costs a full CI round whenever `main`
  moves. Shorter CI makes that rarer. On 2026-10-13 (#714) the jobs took: python 12 min (11 in
  pytest), eval 4.7 min, rust 2.5 min, hygiene 0.5 min. In pytest, 17 generator tests reparsed
  `eval/catalog/functions.yaml` (0.7 s) once per seed, which made up almost all of the 12 min.
  In eval, `make eval-check` (the gate, 1.8 min) is followed by `make eval` (2.5 min), an
  informational report that reruns every suite.
- **Decision:**
  - The generator tests parse the catalog once (`functools.cache`) and hand each caller a deep
    copy (6 ms). The same 672 tests run in about 30 s.
  - CI runs the full eval report, and uploads it, only when an earlier step failed (to debug a
    failed gate), on pushes to `main`, and on manual runs. PRs and merge groups whose gate passes
    skip it. `make eval-check` itself is unchanged and still gates every PR.
- **Consequences:** PR CI drops from about 12 min to about 3 min (eval-check is now the longest
  job). A green PR has no eval-report artifact; run `make eval` locally or read the one from the
  `main` push. Rule for later changes: keep the slow informational work off the PR path, and
  keep the gates on it.
