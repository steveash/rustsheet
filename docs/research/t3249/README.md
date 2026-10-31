# T-3249: BETA.DIST(.,FALSE) at large shapes (oracle sweep)

- `mkprobe.py` rebuilds both cases byte for byte (`python3 docs/research/t3249/mkprobe.py --write` from the repo root;
  it overwrites the case files and `probe_*.meta.json`). It reuses T-2464's exact encoder (`../t2464/w2-forensics`).
- `analyze.py` scores the engine against the sidecars (no fitting): `analyze.out` was taken on the PR's head.
- Filled on 2026-10-30 with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric --case
  probes-numeric/t3249-beta-large-shape-fit.case.json --case probes-numeric/t3249-beta-large-shape-hold.case.json`
  (the backend's canary sheet passed; 30 s for both). `--case` takes the suite-relative id with `.case.json`.

Changes to the task's sketch:
- The hold-out uses runs of 16 doubles, not 32: all its shapes (9 a values by 7 b values, less (1025, 1.5)) at 32
  doubles and 9 runs would be 17,856 rows, above a case's 16,384-row limit. Its levels are -35, -75, -150, -250,
  -350, -450 and -600, plus a run that starts at the mode and goes up (the fit's mode run is centred on the mode).
- Every run sits below the mode. Above the mode ln f falls only like (b - 1) ln(1 - x), which does not reach these
  levels for small b. Every level was reachable for every shape, so none was skipped.
- The F.DIST column covers every row, since 2a and 2b are integers for every shape.

Findings (from `analyze.out`; statistics.md "Beta density for min(a,b) > 1 is Loader's saddle point" has the summary):
the miss grows with |ln f| at every shape and does not collapse above 400. The sd is 1 at the mode run and 257-283
units of 2^-52 at ln f = -600 to -650. The F density at the mapped point grows the same way.
