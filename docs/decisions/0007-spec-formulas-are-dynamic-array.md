# 0007 — Spec-case formulas use Microsoft 365 dynamic-array semantics

- **Status:** accepted (2026-10-01)
- **Decision:**
  - Formulas in `rustsheet-case/v1` specs mean "typed into current M365 Excel": they are written
    to xlsx as dynamic-array formulas, so there is no implicit intersection.
  - Legacy (pre-dynamic-array) behavior is tested only through Excel-authored workbooks that
    contain legacy formulas.
  - Empty-string constants are materialized as `=""`.
- **Consequences:** The xlsx builder and the engine adapter must implement identical conventions.
  Both reference this record.
