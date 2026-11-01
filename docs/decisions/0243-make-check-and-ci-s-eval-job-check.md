# 0243 — make check and CI's eval job check the eval math sidecars (T-3588)

- **Status:** accepted (2026-11-01, T-3588).
- **Context:**
  - R1(c) (decision 0188) counts a visibly-off numeric cell inside Gate A's envelope only through
    its case's `*.math.json` sidecar. Nothing told a probe author to run `rustsheet-accuracy math`,
    so 17 cases went without one (T-3587), and a formula edited after its sidecar is silently
    dropped by the runner (`MathTable::attach` reads it as stale).
  - The full mpmath run takes about 2 min 20 s over probes and probes-numeric, too slow for every
    `make check`. A candidate-only pass needs no mpmath. It must, however, tell a cell the
    generator dropped ("no single value", "not converged or too slow") from one nobody ran the
    generator for, and only mpmath can decide those reasons.
  - `rustsheet-eval cases` lists a suite's cases (`suite_cases`), so the check needs the release
    binary and `tools/`. In CI only the eval job builds that binary, and only the python job has uv.
- **Decision:**
  - The sidecar records the candidates the generator dropped under an optional `skipped` map
    (sheet → A1 → `f`, `why`), within `rustsheet-math/v1`. The runner reads past it.
  - `rustsheet-accuracy math --check --suite S ...` fails on a candidate cell that the sidecar neither
    holds nor lists as skipped, and on an entry whose `f` is not its cell's formula. Each line names
    the case, and the last line gives the regenerate command.
  - `make eval-math-check` runs it for probes and probes-numeric, and `make check` includes it.
    CI's eval job adds `astral-sh/setup-uv@v6`, the action the python job already uses, and runs it
    after `make eval-gates`, which built the binary.
- **Consequences:**
  - A probe case that lands without its sidecar, or after an edit that leaves the sidecar stale, fails
    `make check` and CI. The excel-probe skill and quality-gates.md say to run
    `rustsheet-accuracy math --suite <suite>` after an oracle fill.
  - The candidate rule has one definition (`eval_math.candidates`), which both `math` and
    `math --check` use.
  - CI's eval job gains the uv setup, about 1-3 s with uv's cache, plus the check itself.
