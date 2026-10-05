# 0026 — Parity before re-architecture

- **Status:** accepted (2026-10-05, owner request; T-0039)
- **Context:**
  - Milestones were worked in numeric order. That puts the dependency graph (M3) and storage (M4)
    ahead of most function families (M5–M10), and `perf` was on the critical-lane line.
  - Those milestones re-architect the engine. A re-architecture is only as safe as the test
    suite that checks it, and the suite still covers just the 49 M1/M2 catalog functions with
    oracle evidence.
  - The owner wants near-100% parity with Microsoft 365 Excel, on every function worth having,
    before the engine is restructured. A complete suite makes the later refactor cheaper.
- **Decision:**
  - Work happens in phases, in the order listed in docs/milestones.md, not in numeric order.
    - **Phase 1, parity:** M0, M1, M2, M5, M6, M7, M8, M9, M10, M11 (real-world fidelity).
    - **Phase 2, architecture and performance:** M3, M4, M13 (performance at scale, split out
      of M11).
    - **Phase 3, release:** M12.
    The "current milestone" (planner, gap-triager, steward critical lanes) is the first
    milestone in that order whose exit criteria are not yet met.
  - **Phase 1 exits** when all of these hold:
    - The catalog (T-0040) lists every Excel function, with its parity scope.
    - Every in-scope function has oracle-backed cases covering each parameter mode and edge
      pool, and reaches at least 99.9% bit-exact, 100% within 4 ULP.
    - Each remaining failure has a filed task or a `DEVIATION` entry.
    - The M1, M2, M5–M10 exit criteria are met, and so is M11's fidelity bar.
  - **Out of scope** are functions whose result depends on something outside the workbook:
    - web and online services: WEBSERVICE, IMAGE, STOCKHISTORY, TRANSLATE, DETECTLANGUAGE,
      COPILOT and linked data types;
    - external data and servers: RTD, the CUBE* functions, SQL.REQUEST;
    - code and add-ins: CALL, REGISTER.ID, EUROCONVERT, PY.
    The catalog records each such function as `scope: out` with a reason. If a function's value
    comes only from the environment (for example parts of INFO and CELL), that decision is made
    per argument, with probes, not assumed.
  - **Allowed during Phase 1:** directive-5 safety work (resource limits, recursion bounds,
    panics), test infrastructure (harness, generator, oracle, fuzzing), quality-lane cleanup,
    and local performance fixes that a test needs to finish. Fixes inside the existing
    evaluator stay as small as the parity fix needs.
  - **Deferred to Phase 2:** the new dependency graph, storage, structural-edit rewrite,
    benchmarks and perf gate, and parallel recalculation. The open M3 tasks are `blocked` with a
    pointer here.
  - Some parity work needs the Phase 2 graph: circular references and iterative calculation,
    dynamic dependencies, TABLE(). Its oracle cases are still written in Phase 1, so the suite
    is ready before the refactor; only the implementation waits.
- **Consequences:**
  - The steward sets the critical lanes from docs/milestones.md, so `perf` drops off without a
    routine change. Function work fills the worker runs.
  - The full function catalog (T-0040) becomes the first planning priority: the gap-triager
    and the planner can only file per-function work once it exists.
  - Phase 2 tasks unblock when the steward records in docs/status.md that Phase 1 has exited.
  - Engine performance can drift during Phase 1. Accepted: no perf gate exists yet, and the
    gate is set up at the start of Phase 2 against the code as it stands then.
