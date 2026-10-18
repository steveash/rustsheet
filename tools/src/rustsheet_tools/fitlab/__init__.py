"""The fit lab: score candidate kernel implementations against an oracle sweep (T-1523).

`rustsheet-fitlab <fit>` loads a sweep (an oracle-filled probe case), runs every candidate and
the engine's current values on its cells, and ranks them by bit-exact and within-4-ulp counts.
See docs/architecture/fit-lab.md. Modules: `sweep` (inputs), `score` (verdicts, ranking),
`exact` (correctly rounded arithmetic and the x87 model), `fit` (a fit and its run), `kernels`
(shipped fits), `cli`.
"""
