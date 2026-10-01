# 0006 — One xlsx value parser; tools shell out to it

- **Status:** accepted (2026-10-01)
- **Decision:** `crates/rustsheet-eval/src/xlsx_snapshot.rs` is the only code that decodes
  stored xlsx values for scoring. It is deliberately independent of the engine's importer, so an
  importer bug cannot leak into the expectations. Python tooling calls `rustsheet-eval snapshot`.
- **Consequences:** Value decoding fixes happen once. The reviewer-quality agent rejects new parsers.
